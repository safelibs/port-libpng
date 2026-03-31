use crate::chunks::{call_app_error, read_core, write_core};
use crate::interlace::mask_packed_row_padding_for_width;
use crate::types::*;
use core::ffi::c_int;

const PNG_HAVE_IHDR: png_uint_32 = 0x01;
const PNG_INVERT_MONO: png_uint_32 = 0x0020;
const PNG_SHIFT: png_uint_32 = 0x0008;
const PNG_INTERLACE: png_uint_32 = 0x0002;
const PNG_QUANTIZE: png_uint_32 = 0x0040;
const PNG_EXPAND: png_uint_32 = 0x1000;
const PNG_GRAY_TO_RGB: png_uint_32 = 0x4000;
const PNG_SWAP_ALPHA: png_uint_32 = 0x20000;
const PNG_INVERT_ALPHA: png_uint_32 = 0x80000;
const PNG_EXPAND_16: png_uint_32 = 0x0200;
const PNG_16_TO_8: png_uint_32 = 0x0400;
const PNG_EXPAND_TRNS: png_uint_32 = 0x2000000;
const PNG_SCALE_16_TO_8: png_uint_32 = 0x4000000;
const PNG_BGR: png_uint_32 = 0x0001;

const PNG_FLAG_ROW_INIT: png_uint_32 = 0x0040;
const PNG_FLAG_DETECT_UNINITIALIZED: png_uint_32 = 0x4000;

unsafe extern "C" {
    fn png_safe_call_set_quantize(
        png_ptr: png_structrp,
        palette: png_colorp,
        num_palette: c_int,
        maximum_colors: c_int,
        histogram: png_const_uint_16p,
        full_quantize: c_int,
    ) -> c_int;
    fn upstream_png_read_row(png_ptr: png_structrp, row: png_bytep, display_row: png_bytep);
}

fn rtran_ok(png_ptr: png_structrp, need_ihdr: bool) -> bool {
    if png_ptr.is_null() {
        return false;
    }

    let mut core = read_core(png_ptr);
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
            let _ = call_app_error(png_ptr, b"invalid before the PNG header has been read\0");
        }
        return false;
    }

    core.flags |= PNG_FLAG_DETECT_UNINITIALIZED;
    write_core(png_ptr, &core);
    true
}

fn rowbytes_for_width(width: usize, pixel_depth: usize) -> usize {
    (width * pixel_depth).div_ceil(8)
}

fn infer_pixel_depth(core: png_safe_read_core, width: usize, rowbytes: usize) -> usize {
    let transformed = usize::from(core.transformed_pixel_depth);
    if transformed != 0 {
        return transformed;
    }

    let derived = usize::from(core.channels) * usize::from(core.bit_depth);
    if derived != 0 && rowbytes_for_width(width, derived) == rowbytes {
        return derived;
    }

    const CANDIDATES: [usize; 9] = [1, 2, 4, 8, 16, 24, 32, 48, 64];
    CANDIDATES
        .into_iter()
        .find(|candidate| rowbytes_for_width(width, *candidate) == rowbytes)
        .unwrap_or(0)
}

