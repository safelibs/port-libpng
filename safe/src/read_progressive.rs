use crate::chunks::{read_core, read_info_core};
use crate::common::PNG_FLAG_ROW_INIT;
use crate::state;
use crate::types::*;
use core::ffi::c_int;
use core::ptr;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

unsafe extern "C" {
    fn bridge_png_process_data_pause(png_ptr: png_structrp, save: c_int) -> usize;
    fn bridge_png_process_data_skip(png_ptr: png_structrp) -> png_uint_32;
    fn bridge_png_set_read_fn(png_ptr: png_structrp, io_ptr: png_voidp, read_data_fn: png_rw_ptr);
    fn png_safe_call_read_row(
        png_ptr: png_structrp,
        row: png_bytep,
        display_row: png_bytep,
    ) -> c_int;
    fn png_safe_resume_finish_idat(png_ptr: png_structrp);
    fn png_safe_progressive_buffer_read_bridge(png_ptr: png_structp, out: png_bytep, length: usize);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_rust_progressive_buffer_read(
    png_ptr: png_structp,
    out: png_bytep,
    length: usize,
) -> c_int {
    if png_ptr.is_null() {
        return 0;
    }

    let mut short_read = false;

    state::update_png(png_ptr, |png_state| {
        let progressive = &mut png_state.progressive_state;
        let end = progressive.decode_offset.saturating_add(length);
        if end > progressive.buffered.len() {
            progressive.short_read = true;
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
        return 0;
    }

    1
}

fn unread_bytes_from_last_call(progressive: &crate::read_util::ProgressiveReadState) -> usize {
    let start = progressive.current_input_start;
    let end = start.saturating_add(progressive.current_input_size);
    let consumed = progressive.decode_offset.max(start);
    end.saturating_sub(consumed)
}

fn clear_short_read(png_ptr: png_structrp) {
    state::update_png(png_ptr, |png_state| {
        png_state.progressive_state.short_read = false;
    });
}

fn take_short_read(png_ptr: png_structrp) -> bool {
    let mut short_read = false;
    state::update_png(png_ptr, |png_state| {
        short_read = png_state.progressive_state.short_read;
        png_state.progressive_state.short_read = false;
    });
    short_read
}

fn take_pause_request(png_ptr: png_structrp) -> bool {
    let mut pause_requested = false;
    state::update_png(png_ptr, |png_state| {
        pause_requested = png_state.progressive_state.pause_requested;
        png_state.progressive_state.pause_requested = false;
    });
    pause_requested
}

fn note_progressive_pause_bytes(png_ptr: png_structrp, save: bool) -> usize {
    let mut unread = 0;

    state::update_png(png_ptr, |png_state| {
        let progressive = &mut png_state.progressive_state;
        unread = unread_bytes_from_last_call(progressive);
        progressive.last_pause_bytes = unread;
        progressive.paused_with_save = save;
        progressive.pause_requested = true;

        if !save {
            progressive.buffered.clear();
            progressive.decode_offset = 0;
            progressive.current_input_start = 0;
            progressive.current_input_size = 0;
        }
    });

    unread
}

fn progressive_rowbytes(png_ptr: png_structrp, info_ptr: png_inforp) -> usize {
    let core = read_core(png_ptr);
    if core.rowbytes != 0 {
        core.rowbytes
    } else {
        read_info_core(info_ptr).rowbytes
    }
}

fn compact_progressive_buffer(png_ptr: png_structrp) {
    state::update_png(png_ptr, |png_state| {
        let progressive = &mut png_state.progressive_state;
        let drain = progressive.decode_offset;
        if drain == 0 {
            return;
        }
        if drain < 4096 && drain < progressive.buffered.len() / 2 {
            return;
        }

        progressive.buffered.drain(..drain);
        progressive.decode_offset = 0;
        progressive.current_input_start = progressive.current_input_start.saturating_sub(drain);
    });
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RowReadOutcome {
    Completed,
    SuspendedBeforeRow,
    CompletedThenSuspended,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ProgressMarker {
    decode_offset: usize,
    phase: crate::read_util::ReadPhase,
    info_emitted: bool,
    end_emitted: bool,
    decoded: bool,
    row_number: png_uint_32,
    pass: c_int,
}

fn progress_marker(png_ptr: png_structrp) -> ProgressMarker {
    let core = read_core(png_ptr);
    state::get_png(png_ptr)
        .map(|png_state| ProgressMarker {
            decode_offset: png_state.progressive_state.decode_offset,
            phase: png_state.read_phase,
            info_emitted: png_state.progressive_state.info_emitted,
            end_emitted: png_state.progressive_state.end_emitted,
            decoded: png_state.progressive_state.decoded,
            row_number: core.row_number,
            pass: core.pass,
        })
        .unwrap_or(ProgressMarker {
            row_number: core.row_number,
            pass: core.pass,
            ..ProgressMarker::default()
        })
}

unsafe fn emit_info_callback(png_ptr: png_structrp, info_ptr: png_inforp) -> bool {
    let already_emitted = state::get_png(png_ptr)
        .map(|png_state| png_state.progressive_state.info_emitted)
        .unwrap_or(false);
    if already_emitted {
        return false;
    }

    if let Some((_, info_fn, _, _)) = crate::io::progressive_read_registration(png_ptr) {
        if let Some(callback) = info_fn {
            unsafe { callback(png_ptr, info_ptr) };
        }
    }

    state::update_png(png_ptr, |png_state| {
        png_state.progressive_state.info_emitted = true;
    });
    take_pause_request(png_ptr)
}

unsafe fn emit_row_callback(
    png_ptr: png_structrp,
    row: &mut [u8],
    row_num: png_uint_32,
    pass: c_int,
) -> bool {
    if let Some((_, _, row_fn, _)) = crate::io::progressive_read_registration(png_ptr) {
        if let Some(callback) = row_fn {
            unsafe { callback(png_ptr, row.as_mut_ptr(), row_num, pass) };
        }
    }

    take_pause_request(png_ptr)
}

unsafe fn emit_end_callback(png_ptr: png_structrp, info_ptr: png_inforp) -> bool {
    let already_emitted = state::get_png(png_ptr)
        .map(|png_state| png_state.progressive_state.end_emitted)
        .unwrap_or(false);
    if already_emitted {
        return false;
    }

    if let Some((_, _, _, end_fn)) = crate::io::progressive_read_registration(png_ptr) {
        if let Some(callback) = end_fn {
            unsafe { callback(png_ptr, info_ptr) };
        }
    }

    state::update_png(png_ptr, |png_state| {
        png_state.progressive_state.end_emitted = true;
        png_state.progressive_state.decoded = true;
        png_state.progressive_state.current_input_start = png_state.progressive_state.decode_offset;
        png_state.progressive_state.current_input_size = 0;
    });
    take_pause_request(png_ptr)
}

unsafe fn call_read_impl_or_suspend(png_ptr: png_structrp, call: impl FnOnce()) -> Result<(), ()> {
    clear_short_read(png_ptr);
    match catch_unwind(AssertUnwindSafe(call)) {
        Ok(()) => Ok(()),
        Err(payload) => {
            if payload.is::<crate::read::ProgressiveSuspend>() {
                return Err(());
            }

            resume_unwind(payload)
        }
    }
}

unsafe fn read_row_or_suspend(
    png_ptr: png_structrp,
    row: png_bytep,
    display_row: png_bytep,
) -> RowReadOutcome {
    let core_before = read_core(png_ptr);
    let snapshot = unsafe { crate::read::snapshot_parse_state(png_ptr, ptr::null_mut()) };
    clear_short_read(png_ptr);
    if unsafe { png_safe_call_read_row(png_ptr, row, display_row) } != 0 {
        unsafe { crate::read::free_parse_snapshot(&snapshot) };
        return RowReadOutcome::Completed;
    }

    if take_short_read(png_ptr) {
        let core_after = read_core(png_ptr);
        let row_completed =
            core_after.row_number != core_before.row_number || core_after.pass != core_before.pass;
        if row_completed {
            unsafe { png_safe_resume_finish_idat(png_ptr) };
        }
        unsafe { crate::read::free_parse_snapshot(&snapshot) };
        return if row_completed {
            RowReadOutcome::CompletedThenSuspended
        } else {
            RowReadOutcome::SuspendedBeforeRow
        };
    }

    unsafe { crate::read::free_parse_snapshot(&snapshot) };
    crate::read::raise_read_longjmp()
}

unsafe fn drive_progressive_decode(png_ptr: png_structrp, info_ptr: png_inforp) {
    unsafe {
        bridge_png_set_read_fn(
            png_ptr,
            ptr::null_mut(),
            Some(png_safe_progressive_buffer_read_bridge),
        );
    }

    loop {
        let before = progress_marker(png_ptr);
        if state::get_png(png_ptr)
            .map(|png_state| png_state.progressive_state.decoded)
            .unwrap_or(false)
        {
            break;
        }

        let info_emitted = state::get_png(png_ptr)
            .map(|png_state| png_state.progressive_state.info_emitted)
            .unwrap_or(false);
        if !info_emitted {
            if unsafe {
                call_read_impl_or_suspend(png_ptr, || {
                    crate::read::read_info_impl(png_ptr, info_ptr)
                })
            }
            .is_err()
            {
                break;
            }

            if unsafe { emit_info_callback(png_ptr, info_ptr) } {
                break;
            }

            if matches!(
                state::get_png(png_ptr).map(|png_state| png_state.read_phase),
                Some(crate::read_util::ReadPhase::Terminal)
            ) {
                let _ = unsafe { emit_end_callback(png_ptr, info_ptr) };
                break;
            }

            if progress_marker(png_ptr) == before {
                break;
            }
            continue;
        }

        let core = read_core(png_ptr);
        if (core.flags & PNG_FLAG_ROW_INIT) == 0 {
            break;
        }

        if core.pass < 7 && core.row_number < core.num_rows {
            let rowbytes = progressive_rowbytes(png_ptr, info_ptr);
            if rowbytes == 0 {
                break;
            }

            let mut row = vec![0; rowbytes];
            let row_outcome =
                unsafe { read_row_or_suspend(png_ptr, row.as_mut_ptr(), ptr::null_mut()) };
            if row_outcome == RowReadOutcome::SuspendedBeforeRow {
                break;
            }

            let core_after = read_core(png_ptr);
            let row_num = core.row_number;
            let pass = core.pass;

            crate::interlace::sanitize_row_padding(png_ptr, row.as_mut_ptr(), ptr::null_mut());

            let mut callback_core = core_after;
            callback_core.row_number = row_num;
            callback_core.pass = pass;
            crate::chunks::write_core(png_ptr, &callback_core);
            let pause_requested = unsafe { emit_row_callback(png_ptr, &mut row, row_num, pass) };
            crate::chunks::write_core(png_ptr, &core_after);

            if pause_requested {
                break;
            }

            if row_outcome == RowReadOutcome::CompletedThenSuspended {
                break;
            }

            if progress_marker(png_ptr) == before {
                break;
            }
            continue;
        }

        if unsafe {
            call_read_impl_or_suspend(png_ptr, || crate::read::read_end_impl(png_ptr, info_ptr))
        }
        .is_err()
        {
            break;
        }

        let _ = unsafe { emit_end_callback(png_ptr, info_ptr) };
        break;
    }

    compact_progressive_buffer(png_ptr);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_rust_process_data(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    buffer: png_bytep,
    buffer_size: usize,
) -> c_int {
    if png_ptr.is_null() || info_ptr.is_null() {
        return 1;
    }

    crate::read::catch_read_status(|| unsafe {
        if !buffer.is_null() && buffer_size != 0 {
            let input = core::slice::from_raw_parts(buffer, buffer_size);
            state::update_png(png_ptr, |png_state| {
                png_state.progressive_state.current_input_start =
                    png_state.progressive_state.buffered.len();
                png_state.progressive_state.current_input_size = input.len();
                png_state
                    .progressive_state
                    .buffered
                    .extend_from_slice(input);
            });
        } else {
            state::update_png(png_ptr, |png_state| {
                png_state.progressive_state.current_input_start =
                    png_state.progressive_state.buffered.len();
                png_state.progressive_state.current_input_size = 0;
            });
        }

        drive_progressive_decode(png_ptr, info_ptr);
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_process_data_pause(
    png_ptr: png_structrp,
    save: core::ffi::c_int,
) -> usize {
    crate::abi_guard!(png_ptr, unsafe {
        bridge_png_process_data_pause(png_ptr, save)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_process_data_skip(png_ptr: png_structrp) -> png_uint_32 {
    crate::abi_guard!(png_ptr, unsafe { bridge_png_process_data_skip(png_ptr) })
}
