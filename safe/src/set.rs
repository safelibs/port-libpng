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
    crate::abi_guard!(png_ptr, unsafe { upstream_png_set_sig_bytes(png_ptr, num_bytes) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_rows(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    row_pointers: png_bytepp,
) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_set_rows(png_ptr, info_ptr, row_pointers)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_user_limits(
    png_ptr: png_structrp,
    user_width_max: png_uint_32,
    user_height_max: png_uint_32,
) {
    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_set_user_limits(png_ptr, user_width_max, user_height_max)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_chunk_cache_max(
    png_ptr: png_structrp,
    user_chunk_cache_max: png_uint_32,
) {
    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_set_chunk_cache_max(png_ptr, user_chunk_cache_max)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_chunk_malloc_max(
    png_ptr: png_structrp,
    user_chunk_malloc_max: png_alloc_size_t,
) {
    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_set_chunk_malloc_max(png_ptr, user_chunk_malloc_max)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_benign_errors(png_ptr: png_structrp, allowed: core::ffi::c_int) {
    crate::abi_guard!(png_ptr, unsafe { upstream_png_set_benign_errors(png_ptr, allowed) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_check_for_invalid_index(
    png_ptr: png_structrp,
    allowed: core::ffi::c_int,
) {
    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_set_check_for_invalid_index(png_ptr, allowed)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_option(
    png_ptr: png_structrp,
    option: core::ffi::c_int,
    onoff: core::ffi::c_int,
) -> core::ffi::c_int {
    crate::abi_guard!(png_ptr, unsafe { upstream_png_set_option(png_ptr, option, onoff) })
}
