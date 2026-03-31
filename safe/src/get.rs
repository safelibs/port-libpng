use crate::types::*;

unsafe extern "C" {
    fn upstream_png_get_valid(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        flag: png_uint_32,
    ) -> png_uint_32;
    fn upstream_png_get_rowbytes(png_ptr: png_const_structrp, info_ptr: png_const_inforp) -> usize;
    fn upstream_png_get_rows(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_bytepp;
    fn upstream_png_get_image_width(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_uint_32;
    fn upstream_png_get_image_height(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_uint_32;
    fn upstream_png_get_bit_depth(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_byte;
    fn upstream_png_get_color_type(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_byte;
    fn upstream_png_get_filter_type(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_byte;
    fn upstream_png_get_interlace_type(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_byte;
    fn upstream_png_get_compression_type(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_byte;
    fn upstream_png_get_channels(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_byte;
    fn upstream_png_get_user_width_max(png_ptr: png_const_structrp) -> png_uint_32;
    fn upstream_png_get_user_height_max(png_ptr: png_const_structrp) -> png_uint_32;
    fn upstream_png_get_chunk_cache_max(png_ptr: png_const_structrp) -> png_uint_32;
    fn upstream_png_get_chunk_malloc_max(png_ptr: png_const_structrp) -> png_alloc_size_t;
    fn upstream_png_get_palette_max(
        png_ptr: png_const_structp,
        info_ptr: png_const_infop,
    ) -> core::ffi::c_int;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_valid(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
    flag: png_uint_32,
) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_get_valid(png_ptr, info_ptr, flag)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_rowbytes(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> usize {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_get_rowbytes(png_ptr, info_ptr)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_rows(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_bytepp {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe { upstream_png_get_rows(png_ptr, info_ptr) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_image_width(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_get_image_width(png_ptr, info_ptr)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_image_height(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_get_image_height(png_ptr, info_ptr)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_bit_depth(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_byte {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_get_bit_depth(png_ptr, info_ptr)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_color_type(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_byte {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_get_color_type(png_ptr, info_ptr)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_filter_type(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_byte {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_get_filter_type(png_ptr, info_ptr)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_interlace_type(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_byte {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_get_interlace_type(png_ptr, info_ptr)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_compression_type(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_byte {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_get_compression_type(png_ptr, info_ptr)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_channels(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_byte {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_get_channels(png_ptr, info_ptr)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_user_width_max(png_ptr: png_const_structrp) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe { upstream_png_get_user_width_max(png_ptr) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_user_height_max(png_ptr: png_const_structrp) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe { upstream_png_get_user_height_max(png_ptr) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_chunk_cache_max(png_ptr: png_const_structrp) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_get_chunk_cache_max(png_ptr)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_chunk_malloc_max(png_ptr: png_const_structrp) -> png_alloc_size_t {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_get_chunk_malloc_max(png_ptr)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_palette_max(
    png_ptr: png_const_structp,
    info_ptr: png_const_infop,
) -> core::ffi::c_int {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_get_palette_max(png_ptr, info_ptr)
    })
}
