use crate::chunks::{call_app_error, read_core, write_core};
use crate::interlace::mask_packed_row_padding;
use crate::types::*;
use core::ffi::c_int;

const PNG_HAVE_IHDR: png_uint_32 = 0x01;
const PNG_INVERT_MONO: png_uint_32 = 0x0020;
const PNG_EXPAND: png_uint_32 = 0x1000;
const PNG_GRAY_TO_RGB: png_uint_32 = 0x4000;
const PNG_SWAP_ALPHA: png_uint_32 = 0x20000;
const PNG_INVERT_ALPHA: png_uint_32 = 0x80000;
const PNG_EXPAND_16: png_uint_32 = 0x0200;
const PNG_16_TO_8: png_uint_32 = 0x0400;
const PNG_EXPAND_tRNS: png_uint_32 = 0x2000000;
const PNG_SCALE_16_TO_8: png_uint_32 = 0x4000000;
const PNG_BGR: png_uint_32 = 0x0001;

const PNG_FLAG_ROW_INIT: png_uint_32 = 0x0040;
const PNG_FLAG_DETECT_UNINITIALIZED: png_uint_32 = 0x4000;

unsafe extern "C" {
    fn png_safe_call_read_row(png_ptr: png_structrp, row: png_bytep, display_row: png_bytep)
    -> c_int;
}

fn rtran_ok(png_ptr: png_structrp, need_ihdr: bool) -> bool {
    if png_ptr.is_null() {
        return false;
    }

    let mut core = unsafe { read_core(png_ptr) };
    if (core.flags & PNG_FLAG_ROW_INIT) != 0 {
        unsafe {
            let _ = call_app_error(
                png_ptr,
                b"invalid after png_start_read_image or png_read_update_info\0",
            );
        }
        return false;
    }

    if need_ihdr && (core.mode & PNG_HAVE_IHDR) == 0 {
        unsafe {
            let _ = call_app_error(
                png_ptr,
                b"invalid before the PNG header has been read\0",
            );
        }
        return false;
    }

    core.flags |= PNG_FLAG_DETECT_UNINITIALIZED;
    unsafe {
        write_core(png_ptr, &core);
    }
    true
}

fn update_transform(png_ptr: png_structrp, transform_mask: png_uint_32) {
    if !rtran_ok(png_ptr, false) {
        return;
    }

    let mut core = unsafe { read_core(png_ptr) };
    core.transformations |= transform_mask;
    unsafe {
        write_core(png_ptr, &core);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_expand(png_ptr: png_structrp) {
    update_transform(png_ptr, PNG_EXPAND | PNG_EXPAND_tRNS);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_expand_16(png_ptr: png_structrp) {
    update_transform(png_ptr, PNG_EXPAND_16 | PNG_EXPAND | PNG_EXPAND_tRNS);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_palette_to_rgb(png_ptr: png_structrp) {
    update_transform(png_ptr, PNG_EXPAND | PNG_EXPAND_tRNS);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_tRNS_to_alpha(png_ptr: png_structrp) {
    update_transform(png_ptr, PNG_EXPAND | PNG_EXPAND_tRNS);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_gray_to_rgb(png_ptr: png_structrp) {
    update_transform(png_ptr, PNG_EXPAND | PNG_GRAY_TO_RGB);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_scale_16(png_ptr: png_structrp) {
    update_transform(png_ptr, PNG_SCALE_16_TO_8);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_strip_16(png_ptr: png_structrp) {
    update_transform(png_ptr, PNG_16_TO_8);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_quantize(
    png_ptr: png_structrp,
    _palette: png_colorp,
    _num_palette: c_int,
    _maximum_colors: c_int,
    _histogram: png_const_uint_16p,
    _full_quantize: c_int,
) {
    let _ = rtran_ok(png_ptr, false);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_shift(
    png_ptr: png_structrp,
    true_bits: png_const_color_8p,
) {
    if true_bits.is_null() || !rtran_ok(png_ptr, false) {
        return;
    }

    let mut core = unsafe { read_core(png_ptr) };
    core.shift = unsafe { *true_bits };
    unsafe {
        write_core(png_ptr, &core);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_swap_alpha(png_ptr: png_structrp) {
    update_transform(png_ptr, PNG_SWAP_ALPHA);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_invert_alpha(png_ptr: png_structrp) {
    update_transform(png_ptr, PNG_INVERT_ALPHA);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_invert_mono(png_ptr: png_structrp) {
    update_transform(png_ptr, PNG_INVERT_MONO);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_bgr(png_ptr: png_structrp) {
    update_transform(png_ptr, PNG_BGR);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_row(
    png_ptr: png_structrp,
    row: png_bytep,
    display_row: png_bytep,
) {
    if unsafe { png_safe_call_read_row(png_ptr, row, display_row) } == 0 {
        return;
    }

    unsafe {
        mask_packed_row_padding(png_ptr, row);
        mask_packed_row_padding(png_ptr, display_row);
    }
}
