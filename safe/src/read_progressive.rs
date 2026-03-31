use crate::chunks::{
    read_core, read_info_core, rollback_info_state, set_read_phase, sync_read_phase_from_core,
    sync_unknown_chunk_policy_to_upstream, write_core,
};
use crate::read_util::ReadPhase;
use crate::state;
use crate::types::*;

unsafe extern "C" {
    fn png_safe_call_push_restore_buffer(
        png_ptr: png_structrp,
        buffer: png_bytep,
        buffer_size: usize,
    ) -> core::ffi::c_int;
    fn png_safe_call_push_read_sig(
        png_ptr: png_structrp,
        info_ptr: png_inforp,
    ) -> core::ffi::c_int;
    fn png_safe_call_push_read_chunk(
        png_ptr: png_structrp,
        info_ptr: png_inforp,
    ) -> core::ffi::c_int;
    fn png_safe_call_push_read_idat(png_ptr: png_structrp) -> core::ffi::c_int;
    fn upstream_png_process_data_pause(png_ptr: png_structrp, save: core::ffi::c_int) -> usize;
    fn upstream_png_process_data_skip(png_ptr: png_structrp) -> png_uint_32;
}

const PNG_READ_SIG_MODE: i32 = 0;
const PNG_READ_CHUNK_MODE: i32 = 1;
const PNG_READ_IDAT_MODE: i32 = 2;

#[derive(Clone)]
struct ProgressiveSnapshot {
    core: png_safe_read_core,
    png_state: Option<state::PngStructState>,
    info_core: png_safe_info_core,
    info_state: Option<state::PngInfoState>,
}

fn snapshot_progressive_state(png_ptr: png_structrp, info_ptr: png_inforp) -> ProgressiveSnapshot {
    ProgressiveSnapshot {
        core: read_core(png_ptr),
        png_state: state::get_png(png_ptr),
        info_core: read_info_core(info_ptr),
        info_state: state::get_info(info_ptr),
    }
}

fn rollback_progressive_state(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ProgressiveSnapshot,
) {
    write_core(png_ptr, &snapshot.core);
    rollback_info_state(info_ptr, &snapshot.info_core);
    if let Some(png_state) = snapshot.png_state.clone() {
        state::register_png(png_ptr, png_state);
    }
    if let Some(info_state) = snapshot.info_state {
        state::register_info(info_ptr, info_state);
    }
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

        sync_unknown_chunk_policy_to_upstream(png_ptr);
        let snapshot = snapshot_progressive_state(png_ptr, info_ptr);
        if png_safe_call_push_restore_buffer(png_ptr, buffer, buffer_size) == 0 {
            rollback_progressive_state(png_ptr, info_ptr, &snapshot);
            crate::error::png_longjmp(png_ptr, 1);
        }

        loop {
            let core = read_core(png_ptr);
            if core.buffer_size == 0 {
                break;
            }

            let status = match core.process_mode {
                PNG_READ_SIG_MODE => png_safe_call_push_read_sig(png_ptr, info_ptr),
                PNG_READ_CHUNK_MODE => png_safe_call_push_read_chunk(png_ptr, info_ptr),
                PNG_READ_IDAT_MODE => png_safe_call_push_read_idat(png_ptr),
                _ => {
                    let mut updated = core;
                    updated.buffer_size = 0;
                    write_core(png_ptr, &updated);
                    1
                }
            };

            if status == 0 {
                rollback_progressive_state(png_ptr, info_ptr, &snapshot);
                crate::error::png_longjmp(png_ptr, 1);
            }
        }

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
