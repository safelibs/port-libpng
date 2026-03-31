use crate::chunks::{call_error, call_warning, read_core};
use crate::read_util::{validate_chunk_name, ReadPhase, PNG_IEND, PNG_SIGNATURE};
use crate::state;
use crate::types::*;
use core::ptr;

unsafe extern "C" {
    fn upstream_png_set_read_fn(
        png_ptr: png_structrp,
        io_ptr: png_voidp,
        read_data_fn: png_rw_ptr,
    );
}

unsafe extern "C" fn png_safe_progressive_buffer_read(
    png_ptr: png_structp,
    out: png_bytep,
    length: usize,
) {
    let mut short_read = false;

    state::update_png(png_ptr, |png_state| {
        let progressive = &mut png_state.progressive_state;
        let end = progressive.decode_offset.saturating_add(length);
        if end > progressive.buffered.len() {
            short_read = true;
            return;
        }

        unsafe {
            ptr::copy_nonoverlapping(
                progressive.buffered.as_ptr().add(progressive.decode_offset),
                out,
                length,
            );
        }
        progressive.decode_offset = end;
    });

    if short_read {
        unsafe { crate::error::png_error(png_ptr, b"progressive short read\0".as_ptr().cast()) };
    }
}

fn read_be_u32(bytes: &[u8]) -> png_uint_32 {
    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn complete_png_length(buffer: &[u8]) -> Option<Result<usize, &'static [u8]>> {
    if buffer.len() < PNG_SIGNATURE.len() {
        return None;
    }
    if buffer[..PNG_SIGNATURE.len()] != PNG_SIGNATURE {
        return Some(Err(b"Not a PNG file\0".as_slice()));
    }

    let mut offset = PNG_SIGNATURE.len();
    while offset.checked_add(12)? <= buffer.len() {
        let length = usize::try_from(read_be_u32(&buffer[offset..offset + 4])).ok()?;
        let name = [
            buffer[offset + 4],
            buffer[offset + 5],
            buffer[offset + 6],
            buffer[offset + 7],
        ];
        if !validate_chunk_name(name) {
            return Some(Err(b"invalid chunk type\0".as_slice()));
        }

        let next = offset.checked_add(8)?.checked_add(length)?.checked_add(4)?;
        if next > buffer.len() {
            return None;
        }

        if u32::from_be_bytes(name) == PNG_IEND {
            return Some(Ok(next));
        }

        offset = next;
    }

    None
}

fn unread_bytes_from_last_call(progressive: &crate::read_util::ProgressiveReadState) -> usize {
    let start = progressive.current_input_start;
    let end = start.saturating_add(progressive.current_input_size);
    let consumed = progressive.decode_offset.max(start);
    end.saturating_sub(consumed)
}

fn note_progressive_pause_bytes(png_ptr: png_structrp, save: bool) -> usize {
    let mut unread = 0;

    state::update_png(png_ptr, |png_state| {
        unread = unread_bytes_from_last_call(&png_state.progressive_state);
        png_state.progressive_state.last_pause_bytes = unread;
        png_state.progressive_state.paused_with_save = save;
    });

    unread
}