fn update_transform(png_ptr: png_structrp, transform_mask: png_uint_32) {
    if !rtran_ok(png_ptr, false) {
        return;
    }

    let mut core = read_core(png_ptr);
    core.transformations |= transform_mask;
    write_core(png_ptr, &core);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_expand(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, {
        update_transform(png_ptr, PNG_EXPAND | PNG_EXPAND_TRNS);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_expand_16(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, {
        update_transform(png_ptr, PNG_EXPAND_16 | PNG_EXPAND | PNG_EXPAND_TRNS);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_palette_to_rgb(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, {
        update_transform(png_ptr, PNG_EXPAND | PNG_EXPAND_TRNS);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_tRNS_to_alpha(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, {
        update_transform(png_ptr, PNG_EXPAND | PNG_EXPAND_TRNS);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_gray_to_rgb(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, {
        update_transform(png_ptr, PNG_EXPAND | PNG_GRAY_TO_RGB);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_scale_16(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, {
        update_transform(png_ptr, PNG_SCALE_16_TO_8);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_strip_16(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, {
        update_transform(png_ptr, PNG_16_TO_8);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_quantize(
    png_ptr: png_structrp,
    palette: png_colorp,
    num_palette: c_int,
    maximum_colors: c_int,
    histogram: png_const_uint_16p,
    full_quantize: c_int,
) {
    crate::abi_guard!(png_ptr, {
        if !rtran_ok(png_ptr, false) {
            return;
        }

        if unsafe {
            png_safe_call_set_quantize(
                png_ptr,
                palette,
                num_palette,
                maximum_colors,
                histogram,
                full_quantize,
            )
        } == 0
        {
            return;
        }

        let mut core = read_core(png_ptr);
        core.transformations |= PNG_QUANTIZE;
        write_core(png_ptr, &core);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_shift(png_ptr: png_structrp, true_bits: png_const_color_8p) {
    crate::abi_guard!(png_ptr, {
        if true_bits.is_null() || !rtran_ok(png_ptr, false) {
            return;
        }

        let mut core = read_core(png_ptr);
        core.transformations |= PNG_SHIFT;
        core.shift = unsafe { *true_bits };
        write_core(png_ptr, &core);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_swap_alpha(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, {
        update_transform(png_ptr, PNG_SWAP_ALPHA);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_invert_alpha(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, {
        update_transform(png_ptr, PNG_INVERT_ALPHA);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_invert_mono(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, {
        update_transform(png_ptr, PNG_INVERT_MONO);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_bgr(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, {
        update_transform(png_ptr, PNG_BGR);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_row(
    png_ptr: png_structrp,
    row: png_bytep,
    display_row: png_bytep,
) {
    crate::abi_guard!(png_ptr, {
        if png_ptr.is_null() {
            return;
        }

        if row.is_null() && display_row.is_null() {
            unsafe {
                upstream_png_read_row(png_ptr, row, display_row);
            }
            return;
        }

        let core = read_core(png_ptr);
        let handled_interlace = core.interlaced != 0 && (core.transformations & PNG_INTERLACE) != 0;

        if handled_interlace {
            unsafe {
                upstream_png_read_row(png_ptr, row, display_row);
            }

            let rowbytes = if core.rowbytes != 0 {
                core.rowbytes
            } else {
                core.info_rowbytes
            };
            let width = usize::try_from(core.width).unwrap_or(0);
            let pixel_depth = infer_pixel_depth(core, width, rowbytes);

            if width != 0 && pixel_depth != 0 && pixel_depth < 8 {
                if !row.is_null() && rowbytes != 0 {
                    let row_slice = unsafe { std::slice::from_raw_parts_mut(row, rowbytes) };
                    mask_packed_row_padding_for_width(row_slice, width, pixel_depth);
                }

                if !display_row.is_null() && rowbytes != 0 {
                    let display_slice =
                        unsafe { std::slice::from_raw_parts_mut(display_row, rowbytes) };
                    mask_packed_row_padding_for_width(display_slice, width, pixel_depth);
                }
            }

            return;
        }

        unsafe {
            upstream_png_read_row(png_ptr, row, display_row);
        }

        if core.interlaced == 0 {
            let rowbytes = if core.rowbytes != 0 {
                core.rowbytes
            } else {
                core.info_rowbytes
            };
            let width = usize::try_from(core.width).unwrap_or(0);
            let pixel_depth = infer_pixel_depth(core, width, rowbytes);

            if width != 0 && pixel_depth != 0 && pixel_depth < 8 {
                if !row.is_null() && rowbytes != 0 {
                    let row_slice = unsafe { std::slice::from_raw_parts_mut(row, rowbytes) };
                    mask_packed_row_padding_for_width(row_slice, width, pixel_depth);
                }

                if !display_row.is_null() && rowbytes != 0 {
                    let display_slice =
                        unsafe { std::slice::from_raw_parts_mut(display_row, rowbytes) };
                    mask_packed_row_padding_for_width(display_slice, width, pixel_depth);
                }
            }
        }
    });
}
