use crate::read_util::{
    PNG_HANDLE_CHUNK_ALWAYS, PNG_HANDLE_CHUNK_AS_DEFAULT, PNG_HANDLE_CHUNK_IF_SAFE,
    PNG_HANDLE_CHUNK_NEVER, ReadPhase, UnknownChunkSetting, ancillary_chunk, checked_chunk_length,
    checked_idat_limit, copy_chunk_name, is_known_chunk_name, known_chunks_to_ignore, safe_to_copy,
};
use crate::state;
use crate::types::*;
use crate::zlib;
use core::ffi::{c_char, c_int};

pub(crate) fn read_core(png_ptr: png_const_structrp) -> png_safe_read_core {
    state::with_png(png_ptr.cast_mut(), |png_state| png_state.core).unwrap_or_default()
}

pub(crate) fn write_core(png_ptr: png_structrp, core: &png_safe_read_core) {
    state::update_png(png_ptr, |png_state| {
        png_state.core = *core;
    });
}

pub(crate) fn read_info_core(info_ptr: png_const_inforp) -> png_safe_info_core {
    state::with_info(info_ptr.cast_mut(), |info_state| info_state.core).unwrap_or_default()
}

pub(crate) fn write_info_core(info_ptr: png_inforp, core: &png_safe_info_core) {
    state::update_info(info_ptr, |info_state| {
        info_state.core = *core;
    });
}

pub(crate) unsafe fn call_warning(png_ptr: png_structrp, message: &[u8]) -> c_int {
    let callback = state::with_png(png_ptr, |png_state| png_state.warning_fn).flatten();
    if let Some(callback) = callback {
        unsafe { callback(png_ptr, message.as_ptr().cast::<c_char>()) };
    }
    1
}

pub(crate) unsafe fn call_benign_error(png_ptr: png_structrp, message: &[u8]) -> c_int {
    let benign = state::with_png(png_ptr, |png_state| png_state.benign_errors != 0).unwrap_or(true);
    if benign {
        unsafe { call_warning(png_ptr, message) }
    } else {
        unsafe { crate::error::png_error(png_ptr, message.as_ptr().cast::<c_char>()) }
    }
}

pub(crate) unsafe fn call_app_error(png_ptr: png_structrp, message: &[u8]) -> c_int {
    unsafe { crate::error::png_error(png_ptr, message.as_ptr().cast::<c_char>()) }
}

pub(crate) unsafe fn call_error(png_ptr: png_structrp, message: &[u8]) -> c_int {
    let callback = state::with_png(png_ptr, |png_state| png_state.error_fn).flatten();
    if let Some(callback) = callback {
        unsafe { callback(png_ptr, message.as_ptr().cast::<c_char>()) };
    }
    1
}

pub(crate) fn set_read_phase(png_ptr: png_structrp, phase: ReadPhase) {
    state::update_png(png_ptr, |state| {
        state.read_phase = phase;
    });
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

pub(crate) fn keep_for_chunk(png_ptr: png_const_structrp, name: [png_byte; 4]) -> c_int {
    if let Some(keep) = explicit_keep_for_chunk(png_ptr, name) {
        return keep;
    }

    if is_known_chunk_name(name) {
        PNG_HANDLE_CHUNK_AS_DEFAULT
    } else {
        state::get_png(png_ptr.cast_mut())
            .map(|state| state.unknown_default_keep)
            .unwrap_or(PNG_HANDLE_CHUNK_AS_DEFAULT)
    }
}

pub(crate) fn dispatch_user_chunk_callback(
    png_ptr: png_structrp,
    chunk: &mut png_unknown_chunk,
) -> Option<c_int> {
    let png_state = state::get_png(png_ptr)?;
    let callback = png_state.read_user_chunk_fn?;
    if callback as usize == crate::io::png_safe_read_user_chunk_trampoline as usize {
        return Some(0);
    }
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
    length: png_uint_32,
    requested_allocation: usize,
) -> Result<(), &'static [u8]> {
    let declared = checked_chunk_length(length).ok_or(b"chunk length overflow\0".as_slice())?;
    let png_state = state::get_png(png_ptr.cast_mut()).ok_or(b"missing png state\0".as_slice())?;

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
    let png_state = state::get_png(png_ptr).ok_or(b"missing png state\0".as_slice())?;
    let mut limit = 0x7fff_ffffusize as png_alloc_size_t;
    if png_state.user_chunk_malloc_max != 0 {
        limit = limit.min(png_state.user_chunk_malloc_max);
    }

    if chunk_name == crate::read_util::PNG_IDAT {
        let idat_limit =
            checked_idat_limit(&read_core(png_ptr)).ok_or(b"chunk length overflow\0".as_slice())?;
        limit = limit.max(idat_limit);
    }

    if (declared as png_alloc_size_t) > limit {
        return Err(b"chunk data is too large\0".as_slice());
    }

    if chunk_is_ancillary(name) {
        validate_ancillary_chunk_limits(png_ptr, length, declared)?;
    }

    Ok(())
}

pub(crate) fn reserve_chunk_cache_slot(
    png_ptr: png_structrp,
    warning_message: &'static [u8],
) -> Result<(), &'static [u8]> {
    let mut outcome = Ok(());

    state::update_png(png_ptr, |png_state| {
        if png_state.user_chunk_cache_max == 0 {
            return;
        }

        if png_state.user_chunk_cache_max == 1 {
            outcome = Err(warning_message);
            return;
        }

        png_state.user_chunk_cache_max -= 1;
        if png_state.user_chunk_cache_max == 1 {
            outcome = Err(warning_message);
        }
    });

    outcome
}

pub(crate) fn keep_requests_storage(keep: c_int, name: [png_byte; 4]) -> bool {
    keep == PNG_HANDLE_CHUNK_ALWAYS
        || (keep == PNG_HANDLE_CHUNK_IF_SAFE && chunk_is_ancillary(name))
}

pub(crate) fn keep_discards_chunk(keep: c_int, name: [png_byte; 4]) -> bool {
    keep == PNG_HANDLE_CHUNK_NEVER
        || (keep == PNG_HANDLE_CHUNK_IF_SAFE && !chunk_safe_to_copy(name))
}

pub(crate) fn ancillary_error_is_fatal(png_ptr: png_structrp) -> bool {
    state::get_png(png_ptr)
        .map(|png_state| png_state.benign_errors == 0)
        .unwrap_or(true)
}
