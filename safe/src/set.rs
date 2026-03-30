use crate::common::{
    PNG_FLAG_APP_ERRORS_WARN, PNG_FLAG_APP_WARNINGS_WARN, PNG_FLAG_BENIGN_ERRORS_WARN,
    PNG_OPTION_INVALID, PNG_OPTION_NEXT, PNG_USER_TRANSFORM,
};
use crate::state::{info_ptr_state, png_ptr_state};
use crate::types::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_sig_bytes(png_ptr: png_structrp, num_bytes: core::ffi::c_int) {
    crate::abi_guard!(png_ptr, {
        let Some(state) = png_ptr_state(png_ptr) else {
            return;
        };

        let mut bytes = num_bytes;
        if bytes < 0 {
            bytes = 0;
        }
        if bytes > 8 {
            crate::error::png_error(
                png_ptr,
                b"Too many bytes for PNG signature\0".as_ptr().cast(),
            );
        }

        state.sig_bytes = bytes as png_byte;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_rows(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    row_pointers: png_bytepp,
) {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() || info_ptr.is_null() {
            return;
        }

        if let Some(info) = info_ptr_state(info_ptr) {
            info.row_pointers = row_pointers;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_user_limits(
    png_ptr: png_structrp,
    user_width_max: png_uint_32,
    user_height_max: png_uint_32,
) {
    crate::abi_guard!(png_ptr, {
        if let Some(state) = png_ptr_state(png_ptr) {
            state.user_width_max = user_width_max;
            state.user_height_max = user_height_max;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_chunk_cache_max(
    png_ptr: png_structrp,
    user_chunk_cache_max: png_uint_32,
) {
    crate::abi_guard!(png_ptr, {
        if let Some(state) = png_ptr_state(png_ptr) {
            state.user_chunk_cache_max = user_chunk_cache_max;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_chunk_malloc_max(
    png_ptr: png_structrp,
    user_chunk_malloc_max: png_alloc_size_t,
) {
    crate::abi_guard!(png_ptr, {
        if let Some(state) = png_ptr_state(png_ptr) {
            state.user_chunk_malloc_max = user_chunk_malloc_max;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_benign_errors(png_ptr: png_structrp, allowed: core::ffi::c_int) {
    crate::abi_guard!(png_ptr, {
        let Some(state) = png_ptr_state(png_ptr) else {
            return;
        };

        if allowed != 0 {
            state.flags |=
                PNG_FLAG_BENIGN_ERRORS_WARN | PNG_FLAG_APP_WARNINGS_WARN | PNG_FLAG_APP_ERRORS_WARN;
        } else {
            state.flags &= !(PNG_FLAG_BENIGN_ERRORS_WARN
                | PNG_FLAG_APP_WARNINGS_WARN
                | PNG_FLAG_APP_ERRORS_WARN);
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_check_for_invalid_index(
    png_ptr: png_structrp,
    allowed: core::ffi::c_int,
) {
    crate::abi_guard!(png_ptr, {
        let Some(state) = png_ptr_state(png_ptr) else {
            return;
        };

        if allowed > 0 {
            state.num_palette_max = 0;
        } else {
            state.num_palette_max = -1;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_option(
    png_ptr: png_structrp,
    option: core::ffi::c_int,
    onoff: core::ffi::c_int,
) -> core::ffi::c_int {
    crate::abi_guard!(png_ptr, {
        let Some(state) = png_ptr_state(png_ptr) else {
            return PNG_OPTION_INVALID;
        };

        if option >= 0 && option < PNG_OPTION_NEXT && (option & 1) == 0 {
            let mask = 3u32 << option;
            let setting = (2u32 + u32::from(onoff != 0)) << option;
            let current = state.options;
            state.options = (current & !mask) | setting;
            ((current & mask) >> option) as core::ffi::c_int
        } else {
            PNG_OPTION_INVALID
        }
    })
}
