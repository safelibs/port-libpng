use crate::read_util::{
    PNG_HANDLE_CHUNK_AS_DEFAULT, ReadPhase, UnknownChunkSetting, ancillary_chunk,
    checked_chunk_length, copy_chunk_name, known_chunks_to_ignore, safe_to_copy,
};
use crate::state;
use crate::types::*;
use crate::zlib;
use core::ffi::{c_char, c_int};
use core::mem::MaybeUninit;
use core::ptr;

unsafe extern "C" {
    fn png_safe_read_core_get(png_ptr: png_const_structrp, out: *mut png_safe_read_core);
    fn png_safe_read_core_set(png_ptr: png_structrp, input: *const png_safe_read_core);
    fn png_safe_info_core_get(info_ptr: png_const_inforp, out: *mut png_safe_info_core);
    fn png_safe_info_core_set(info_ptr: png_inforp, input: *const png_safe_info_core);

    fn png_safe_call_warning(png_ptr: png_structrp, message: png_const_charp) -> c_int;
    fn png_safe_call_benign_error(png_ptr: png_structrp, message: png_const_charp) -> c_int;
    fn png_safe_call_app_error(png_ptr: png_structrp, message: png_const_charp) -> c_int;
    fn png_safe_call_error(png_ptr: png_structrp, message: png_const_charp) -> c_int;
    fn upstream_png_set_keep_unknown_chunks(
        png_ptr: png_structrp,
        keep: c_int,
        chunk_list: png_const_bytep,
        num_chunks_in: c_int,
    );
}

pub(crate) fn read_core(png_ptr: png_const_structrp) -> png_safe_read_core {
    let mut out = MaybeUninit::<png_safe_read_core>::zeroed();
    unsafe {
        png_safe_read_core_get(png_ptr, out.as_mut_ptr());
        out.assume_init()
    }
}

pub(crate) fn write_core(png_ptr: png_structrp, core: &png_safe_read_core) {
    unsafe {
        png_safe_read_core_set(png_ptr, core);
    }
}

pub(crate) fn read_info_core(info_ptr: png_const_inforp) -> png_safe_info_core {
    let mut out = MaybeUninit::<png_safe_info_core>::zeroed();
    unsafe {
        png_safe_info_core_get(info_ptr, out.as_mut_ptr());
        out.assume_init()
    }
}

pub(crate) fn write_info_core(info_ptr: png_inforp, core: &png_safe_info_core) {
    unsafe {
        png_safe_info_core_set(info_ptr, core);
    }
}

pub(crate) unsafe fn call_warning(png_ptr: png_structrp, message: &[u8]) -> c_int {
    unsafe { png_safe_call_warning(png_ptr, message.as_ptr().cast::<c_char>()) }
}

pub(crate) unsafe fn call_benign_error(png_ptr: png_structrp, message: &[u8]) -> c_int {
    unsafe { png_safe_call_benign_error(png_ptr, message.as_ptr().cast::<c_char>()) }
}

pub(crate) unsafe fn call_app_error(png_ptr: png_structrp, message: &[u8]) -> c_int {
    unsafe { png_safe_call_app_error(png_ptr, message.as_ptr().cast::<c_char>()) }
}

pub(crate) unsafe fn call_error(png_ptr: png_structrp, message: &[u8]) -> c_int {
    unsafe { png_safe_call_error(png_ptr, message.as_ptr().cast::<c_char>()) }
}

pub(crate) fn sync_read_phase_from_core(png_ptr: png_structrp) {
    let core = read_core(png_ptr);
    let phase = ReadPhase::from_core(&core);
    state::update_png(png_ptr, |state| {
        state.read_phase = phase;
    });
}

pub(crate) fn set_read_phase(png_ptr: png_structrp, phase: ReadPhase) {
    state::update_png(png_ptr, |state| {
        state.read_phase = phase;
    });
}

pub(crate) fn rollback_info_state(info_ptr: png_inforp, snapshot: &png_safe_info_core) {
    write_info_core(info_ptr, snapshot);
}

fn apply_unknown_chunk_setting(
    list: &mut Vec<UnknownChunkSetting>,
    name: [png_byte; 4],
    keep: c_int,
) {
    if let Some(entry) = list.iter_mut().find(|entry| entry.name == name) {
        entry.keep = keep as png_byte;
    } else if keep != PNG_HANDLE_CHUNK_AS_DEFAULT {
        list.push(UnknownChunkSetting::new(name, keep as png_byte));
    }
}

pub(crate) fn apply_keep_unknown_chunks_state(
    png_ptr: png_structrp,
    keep: c_int,
    chunk_list: png_const_bytep,
    num_chunks_in: c_int,
) {
    state::update_png(png_ptr, |png_state| {
        if num_chunks_in <= 0 {
            png_state.unknown_default_keep = keep;
            if num_chunks_in == 0 {
                return;
            }
        }

        let names: Vec<[png_byte; 4]> = if num_chunks_in < 0 {
            known_chunks_to_ignore().to_vec()
        } else if let Ok(num_chunks) = usize::try_from(num_chunks_in) {
            (0..num_chunks)
                .filter_map(|index| copy_chunk_name(chunk_list, index))
                .collect()
        } else {
            Vec::new()
        };

        for name in names {
            apply_unknown_chunk_setting(&mut png_state.unknown_chunk_list, name, keep);
        }

        png_state
            .unknown_chunk_list
            .retain(|entry| c_int::from(entry.keep) != PNG_HANDLE_CHUNK_AS_DEFAULT);
    });
}