unsafe fn decode_buffered_progressive_png(png_ptr: png_structrp, info_ptr: png_inforp) {
    unsafe {
        upstream_png_set_read_fn(
            png_ptr,
            ptr::null_mut(),
            Some(png_safe_progressive_buffer_read),
        );
    }

    if let Some((progressive_ptr, info_fn, row_fn, end_fn)) =
        crate::io::progressive_read_registration(png_ptr)
    {
        let _ = progressive_ptr;
        let info_emitted = state::get_png(png_ptr)
            .map(|png_state| png_state.progressive_state.info_emitted)
            .unwrap_or(false);
        if !info_emitted {
            unsafe { crate::read::png_read_info(png_ptr, info_ptr) };

            if let Some(callback) = info_fn {
                unsafe { callback(png_ptr, info_ptr) };
            }
            state::update_png(png_ptr, |png_state| {
                png_state.progressive_state.info_emitted = true;
            });
        }

        if (read_core(png_ptr).flags & crate::common::PNG_FLAG_ROW_INIT) == 0 {
            unsafe { crate::read::png_start_read_image(png_ptr) };
        }

        let core = read_core(png_ptr);
        let rowbytes = if core.rowbytes != 0 {
            core.rowbytes
        } else {
            crate::chunks::read_info_core(info_ptr).rowbytes
        };
        let mut row = vec![0; rowbytes];

        loop {
            let before = read_core(png_ptr);
            if before.pass >= 7 || before.row_number >= before.num_rows {
                break;
            }

            let row_num = before.row_number;
            let pass = before.pass;
            unsafe { crate::read::png_read_row(png_ptr, row.as_mut_ptr(), ptr::null_mut()) };
            if let Some(callback) = row_fn {
                unsafe { callback(png_ptr, row.as_mut_ptr(), row_num, pass) };
            }

            let after = read_core(png_ptr);
            if after.pass >= 7 || after.row_number >= after.num_rows {
                break;
            }
        }

        let end_emitted = state::get_png(png_ptr)
            .map(|png_state| png_state.progressive_state.end_emitted)
            .unwrap_or(false);
        if !end_emitted {
            unsafe { crate::read::png_read_end(png_ptr, info_ptr) };
            if let Some(callback) = end_fn {
                unsafe { callback(png_ptr, info_ptr) };
            }
            state::update_png(png_ptr, |png_state| {
                png_state.progressive_state.end_emitted = true;
            });
        }
    }

    state::update_png(png_ptr, |png_state| {
        png_state.progressive_state.decoded = true;
        png_state.progressive_state.current_input_start = png_state.progressive_state.decode_offset;
        png_state.progressive_state.current_input_size = 0;
        png_state.read_phase = ReadPhase::Terminal;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_process_data(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    buffer: png_bytep,
    buffer_size: usize,
) {
    crate::abi_guard!(png_ptr, unsafe {
        if png_ptr.is_null() || info_ptr.is_null() {
            return;
        }

        if !buffer.is_null() && buffer_size != 0 {
            let input = core::slice::from_raw_parts(buffer, buffer_size);
            state::update_png(png_ptr, |png_state| {
                png_state.progressive_state.current_input_start =
                    png_state.progressive_state.buffered.len();
                png_state.progressive_state.current_input_size = input.len();
                png_state.progressive_state.buffered.extend_from_slice(input);
                png_state.read_phase = ReadPhase::ChunkPayload;
            });
        } else {
            state::update_png(png_ptr, |png_state| {
                png_state.progressive_state.current_input_start =
                    png_state.progressive_state.buffered.len();
                png_state.progressive_state.current_input_size = 0;
            });
        }

        let buffered = state::get_png(png_ptr)
            .map(|png_state| png_state.progressive_state.buffered.clone())
            .unwrap_or_default();

        match complete_png_length(&buffered) {
            None => {}
            Some(Err(message)) => {
                let _ = call_error(png_ptr, message);
                crate::error::png_longjmp(png_ptr, 1);
            }
            Some(Ok(_)) => {
                if !state::get_png(png_ptr)
                    .map(|png_state| png_state.progressive_state.decoded)
                    .unwrap_or(false)
                {
                    decode_buffered_progressive_png(png_ptr, info_ptr);
                }
            }
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_process_data_pause(
    png_ptr: png_structrp,
    save: core::ffi::c_int,
) -> usize {
    crate::abi_guard!(png_ptr, {
        if state::get_png(png_ptr)
            .map(|png_state| png_state.progressive_state.decoded)
            .unwrap_or(false)
        {
            state::update_png(png_ptr, |png_state| {
                png_state.progressive_state.last_pause_bytes = 0;
                png_state.progressive_state.paused_with_save = save != 0;
            });
            return 0;
        }

        let unread = note_progressive_pause_bytes(png_ptr, save != 0);
        state::update_png(png_ptr, |png_state| {
            if !png_state.progressive_state.decoded {
                png_state.read_phase = ReadPhase::ChunkPayload;
            }
        });
        unread
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_process_data_skip(png_ptr: png_structrp) -> png_uint_32 {
    crate::abi_guard!(png_ptr, {
        let _ = unsafe {
            call_warning(
                png_ptr,
                b"png_process_data_skip is ignored by the Rust progressive reader\0",
            )
        };
        state::update_png(png_ptr, |png_state| {
            png_state.progressive_state.last_skip_bytes = 0;
            if !png_state.progressive_state.decoded {
                png_state.read_phase = ReadPhase::ChunkPayload;
            }
        });
        0
    })
}
