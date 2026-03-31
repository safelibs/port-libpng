use crate::common::PNG_OPTION_INVALID;
use crate::state;
use crate::types::*;

unsafe extern "C" {
    fn upstream_png_set_sig_bytes(png_ptr: png_structrp, num_bytes: core::ffi::c_int);
    fn upstream_png_set_rows(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        row_pointers: png_bytepp,
    );
    fn upstream_png_set_user_limits(
        png_ptr: png_structrp,
        user_width_max: png_uint_32,
        user_height_max: png_uint_32,
    );
    fn upstream_png_set_chunk_cache_max(png_ptr: png_structrp, user_chunk_cache_max: png_uint_32);
    fn upstream_png_set_chunk_malloc_max(
        png_ptr: png_structrp,
        user_chunk_malloc_max: png_alloc_size_t,
    );
    fn upstream_png_set_benign_errors(png_ptr: png_structrp, allowed: core::ffi::c_int);
    fn upstream_png_set_check_for_invalid_index(
        png_ptr: png_structrp,
        allowed: core::ffi::c_int,
    );
    fn upstream_png_set_option(
        png_ptr: png_structrp,
        option: core::ffi::c_int,
        onoff: core::ffi::c_int,
    ) -> core::ffi::c_int;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_sig_bytes(png_ptr: png_structrp, num_bytes: core::ffi::c_int) {
    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_set_sig_bytes(png_ptr, num_bytes);
        state::update_png(png_ptr, |state| {
            state.sig_bytes = num_bytes.clamp(0, 8);
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_rows(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    row_pointers: png_bytepp,
) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_set_rows(png_ptr, info_ptr, row_pointers);
        state::update_info(info_ptr, |state| {
            state.row_pointers = row_pointers;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_user_limits(
    png_ptr: png_structrp,
    user_width_max: png_uint_32,
    user_height_max: png_uint_32,
) {
    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_set_user_limits(png_ptr, user_width_max, user_height_max);
        state::update_png(png_ptr, |state| {
            state.user_width_max = user_width_max;
            state.user_height_max = user_height_max;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_chunk_cache_max(
    png_ptr: png_structrp,
    user_chunk_cache_max: png_uint_32,
) {
    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_set_chunk_cache_max(png_ptr, user_chunk_cache_max);
        state::update_png(png_ptr, |state| {
            state.user_chunk_cache_max = user_chunk_cache_max;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_chunk_malloc_max(
    png_ptr: png_structrp,
    user_chunk_malloc_max: png_alloc_size_t,
) {
    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_set_chunk_malloc_max(png_ptr, user_chunk_malloc_max);
        state::update_png(png_ptr, |state| {
            state.user_chunk_malloc_max = user_chunk_malloc_max;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_benign_errors(png_ptr: png_structrp, allowed: core::ffi::c_int) {
    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_set_benign_errors(png_ptr, allowed);
        state::update_png(png_ptr, |state| {
            state.benign_errors = if allowed != 0 { 1 } else { 0 };
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_check_for_invalid_index(
    png_ptr: png_structrp,
    allowed: core::ffi::c_int,
) {
    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_set_check_for_invalid_index(png_ptr, allowed);
        state::update_png(png_ptr, |state| {
            state.check_for_invalid_index = if allowed > 0 { 1 } else { 0 };
            state.palette_max = if allowed > 0 { 0 } else { -1 };
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_option(
    png_ptr: png_structrp,
    option: core::ffi::c_int,
    onoff: core::ffi::c_int,
) -> core::ffi::c_int {
    crate::abi_guard!(png_ptr, unsafe {
        let result = upstream_png_set_option(png_ptr, option, onoff);
        if result != PNG_OPTION_INVALID {
            state::update_png(png_ptr, |state| {
                let mask = 3u32 << option;
                let setting = (2u32 + u32::from(onoff != 0)) << option;
                state.options = (state.options & !mask) | setting;
            });
        }
        result
    })
}