pub(crate) fn explicit_keep_for_chunk(
    png_ptr: png_const_structrp,
    name: [png_byte; 4],
) -> Option<c_int> {
    state::get_png(png_ptr.cast_mut())?
        .unknown_chunk_list
        .iter()
        .find(|entry| entry.name == name)
        .map(|entry| c_int::from(entry.keep))
}

pub(crate) fn effective_keep_for_chunk(png_ptr: png_const_structrp, name: [png_byte; 4]) -> c_int {
    explicit_keep_for_chunk(png_ptr, name).unwrap_or_else(|| {
        state::get_png(png_ptr.cast_mut())
            .map(|state| state.unknown_default_keep)
            .unwrap_or(PNG_HANDLE_CHUNK_AS_DEFAULT)
    })
}

pub(crate) fn dispatch_user_chunk_callback(
    png_ptr: png_structrp,
    chunk: &mut png_unknown_chunk,
) -> Option<c_int> {
    let png_state = state::get_png(png_ptr)?;
    let callback = png_state.read_user_chunk_fn?;
    Some(unsafe { callback(png_ptr, chunk as *mut png_unknown_chunk) })
}

pub(crate) fn chunk_safe_to_copy(name: [png_byte; 4]) -> bool {
    safe_to_copy(name)
}

pub(crate) fn chunk_is_ancillary(name: [png_byte; 4]) -> bool {
    ancillary_chunk(name)
}

pub(crate) fn validate_chunk_length(length: png_uint_32) -> Option<usize> {
    checked_chunk_length(length)
}

pub(crate) fn validate_ancillary_chunk_limits(
    png_ptr: png_const_structrp,
    chunk_name: [png_byte; 4],
    length: png_uint_32,
    cached_chunks: usize,
    requested_allocation: usize,
) -> Result<(), &'static [u8]> {
    let _ = chunk_name;
    let declared = checked_chunk_length(length).ok_or(b"chunk length overflow\0".as_slice())?;
    let png_state = state::get_png(png_ptr.cast_mut()).ok_or(b"missing png state\0".as_slice())?;

    if png_state.user_chunk_cache_max != 0
        && cached_chunks >= usize::try_from(png_state.user_chunk_cache_max).unwrap_or(usize::MAX)
    {
        return Err(b"no space in chunk cache\0".as_slice());
    }

    zlib::validate_ancillary_allocation_limit(
        declared,
        requested_allocation,
        png_state.user_chunk_malloc_max,
    )
}

pub(crate) fn chunk_name_bytes(chunk_name: png_uint_32) -> [png_byte; 4] {
    [
        ((chunk_name >> 24) & 0xff) as png_byte,
        ((chunk_name >> 16) & 0xff) as png_byte,
        ((chunk_name >> 8) & 0xff) as png_byte,
        (chunk_name & 0xff) as png_byte,
    ]
}

pub(crate) fn validate_parser_chunk(
    png_ptr: png_structrp,
    chunk_name: png_uint_32,
    length: png_uint_32,
) -> Result<(), &'static [u8]> {
    let declared = validate_chunk_length(length).ok_or(b"chunk length overflow\0".as_slice())?;
    let name = chunk_name_bytes(chunk_name);

    if chunk_is_ancillary(name) {
        let cached_chunks = read_core(png_ptr)
            .num_chunk_list
            .try_into()
            .unwrap_or(usize::MAX);
        validate_ancillary_chunk_limits(png_ptr, name, length, cached_chunks, declared)?;
    }

    Ok(())
}

pub(crate) fn sync_unknown_chunk_policy_to_upstream(png_ptr: png_structrp) {
    let Some(png_state) = state::get_png(png_ptr) else {
        return;
    };

    unsafe {
        upstream_png_set_keep_unknown_chunks(png_ptr, png_state.unknown_default_keep, ptr::null(), 0);
    }

    let policy_bytes = png_state
        .unknown_chunk_list
        .len()
        .checked_mul(5)
        .unwrap_or(usize::MAX);
    let Some(length) = validate_chunk_length(policy_bytes.try_into().unwrap_or(u32::MAX)) else {
        unsafe {
            let _ = call_app_error(png_ptr, b"png_set_keep_unknown_chunks: too many chunks\0");
        }
        return;
    };

    if let Err(message) = validate_ancillary_chunk_limits(
        png_ptr,
        *b"uNkN",
        length.try_into().unwrap_or(u32::MAX),
        0,
        policy_bytes,
    ) {
        unsafe {
            let _ = call_app_error(png_ptr, message);
        }
        return;
    }

    for entry in &png_state.unknown_chunk_list {
        let keep = effective_keep_for_chunk(png_ptr, entry.name);
        let _ancillary = chunk_is_ancillary(entry.name);
        let _safe_to_copy = chunk_safe_to_copy(entry.name);
        let chunk_name = [entry.name[0], entry.name[1], entry.name[2], entry.name[3], 0];
        unsafe {
            upstream_png_set_keep_unknown_chunks(png_ptr, keep, chunk_name.as_ptr(), 1);
        }
    }
}
