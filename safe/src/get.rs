use crate::common::PNG_INFO_tRNS;
use crate::state::{info_ptr_state_const, png_ptr_state_const};
use crate::types::*;
use core::ptr;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_valid(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
    flag: png_uint_32,
) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        let Some(state) = png_ptr_state_const(png_ptr) else {
            return 0;
        };
        let Some(info) = info_ptr_state_const(info_ptr) else {
            return 0;
        };

        if flag == PNG_INFO_tRNS && state.num_trans == 0 {
            0
        } else {
            info.valid & flag
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_rowbytes(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> usize {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() || info_ptr.is_null() {
            return 0;
        }
        info_ptr_state_const(info_ptr)
            .map(|info| info.rowbytes)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_rows(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_bytepp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() || info_ptr.is_null() {
            return ptr::null_mut();
        }
        info_ptr_state_const(info_ptr)
            .map(|info| info.row_pointers)
            .unwrap_or(ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_image_width(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() || info_ptr.is_null() {
            return 0;
        }
        info_ptr_state_const(info_ptr)
            .map(|info| info.width)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_image_height(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() || info_ptr.is_null() {
            return 0;
        }
        info_ptr_state_const(info_ptr)
            .map(|info| info.height)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_bit_depth(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_byte {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() || info_ptr.is_null() {
            return 0;
        }
        info_ptr_state_const(info_ptr)
            .map(|info| info.bit_depth)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_color_type(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_byte {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() || info_ptr.is_null() {
            return 0;
        }
        info_ptr_state_const(info_ptr)
            .map(|info| info.color_type)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_filter_type(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_byte {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() || info_ptr.is_null() {
            return 0;
        }
        info_ptr_state_const(info_ptr)
            .map(|info| info.filter_type)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_interlace_type(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_byte {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() || info_ptr.is_null() {
            return 0;
        }
        info_ptr_state_const(info_ptr)
            .map(|info| info.interlace_type)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_compression_type(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_byte {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() || info_ptr.is_null() {
            return 0;
        }
        info_ptr_state_const(info_ptr)
            .map(|info| info.compression_type)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_channels(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_byte {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() || info_ptr.is_null() {
            return 0;
        }
        info_ptr_state_const(info_ptr)
            .map(|info| info.channels)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_user_width_max(png_ptr: png_const_structrp) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        png_ptr_state_const(png_ptr)
            .map(|state| state.user_width_max)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_user_height_max(png_ptr: png_const_structrp) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        png_ptr_state_const(png_ptr)
            .map(|state| state.user_height_max)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_chunk_cache_max(png_ptr: png_const_structrp) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        png_ptr_state_const(png_ptr)
            .map(|state| state.user_chunk_cache_max)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_chunk_malloc_max(png_ptr: png_const_structrp) -> png_alloc_size_t {
    crate::abi_guard!(png_ptr.cast_mut(), {
        png_ptr_state_const(png_ptr)
            .map(|state| state.user_chunk_malloc_max)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_palette_max(
    png_ptr: png_const_structp,
    info_ptr: png_const_infop,
) -> core::ffi::c_int {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() || info_ptr.is_null() {
            return 0;
        }
        png_ptr_state_const(png_ptr)
            .map(|state| state.num_palette_max)
            .unwrap_or(0)
    })
}
