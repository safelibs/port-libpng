use crate::chunks::{set_read_phase, sync_read_phase_from_core};
use crate::read_util::ReadPhase;
use crate::state;
use crate::types::*;

unsafe extern "C" {
    fn upstream_png_process_data(
        png_ptr: png_structrp,
        info_ptr: png_inforp,
        buffer: png_bytep,
        buffer_size: usize,
    );
    fn upstream_png_process_data_pause(png_ptr: png_structrp, save: core::ffi::c_int) -> usize;
    fn upstream_png_process_data_skip(png_ptr: png_structrp) -> png_uint_32;
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

        upstream_png_process_data(png_ptr, info_ptr, buffer, buffer_size);
        sync_read_phase_from_core(png_ptr);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_process_data_pause(
    png_ptr: png_structrp,
    save: core::ffi::c_int,
) -> usize {
    crate::abi_guard!(png_ptr, unsafe {
        let remaining = upstream_png_process_data_pause(png_ptr, save);
        state::update_png(png_ptr, |state| {
            state.progressive_state.last_pause_bytes = remaining;
            state.progressive_state.paused_with_save = save != 0;
            if remaining != 0 || save != 0 {
                state.read_phase = ReadPhase::ChunkPayload;
            }
        });
        remaining
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_process_data_skip(png_ptr: png_structrp) -> png_uint_32 {
    crate::abi_guard!(png_ptr, unsafe {
        let skip = upstream_png_process_data_skip(png_ptr);
        state::update_png(png_ptr, |state| {
            state.progressive_state.last_skip_bytes = skip;
        });
        if skip == 0 {
            set_read_phase(png_ptr, ReadPhase::ChunkPayload);
        }
        skip
    })
}
