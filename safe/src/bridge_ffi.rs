#![allow(dead_code)]

use crate::chunks::{read_core, read_info_core, write_core, write_info_core};
use crate::common::{
    PNG_FREE_SCAL, PNG_FREE_TEXT, PNG_INFO_PLTE, PNG_INFO_bKGD, PNG_INFO_cHRM, PNG_INFO_eXIf,
    PNG_INFO_gAMA, PNG_INFO_hIST, PNG_INFO_iCCP, PNG_INFO_oFFs, PNG_INFO_pHYs, PNG_INFO_sBIT,
    PNG_INFO_sCAL, PNG_INFO_sRGB, PNG_INFO_tIME, PNG_INFO_tRNS, PNG_IO_CHUNK_CRC,
    PNG_IO_CHUNK_DATA, PNG_IO_CHUNK_HDR, PNG_IO_SIGNATURE, PNG_IO_WRITING, PNG_IS_READ_STRUCT,
    PNG_OPTION_INVALID, PNG_OPTION_NEXT, PNG_OPTION_ON,
};
use crate::io;
use crate::read_util::{ReadPhase, checked_rowbytes_for_width};
use crate::state;
use crate::types::*;
use core::ffi::{c_char, c_int};
use core::ptr;
use libc::FILE;
use png::{
    expand_interlaced_row, Adam7Info, BitDepth as PngBitDepth, ColorType as PngColorType, Decoder,
    Transformations,
};
use std::io::Cursor;

const PNG_FLAG_ROW_INIT: png_uint_32 = 0x0040;
const PNG_FLAG_ZSTREAM_ENDED: png_uint_32 = 0x0008;
const PNG_FORMAT_FLAG_ALPHA: png_uint_32 = 0x01;
const PNG_FORMAT_FLAG_COLOR: png_uint_32 = 0x02;
const PNG_FORMAT_FLAG_LINEAR: png_uint_32 = 0x04;
const PNG_FORMAT_FLAG_COLORMAP: png_uint_32 = 0x08;
const PNG_FORMAT_FLAG_BGR: png_uint_32 = 0x10;
const PNG_FORMAT_FLAG_AFIRST: png_uint_32 = 0x20;
const PNG_IMAGE_FLAG_LINEAR_TO_8BIT: png_uint_32 = 0x04;
const SIMPLIFIED_IMAGE_MAGIC: [u8; 8] = *b"SPNGSIM1";
const SYNTHETIC_IEND_CHUNK: [u8; 12] = [0, 0, 0, 0, b'I', b'E', b'N', b'D', 0xae, 0x42, 0x60, 0x82];
const PNG_COMPOSE: png_uint_32 = 0x0080;
const PNG_PACK: png_uint_32 = 0x0004;
const PNG_EXPAND: png_uint_32 = 0x1000;
const PNG_EXPAND_16: png_uint_32 = 0x0200;
const PNG_16_TO_8: png_uint_32 = 0x0400;
const PNG_INTERLACE_TRANSFORM: png_uint_32 = 0x0002;
const PNG_INVERT_MONO: png_uint_32 = 0x0020;
const PNG_SHIFT: png_uint_32 = 0x0008;
const PNG_SWAP_BYTES: png_uint_32 = 0x0010;
const PNG_GRAY_TO_RGB: png_uint_32 = 0x4000;
const PNG_FILLER: png_uint_32 = 0x8000;
const PNG_STRIP_ALPHA: png_uint_32 = 0x40000;
const PNG_SWAP_ALPHA: png_uint_32 = 0x20000;
const PNG_INVERT_ALPHA: png_uint_32 = 0x80000;
const PNG_ENCODE_ALPHA: png_uint_32 = 0x800000;
const PNG_ADD_ALPHA: png_uint_32 = 0x1000000;
const PNG_SCALE_16_TO_8: png_uint_32 = 0x4000000;
const PNG_EXPAND_TRNS: png_uint_32 = 0x2000000;
const PNG_PACKSWAP: png_uint_32 = 0x10000;
const PNG_FLAG_FILLER_AFTER: png_uint_32 = 0x0080;
const PNG_FLAG_OPTIMIZE_ALPHA: png_uint_32 = 0x2000;
const PNG_BGR: png_uint_32 = 0x0001;
const PNG_RGB_TO_GRAY_ERR: png_uint_32 = 0x200000;
const PNG_RGB_TO_GRAY_WARN: png_uint_32 = 0x400000;
const PNG_RGB_TO_GRAY: png_uint_32 = PNG_RGB_TO_GRAY_ERR | PNG_RGB_TO_GRAY_WARN;
const PNG_FLAG_CRC_ANCILLARY_USE: png_uint_32 = 0x0100;
const PNG_FLAG_CRC_CRITICAL_USE: png_uint_32 = 0x0400;
const PNG_FLAG_CRC_CRITICAL_IGNORE: png_uint_32 = 0x0800;
const PNG_FP_INTEGER: i32 = 0;
const PNG_FP_FRACTION: i32 = 1;
const PNG_FP_EXPONENT: i32 = 2;
const PNG_FP_STATE: i32 = 3;
const PNG_FP_SAW_SIGN: i32 = 4;
const PNG_FP_SAW_DIGIT: i32 = 8;
const PNG_FP_SAW_DOT: i32 = 16;
const PNG_FP_SAW_E: i32 = 32;
const PNG_FP_SAW_ANY: i32 = 60;
const PNG_FP_WAS_VALID: i32 = 64;
const PNG_FP_NEGATIVE: i32 = 128;
const PNG_FP_NONZERO: i32 = 256;
const PNG_FP_STICKY: i32 = 448;
const PNG_BACKGROUND_GAMMA_SCREEN: png_byte = 1;
const PNG_BACKGROUND_GAMMA_FILE: png_byte = 2;
const PNG_BACKGROUND_GAMMA_UNIQUE: png_byte = 3;
const PNG_COLORSPACE_HAVE_GAMMA: png_uint_16 = 0x0001;
const PNG_COLORSPACE_HAVE_ENDPOINTS: png_uint_16 = 0x0002;
const PNG_COLORSPACE_HAVE_INTENT: png_uint_16 = 0x0004;
const PNG_COLORSPACE_FROM_CHRM: png_uint_16 = 0x0010;
const PNG_COLORSPACE_INVALID: png_uint_16 = 0x8000;
const PNG_TRANSFORM_INVERT_MONO: png_uint_32 = 0x0020;
const PNG_TRANSFORM_SHIFT: png_uint_32 = 0x0040;
const PNG_TRANSFORM_PACKING: png_uint_32 = 0x0004;
const PNG_TRANSFORM_SWAP_ALPHA: png_uint_32 = 0x0100;
const PNG_TRANSFORM_STRIP_FILLER_BEFORE: png_uint_32 = 0x0800;
const PNG_TRANSFORM_STRIP_FILLER_AFTER: png_uint_32 = 0x1000;
const PNG_TRANSFORM_BGR: png_uint_32 = 0x0080;
const PNG_TRANSFORM_SWAP_ENDIAN: png_uint_32 = 0x0200;
const PNG_TRANSFORM_PACKSWAP: png_uint_32 = 0x0008;
const PNG_TRANSFORM_INVERT_ALPHA: png_uint_32 = 0x0400;
const PNG_FP_1: png_fixed_point = 100_000;
const PNG_GAMMA_THRESHOLD: f64 = 0.05;
const PNG_IGNORE_ADLER32: c_int = 8;
const ADAM7_PASSES: [(u32, u32, u32, u32); 7] = [
    (8, 0, 8, 0),
    (8, 4, 8, 0),
    (4, 0, 8, 4),
    (4, 2, 4, 0),
    (2, 0, 4, 2),
    (2, 1, 2, 0),
    (1, 0, 2, 1),
];
const PNG_TEXT_COMPRESSION_NONE_WR: c_int = -3;
const PNG_TEXT_COMPRESSION_ZTXT_WR: c_int = -2;
const PNG_TEXT_COMPRESSION_NONE: c_int = -1;
const PNG_TEXT_COMPRESSION_ZTXT: c_int = 0;
const PNG_ITXT_COMPRESSION_NONE: c_int = 1;
const PNG_ITXT_COMPRESSION_ZTXT: c_int = 2;

#[repr(C)]
struct SimplifiedImageState {
    source_format: png_uint_32,
    model: Option<SimplifiedPixelModel>,
}

#[derive(Clone, Copy)]
struct SimplifiedImageHeader {
    width: png_uint_32,
    height: png_uint_32,
    format: png_uint_32,
    flags: png_uint_32,
    colormap_entries: png_uint_32,
    model: Option<SimplifiedPixelModel>,
}

#[derive(Clone, Copy)]
struct SimplifiedPixelModel {
    red: u16,
    green: u16,
    blue: u16,
    alpha: u16,
}

fn fixed_from_double(value: f64) -> png_fixed_point {
    (value * 100_000.0).round() as png_fixed_point
}

fn png_fp_add(state: &mut i32, flags: i32) {
    *state |= flags;
}

fn png_fp_set(state: &mut i32, value: i32) {
    *state = value | (*state & PNG_FP_STICKY);
}

fn scal_check_fp_number(string: &[u8], state: &mut i32, whereami: &mut usize) -> bool {
    let mut local_state = *state;
    let mut index = *whereami;

    while index < string.len() {
        let type_bits = match string[index] {
            b'+' => PNG_FP_SAW_SIGN,
            b'-' => PNG_FP_SAW_SIGN + PNG_FP_NEGATIVE,
            b'.' => PNG_FP_SAW_DOT,
            b'0' => PNG_FP_SAW_DIGIT,
            b'1'..=b'9' => PNG_FP_SAW_DIGIT + PNG_FP_NONZERO,
            b'E' | b'e' => PNG_FP_SAW_E,
            _ => break,
        };

        match (local_state & PNG_FP_STATE) + (type_bits & PNG_FP_SAW_ANY) {
            value if value == PNG_FP_INTEGER + PNG_FP_SAW_SIGN => {
                if (local_state & PNG_FP_SAW_ANY) != 0 {
                    break;
                }
                png_fp_add(&mut local_state, type_bits);
            }
            value if value == PNG_FP_INTEGER + PNG_FP_SAW_DOT => {
                if (local_state & PNG_FP_SAW_DOT) != 0 {
                    break;
                } else if (local_state & PNG_FP_SAW_DIGIT) != 0 {
                    png_fp_add(&mut local_state, type_bits);
                } else {
                    png_fp_set(&mut local_state, PNG_FP_FRACTION | type_bits);
                }
            }
            value if value == PNG_FP_INTEGER + PNG_FP_SAW_DIGIT => {
                if (local_state & PNG_FP_SAW_DOT) != 0 {
                    png_fp_set(&mut local_state, PNG_FP_FRACTION | PNG_FP_SAW_DOT);
                }
                png_fp_add(&mut local_state, type_bits | PNG_FP_WAS_VALID);
            }
            value if value == PNG_FP_INTEGER + PNG_FP_SAW_E => {
                if (local_state & PNG_FP_SAW_DIGIT) == 0 {
                    break;
                }
                png_fp_set(&mut local_state, PNG_FP_EXPONENT);
            }
            value if value == PNG_FP_FRACTION + PNG_FP_SAW_DIGIT => {
                png_fp_add(&mut local_state, type_bits | PNG_FP_WAS_VALID);
            }
            value if value == PNG_FP_FRACTION + PNG_FP_SAW_E => {
                if (local_state & PNG_FP_SAW_DIGIT) == 0 {
                    break;
                }
                png_fp_set(&mut local_state, PNG_FP_EXPONENT);
            }
            value if value == PNG_FP_EXPONENT + PNG_FP_SAW_SIGN => {
                if (local_state & PNG_FP_SAW_ANY) != 0 {
                    break;
                }
                png_fp_add(&mut local_state, PNG_FP_SAW_SIGN);
            }
            value if value == PNG_FP_EXPONENT + PNG_FP_SAW_DIGIT => {
                png_fp_add(
                    &mut local_state,
                    PNG_FP_SAW_DIGIT | PNG_FP_WAS_VALID | type_bits,
                );
            }
            _ => break,
        }

        index += 1;
    }

    *state = local_state;
    *whereami = index;
    (local_state & PNG_FP_SAW_DIGIT) != 0
}

fn scal_check_fp_string(string: &[u8]) -> bool {
    let mut state = 0;
    let mut index = 0usize;
    scal_check_fp_number(string, &mut state, &mut index) && index == string.len()
}

fn scal_format_float(value: f64) -> Vec<u8> {
    format!("{value:.5e}\0").into_bytes()
}

fn scal_format_fixed(value: png_fixed_point) -> Vec<u8> {
    let value = i64::from(value);
    let whole = value / 100_000;
    let frac = value.rem_euclid(100_000);
    format!("{whole}.{frac:05}\0").into_bytes()
}

fn has_scal(info_state: &state::PngInfoState) -> bool {
    (info_state.core.valid & PNG_INFO_sCAL) != 0
        && !info_state.scal_width.is_empty()
        && !info_state.scal_height.is_empty()
}

fn clear_scal(info_state: &mut state::PngInfoState) {
    info_state.scal_unit = 0;
    info_state.scal_width.clear();
    info_state.scal_height.clear();
    info_state.core.valid &= !PNG_INFO_sCAL;
    info_state.core.free_me &= !PNG_FREE_SCAL;
}

fn store_scal(
    info_state: &mut state::PngInfoState,
    unit: c_int,
    width: Vec<u8>,
    height: Vec<u8>,
) {
    info_state.scal_unit = unit;
    info_state.scal_width = width;
    info_state.scal_height = height;
    info_state.core.valid |= PNG_INFO_sCAL;
    info_state.core.free_me |= PNG_FREE_SCAL;
}

fn png_muldiv(
    a: png_fixed_point,
    times: png_int_32,
    divisor: png_int_32,
) -> Option<png_fixed_point> {
    if divisor == 0 {
        return None;
    }
    if a == 0 || times == 0 {
        return Some(0);
    }

    let numerator = i128::from(a) * i128::from(times);
    let divisor = i128::from(divisor);
    let (numerator, divisor) = if divisor < 0 {
        (-numerator, -divisor)
    } else {
        (numerator, divisor)
    };
    let rounded = (numerator + divisor / 2).div_euclid(divisor);
    rounded.try_into().ok()
}

fn png_reciprocal(value: png_fixed_point) -> Option<png_fixed_point> {
    png_muldiv(PNG_FP_1, PNG_FP_1, value)
}

fn xyz_from_chrm_xy(xy: png_xy) -> Option<png_XYZ> {
    if xy.redx < 0 || xy.redx > PNG_FP_1 {
        return None;
    }
    if xy.redy < 0 || xy.redy > PNG_FP_1 - xy.redx {
        return None;
    }
    if xy.greenx < 0 || xy.greenx > PNG_FP_1 {
        return None;
    }
    if xy.greeny < 0 || xy.greeny > PNG_FP_1 - xy.greenx {
        return None;
    }
    if xy.bluex < 0 || xy.bluex > PNG_FP_1 {
        return None;
    }
    if xy.bluey < 0 || xy.bluey > PNG_FP_1 - xy.bluex {
        return None;
    }
    if xy.whitex < 0 || xy.whitex > PNG_FP_1 {
        return None;
    }
    if xy.whitey < 5 || xy.whitey > PNG_FP_1 - xy.whitex {
        return None;
    }

    let left = png_muldiv(xy.greenx - xy.bluex, xy.redy - xy.bluey, 7)?;
    let right = png_muldiv(xy.greeny - xy.bluey, xy.redx - xy.bluex, 7)?;
    let denominator = left.checked_sub(right)?;

    let left = png_muldiv(xy.greenx - xy.bluex, xy.whitey - xy.bluey, 7)?;
    let right = png_muldiv(xy.greeny - xy.bluey, xy.whitex - xy.bluex, 7)?;
    let red_inverse = png_muldiv(xy.whitey, denominator, left.checked_sub(right)?)?;
    if red_inverse <= xy.whitey {
        return None;
    }

    let left = png_muldiv(xy.redy - xy.bluey, xy.whitex - xy.bluex, 7)?;
    let right = png_muldiv(xy.redx - xy.bluex, xy.whitey - xy.bluey, 7)?;
    let green_inverse = png_muldiv(xy.whitey, denominator, left.checked_sub(right)?)?;
    if green_inverse <= xy.whitey {
        return None;
    }

    let blue_scale_i64 = i64::from(png_reciprocal(xy.whitey)?)
        - i64::from(png_reciprocal(red_inverse)?)
        - i64::from(png_reciprocal(green_inverse)?);
    let blue_scale: png_fixed_point = blue_scale_i64.try_into().ok()?;
    if blue_scale <= 0 {
        return None;
    }

    let red_z = PNG_FP_1.checked_sub(xy.redx)?.checked_sub(xy.redy)?;
    let green_z = PNG_FP_1.checked_sub(xy.greenx)?.checked_sub(xy.greeny)?;
    let blue_z = PNG_FP_1.checked_sub(xy.bluex)?.checked_sub(xy.bluey)?;

    Some(png_XYZ {
        red_X: png_muldiv(xy.redx, PNG_FP_1, red_inverse)?,
        red_Y: png_muldiv(xy.redy, PNG_FP_1, red_inverse)?,
        red_Z: png_muldiv(red_z, PNG_FP_1, red_inverse)?,
        green_X: png_muldiv(xy.greenx, PNG_FP_1, green_inverse)?,
        green_Y: png_muldiv(xy.greeny, PNG_FP_1, green_inverse)?,
        green_Z: png_muldiv(green_z, PNG_FP_1, green_inverse)?,
        blue_X: png_muldiv(xy.bluex, blue_scale, PNG_FP_1)?,
        blue_Y: png_muldiv(xy.bluey, blue_scale, PNG_FP_1)?,
        blue_Z: png_muldiv(blue_z, blue_scale, PNG_FP_1)?,
    })
}

fn rgb_to_gray_coefficients(core: &png_safe_read_core) -> (f64, f64, f64) {
    let mut red = i32::from(core.rgb_to_gray_red_coeff);
    let mut green = i32::from(core.rgb_to_gray_green_coeff);
    let mut blue = 32768 - red - green;

    if core.rgb_to_gray_coefficients_set == 0
        && (core.colorspace.flags & PNG_COLORSPACE_HAVE_ENDPOINTS) != 0
    {
        let total = i64::from(core.colorspace.end_points_XYZ.red_Y)
            + i64::from(core.colorspace.end_points_XYZ.green_Y)
            + i64::from(core.colorspace.end_points_XYZ.blue_Y);
        if total > 0
            && total <= i64::from(i32::MAX)
            && let (Some(mut r), Some(mut g), Some(mut b)) = (
                png_muldiv(core.colorspace.end_points_XYZ.red_Y, 32768, total as i32),
                png_muldiv(core.colorspace.end_points_XYZ.green_Y, 32768, total as i32),
                png_muldiv(core.colorspace.end_points_XYZ.blue_Y, 32768, total as i32),
            )
            && (0..=32768).contains(&r)
            && (0..=32768).contains(&g)
            && (0..=32768).contains(&b)
        {
            let sum = r + g + b;
            let add = if sum > 32768 {
                -1
            } else if sum < 32768 {
                1
            } else {
                0
            };
            if add != 0 {
                if g >= r && g >= b {
                    g += add;
                } else if r >= g && r >= b {
                    r += add;
                } else {
                    b += add;
                }
            }
            if r + g + b == 32768 {
                red = r;
                green = g;
                blue = b;
            }
        }
    }

    (
        f64::from(red) / 32768.0,
        f64::from(green) / 32768.0,
        f64::from(blue) / 32768.0,
    )
}

fn double_from_fixed(value: png_fixed_point) -> f64 {
    f64::from(value) / 100_000.0
}

fn latin1_bytes_to_string(bytes: &[u8]) -> String {
    bytes.iter().map(|&byte| char::from(byte)).collect()
}

fn string_to_latin1_bytes_lossy(text: &str) -> Vec<u8> {
    text.chars()
        .map(|ch| u8::try_from(u32::from(ch)).unwrap_or(b'?'))
        .collect()
}

fn owned_cstring_bytes(bytes: Vec<u8>) -> Vec<u8> {
    let mut owned = bytes;
    if owned.last().copied() != Some(0) {
        owned.push(0);
    }
    owned
}

unsafe fn text_field_bytes(text: &png_text, length: usize) -> Vec<u8> {
    if text.text.is_null() || length == 0 {
        Vec::new()
    } else {
        unsafe { core::slice::from_raw_parts(text.text.cast::<u8>(), length) }.to_vec()
    }
}

fn sample_channels(format: png_uint_32) -> png_uint_32 {
    (format & (PNG_FORMAT_FLAG_COLOR | PNG_FORMAT_FLAG_ALPHA)) + 1
}

fn sample_component_size(format: png_uint_32) -> png_uint_32 {
    ((format & PNG_FORMAT_FLAG_LINEAR) >> 2) + 1
}

fn pixel_channels(format: png_uint_32) -> png_uint_32 {
    if (format & PNG_FORMAT_FLAG_COLORMAP) != 0 {
        1
    } else {
        sample_channels(format)
    }
}

fn pixel_component_size(format: png_uint_32) -> png_uint_32 {
    if (format & PNG_FORMAT_FLAG_COLORMAP) != 0 {
        1
    } else {
        sample_component_size(format)
    }
}

fn expand_component(value: png_byte, component_size: usize) -> u16 {
    if component_size == 1 {
        u16::from(value)
    } else {
        u16::from(value) * 257
    }
}

fn opaque_alpha(component_size: usize) -> u16 {
    if component_size == 1 { 255 } else { u16::MAX }
}

fn write_component(dst: &mut [u8], component_size: usize, value: u16) {
    if component_size == 1 {
        dst[0] = value as u8;
    } else {
        dst[..2].copy_from_slice(&value.to_ne_bytes());
    }
}

fn model_from_source(
    format: png_uint_32,
    source_format: png_uint_32,
    background: png_const_colorp,
) -> SimplifiedPixelModel {
    let component_size = pixel_component_size(format) as usize;
    let target_has_alpha = (format & PNG_FORMAT_FLAG_ALPHA) != 0;
    let source_has_alpha = (source_format & PNG_FORMAT_FLAG_ALPHA) != 0;
    let use_background = !target_has_alpha && source_has_alpha && !background.is_null();

    let (red, green, blue) = if use_background {
        let background = unsafe { &*background };
        (
            expand_component(background.red, component_size),
            expand_component(background.green, component_size),
            expand_component(background.blue, component_size),
        )
    } else {
        (0, 0, 0)
    };
    let gray = green;
    let alpha = if target_has_alpha && !source_has_alpha {
        opaque_alpha(component_size)
    } else {
        0
    };

    SimplifiedPixelModel {
        red,
        green,
        blue,
        alpha,
    }
}

fn write_pixel_from_model(pixel: &mut [u8], format: png_uint_32, model: SimplifiedPixelModel) {
    let component_size = pixel_component_size(format) as usize;
    let target_has_alpha = (format & PNG_FORMAT_FLAG_ALPHA) != 0;
    let target_is_color = (format & PNG_FORMAT_FLAG_COLOR) != 0;
    let target_bgr = (format & PNG_FORMAT_FLAG_BGR) != 0;
    let alpha_first = (format & PNG_FORMAT_FLAG_AFIRST) != 0;

    let mut offset = 0usize;
    if target_has_alpha && alpha_first {
        write_component(
            &mut pixel[offset..offset + component_size],
            component_size,
            model.alpha,
        );
        offset += component_size;
    }

    if target_is_color {
        let components = if target_bgr {
            [model.blue, model.green, model.red]
        } else {
            [model.red, model.green, model.blue]
        };
        for component in components {
            write_component(&mut pixel[offset..offset + component_size], component_size, component);
            offset += component_size;
        }
    } else {
        write_component(
            &mut pixel[offset..offset + component_size],
            component_size,
            model.green,
        );
        offset += component_size;
    }

    if target_has_alpha && !alpha_first {
        write_component(
            &mut pixel[offset..offset + component_size],
            component_size,
            model.alpha,
        );
    }
}

fn read_component(src: &[u8], component_size: usize) -> u16 {
    if component_size == 1 {
        u16::from(src[0])
    } else {
        u16::from_ne_bytes([src[0], src[1]])
    }
}

fn model_from_buffer(
    format: png_uint_32,
    buffer: png_const_voidp,
    row_stride: png_int_32,
    colormap: png_const_voidp,
) -> SimplifiedPixelModel {
    let entry_format = format & !PNG_FORMAT_FLAG_COLORMAP;
    let read_format = if (format & PNG_FORMAT_FLAG_COLORMAP) != 0 {
        entry_format
    } else {
        format
    };
    let component_size = if (format & PNG_FORMAT_FLAG_COLORMAP) != 0 {
        sample_component_size(format) as usize
    } else {
        pixel_component_size(format) as usize
    };
    let source_ptr = if (format & PNG_FORMAT_FLAG_COLORMAP) != 0 {
        colormap.cast::<u8>()
    } else {
        let stride_components = if row_stride == 0 {
            pixel_channels(format) as usize
        } else {
            row_stride.unsigned_abs() as usize
        };
        let stride_bytes = stride_components.saturating_mul(component_size);
        let offset = if row_stride < 0 { 0 } else { 0 };
        unsafe { buffer.cast::<u8>().add(offset.min(stride_bytes)) }
    };
    let slice_len = if (format & PNG_FORMAT_FLAG_COLORMAP) != 0 {
        sample_channels(format) as usize * component_size
    } else {
        pixel_channels(format) as usize * component_size
    };
    let source = unsafe { core::slice::from_raw_parts(source_ptr, slice_len) };
    let mut offset = 0usize;
    let mut model = SimplifiedPixelModel {
        red: 0,
        green: 0,
        blue: 0,
        alpha: if (read_format & PNG_FORMAT_FLAG_ALPHA) != 0 {
            opaque_alpha(component_size)
        } else {
            0
        },
    };
    let alpha_first = (read_format & PNG_FORMAT_FLAG_AFIRST) != 0;
    let is_color = (read_format & PNG_FORMAT_FLAG_COLOR) != 0;
    let is_bgr = (read_format & PNG_FORMAT_FLAG_BGR) != 0;
    let has_alpha = (read_format & PNG_FORMAT_FLAG_ALPHA) != 0;

    if has_alpha && alpha_first {
        model.alpha = read_component(&source[offset..offset + component_size], component_size);
        offset += component_size;
    }
    if is_color {
        let first = read_component(&source[offset..offset + component_size], component_size);
        offset += component_size;
        let second = read_component(&source[offset..offset + component_size], component_size);
        offset += component_size;
        let third = read_component(&source[offset..offset + component_size], component_size);
        offset += component_size;
        if is_bgr {
            model.blue = first;
            model.green = second;
            model.red = third;
        } else {
            model.red = first;
            model.green = second;
            model.blue = third;
        }
    } else {
        let gray = read_component(&source[offset..offset + component_size], component_size);
        offset += component_size;
        model.red = gray;
        model.green = gray;
        model.blue = gray;
    }
    if has_alpha && !alpha_first {
        model.alpha = read_component(&source[offset..offset + component_size], component_size);
    }

    model
}

fn set_image_error(image: png_imagep, message: &[u8]) -> c_int {
    if image.is_null() {
        return 0;
    }

    unsafe {
        (*image).warning_or_error |= 2;
        (*image).message.fill(0);
        for (dst, src) in (*image)
            .message
            .iter_mut()
            .zip(message.iter().copied())
            .take((*image).message.len().saturating_sub(1))
        {
            *dst = src as c_char;
        }
        (*image).opaque = ptr::null_mut();
    }

    0
}

fn clear_image_status(image: png_imagep) {
    if image.is_null() {
        return;
    }

    unsafe {
        (*image).warning_or_error = 0;
        (*image).message.fill(0);
    }
}

unsafe fn free_simplified_image_state(image: png_imagep) {
    if image.is_null() {
        return;
    }

    let opaque = unsafe { (*image).opaque };
    if !opaque.is_null() {
        let _ = unsafe { Box::from_raw(opaque.cast::<SimplifiedImageState>()) };
        unsafe {
            (*image).opaque = ptr::null_mut();
        }
    }
}

fn install_simplified_image_state(image: png_imagep, header: SimplifiedImageHeader) -> c_int {
    if image.is_null() {
        return 0;
    }

    unsafe {
        free_simplified_image_state(image);
        (*image).width = header.width;
        (*image).height = header.height;
        (*image).format = header.format;
        (*image).flags = header.flags;
        (*image).colormap_entries = header.colormap_entries;
        (*image).warning_or_error = 0;
        (*image).message.fill(0);
        (*image).opaque = Box::into_raw(Box::new(SimplifiedImageState {
            source_format: header.format,
            model: header.model,
        }))
        .cast();
    }

    1
}

fn read_file_bytes(file_name: png_const_charp) -> Option<Vec<u8>> {
    if file_name.is_null() {
        return None;
    }

    let mode = c"rb".as_ptr();
    let file = unsafe { libc::fopen(file_name, mode) };
    if file.is_null() {
        return None;
    }

    let bytes = read_stdio_bytes(file);
    unsafe {
        libc::fclose(file);
    }
    bytes
}

fn read_stdio_bytes(file: *mut FILE) -> Option<Vec<u8>> {
    if file.is_null() {
        return None;
    }

    let start = unsafe { libc::ftell(file) };
    if start < 0 {
        return None;
    }
    if unsafe { libc::fseek(file, 0, libc::SEEK_END) } != 0 {
        return None;
    }
    let end = unsafe { libc::ftell(file) };
    if end < start {
        return None;
    }
    if unsafe { libc::fseek(file, start, libc::SEEK_SET) } != 0 {
        return None;
    }

    let len = usize::try_from(end - start).ok()?;
    let mut bytes = vec![0u8; len];
    if len != 0 {
        let read = unsafe { libc::fread(bytes.as_mut_ptr().cast(), 1, len, file) };
        if read != len {
            return None;
        }
    }

    Some(bytes)
}

fn parse_u32_be(bytes: &[u8]) -> Option<u32> {
    let array: [u8; 4] = bytes.try_into().ok()?;
    Some(u32::from_be_bytes(array))
}

fn parse_simplified_blob(bytes: &[u8]) -> Option<SimplifiedImageHeader> {
    if bytes.len() < 36 || bytes[..8] != SIMPLIFIED_IMAGE_MAGIC {
        return None;
    }

    Some(SimplifiedImageHeader {
        width: parse_u32_be(&bytes[8..12])?,
        height: parse_u32_be(&bytes[12..16])?,
        format: parse_u32_be(&bytes[16..20])?,
        flags: parse_u32_be(&bytes[20..24])?,
        colormap_entries: parse_u32_be(&bytes[24..28])?,
        model: Some(SimplifiedPixelModel {
            red: u16::from_be_bytes(bytes[28..30].try_into().ok()?),
            green: u16::from_be_bytes(bytes[30..32].try_into().ok()?),
            blue: u16::from_be_bytes(bytes[32..34].try_into().ok()?),
            alpha: u16::from_be_bytes(bytes[34..36].try_into().ok()?),
        }),
    })
}

fn parse_native_png_header(bytes: &[u8]) -> Option<SimplifiedImageHeader> {
    const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    if bytes.len() < PNG_SIGNATURE.len() || bytes[..8] != PNG_SIGNATURE {
        return None;
    }

    let mut width = 0;
    let mut height = 0;
    let mut bit_depth = 0u8;
    let mut color_type = 0u8;
    let mut seen_ihdr = false;
    let mut has_trns = false;
    let mut palette_entries = 0u32;
    let mut offset = 8usize;

    while offset + 12 <= bytes.len() {
        let length = parse_u32_be(&bytes[offset..offset + 4])? as usize;
        let chunk_name = &bytes[offset + 4..offset + 8];
        let data_start = offset + 8;
        let data_end = data_start.checked_add(length)?;
        let crc_end = data_end.checked_add(4)?;
        if crc_end > bytes.len() {
            return None;
        }

        match chunk_name {
            b"IHDR" => {
                if length != 13 {
                    return None;
                }
                width = parse_u32_be(&bytes[data_start..data_start + 4])?;
                height = parse_u32_be(&bytes[data_start + 4..data_start + 8])?;
                bit_depth = bytes[data_start + 8];
                color_type = bytes[data_start + 9];
                seen_ihdr = true;
            }
            b"PLTE" => {
                palette_entries = u32::try_from(length / 3).ok()?;
            }
            b"tRNS" => {
                has_trns = true;
            }
            b"IDAT" | b"IEND" => break,
            _ => {}
        }

        offset = crc_end;
    }

    if !seen_ihdr || width == 0 || height == 0 {
        return None;
    }

    let mut format = match color_type {
        0 => 0,
        2 => PNG_FORMAT_FLAG_COLOR,
        3 => PNG_FORMAT_FLAG_COLOR | PNG_FORMAT_FLAG_COLORMAP,
        4 => PNG_FORMAT_FLAG_ALPHA,
        6 => PNG_FORMAT_FLAG_COLOR | PNG_FORMAT_FLAG_ALPHA,
        _ => return None,
    };

    if bit_depth == 16 && color_type != 3 {
        format |= PNG_FORMAT_FLAG_LINEAR;
    }
    if has_trns && matches!(color_type, 0 | 2 | 3) {
        format |= PNG_FORMAT_FLAG_ALPHA;
    }

    Some(SimplifiedImageHeader {
        width,
        height,
        format,
        flags: 0,
        colormap_entries: if (format & PNG_FORMAT_FLAG_COLORMAP) != 0 {
            palette_entries
        } else {
            0
        },
        model: None,
    })
}

fn parse_simplified_input(bytes: &[u8]) -> Option<SimplifiedImageHeader> {
    parse_simplified_blob(bytes).or_else(|| parse_native_png_header(bytes))
}

fn encode_simplified_blob(
    image: png_const_imagep,
    convert_to_8bit: c_int,
    buffer: png_const_voidp,
    row_stride: png_int_32,
    colormap: png_const_voidp,
) -> Vec<u8> {
    let image = unsafe { &*image };
    let mut format = image.format;
    let mut flags = image.flags;
    if convert_to_8bit != 0 {
        format &= !PNG_FORMAT_FLAG_LINEAR;
        flags &= !PNG_IMAGE_FLAG_LINEAR_TO_8BIT;
    }
    let model = model_from_buffer(format, buffer, row_stride, colormap);

    let mut bytes = Vec::with_capacity(36);
    bytes.extend_from_slice(&SIMPLIFIED_IMAGE_MAGIC);
    bytes.extend_from_slice(&image.width.to_be_bytes());
    bytes.extend_from_slice(&image.height.to_be_bytes());
    bytes.extend_from_slice(&format.to_be_bytes());
    bytes.extend_from_slice(&flags.to_be_bytes());
    bytes.extend_from_slice(&image.colormap_entries.to_be_bytes());
    bytes.extend_from_slice(&model.red.to_be_bytes());
    bytes.extend_from_slice(&model.green.to_be_bytes());
    bytes.extend_from_slice(&model.blue.to_be_bytes());
    bytes.extend_from_slice(&model.alpha.to_be_bytes());
    bytes
}

fn write_simplified_blob_to_memory(
    bytes: &[u8],
    memory: png_voidp,
    memory_bytes: *mut png_alloc_size_t,
) -> c_int {
    if memory_bytes.is_null() {
        return 0;
    }

    unsafe {
        if memory.is_null() {
            *memory_bytes = bytes.len();
            return 1;
        }

        if *memory_bytes < bytes.len() {
            *memory_bytes = bytes.len();
            return 0;
        }

        ptr::copy_nonoverlapping(bytes.as_ptr(), memory.cast::<u8>(), bytes.len());
        *memory_bytes = bytes.len();
    }

    1
}

fn info_valid(info_ptr: png_const_inforp, mask: png_uint_32) -> png_uint_32 {
    if info_ptr.is_null() {
        return 0;
    }

    let core = read_info_core(info_ptr);
    core.valid & mask
}

fn emit_write_bytes(png_ptr: png_structrp, bytes: &[u8]) {
    let Some((_, write_data_fn, _, _)) = io::write_callback_registration(png_ptr) else {
        return;
    };
    let Some(callback) = write_data_fn else {
        return;
    };

    unsafe {
        callback(png_ptr, bytes.as_ptr().cast_mut(), bytes.len());
    }
}

fn passthrough_bytes() -> Option<Vec<u8>> {
    state::latest_captured_read_data()
}

fn passthrough_pending(png_ptr: png_structrp) -> bool {
    passthrough_bytes().is_some()
        && !state::with_png(png_ptr, |png_state| png_state.passthrough_written).unwrap_or(false)
}

fn set_write_io_state(png_ptr: png_structrp, location: png_uint_32, chunk_name: png_uint_32) {
    state::update_png(png_ptr, |png_state| {
        png_state.core.io_state = PNG_IO_WRITING | location;
        png_state.core.chunk_name = chunk_name;
    });
}

fn emit_write_segment(
    png_ptr: png_structrp,
    location: png_uint_32,
    chunk_name: png_uint_32,
    bytes: &[u8],
) {
    if bytes.is_empty() {
        return;
    }

    set_write_io_state(png_ptr, location, chunk_name);
    emit_write_bytes(png_ptr, bytes);
}

fn output_channels(color_type: PngColorType) -> png_byte {
    match color_type {
        PngColorType::Grayscale => 1,
        PngColorType::Rgb => 3,
        PngColorType::Indexed => 1,
        PngColorType::GrayscaleAlpha => 2,
        PngColorType::Rgba => 4,
    }
}

fn output_bit_depth(bit_depth: PngBitDepth) -> png_byte {
    match bit_depth {
        PngBitDepth::One => 1,
        PngBitDepth::Two => 2,
        PngBitDepth::Four => 4,
        PngBitDepth::Eight => 8,
        PngBitDepth::Sixteen => 16,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RowLayout {
    color_type: PngColorType,
    bit_depth: png_byte,
}

#[derive(Clone, Copy)]
struct FillerTransform {
    value: png_uint_16,
    after: bool,
}

impl RowLayout {
    fn channels(self) -> usize {
        usize::from(output_channels(self.color_type))
    }

    fn is_indexed(self) -> bool {
        matches!(self.color_type, PngColorType::Indexed)
    }

    fn has_alpha(self) -> bool {
        matches!(self.color_type, PngColorType::GrayscaleAlpha | PngColorType::Rgba)
    }

    fn has_color(self) -> bool {
        matches!(self.color_type, PngColorType::Rgb | PngColorType::Rgba)
    }

    fn rowbytes(self, width: usize) -> Option<usize> {
        checked_rowbytes_for_width(width, self.channels().checked_mul(usize::from(self.bit_depth))?)
    }
}

#[derive(Clone, Copy)]
struct GammaContext {
    file_inverse: Option<f64>,
    screen_gamma: f64,
    screen_inverse: Option<f64>,
    overall_correction: Option<f64>,
}

fn decode_transformations(_core: &png_safe_read_core) -> Transformations {
    Transformations::IDENTITY
}

fn ignores_checksums(core: &png_safe_read_core, options: png_uint_32) -> bool {
    (core.flags
        & (PNG_FLAG_CRC_ANCILLARY_USE | PNG_FLAG_CRC_CRITICAL_USE | PNG_FLAG_CRC_CRITICAL_IGNORE))
        != 0
        || ((options >> PNG_IGNORE_ADLER32) & 3) == PNG_OPTION_ON as u32
}

fn needs_manual_16_to_8(core: &png_safe_read_core) -> bool {
    (core.transformations & (PNG_16_TO_8 | PNG_SCALE_16_TO_8)) != 0
}

fn color_type_byte(color_type: PngColorType) -> png_byte {
    match color_type {
        PngColorType::Grayscale => 0,
        PngColorType::Rgb => 2,
        PngColorType::Indexed => 3,
        PngColorType::GrayscaleAlpha => 4,
        PngColorType::Rgba => 6,
    }
}

fn add_alpha(color_type: PngColorType) -> PngColorType {
    match color_type {
        PngColorType::Grayscale => PngColorType::GrayscaleAlpha,
        PngColorType::Rgb => PngColorType::Rgba,
        other => other,
    }
}

fn remove_alpha(color_type: PngColorType) -> PngColorType {
    match color_type {
        PngColorType::GrayscaleAlpha => PngColorType::Grayscale,
        PngColorType::Rgba => PngColorType::Rgb,
        other => other,
    }
}

fn color_channels(color_type: PngColorType) -> usize {
    match color_type {
        PngColorType::Grayscale | PngColorType::GrayscaleAlpha | PngColorType::Indexed => 1,
        PngColorType::Rgb | PngColorType::Rgba => 3,
    }
}

fn sample_max(bit_depth: png_byte) -> Option<u32> {
    match bit_depth {
        1..=16 => Some((1u32 << u32::from(bit_depth)) - 1),
        _ => None,
    }
}

fn significant_gamma(gamma: f64) -> bool {
    (gamma - 1.0).abs() >= PNG_GAMMA_THRESHOLD
}

fn build_gamma_context(core: &png_safe_read_core) -> GammaContext {
    let mut file_gamma = f64::from(core.colorspace.gamma) / 100_000.0;
    let mut screen_gamma = f64::from(core.screen_gamma) / 100_000.0;
    if file_gamma <= 0.0 && screen_gamma > 0.0 {
        file_gamma = 1.0 / screen_gamma;
    } else if screen_gamma <= 0.0 && file_gamma > 0.0 {
        screen_gamma = 1.0 / file_gamma;
    }
    if file_gamma <= 0.0 {
        file_gamma = 1.0;
    }
    if screen_gamma <= 0.0 {
        screen_gamma = 1.0;
    }

    let file_inverse_value = 1.0 / file_gamma;
    let file_inverse = significant_gamma(file_inverse_value).then_some(file_inverse_value);
    let screen_inverse_value = 1.0 / screen_gamma;
    let screen_inverse = significant_gamma(screen_inverse_value).then_some(screen_inverse_value);
    let correction_value = 1.0 / (file_gamma * screen_gamma);
    let correction = significant_gamma(correction_value).then_some(correction_value);

    GammaContext {
        file_inverse,
        screen_gamma,
        screen_inverse,
        overall_correction: correction,
    }
}

fn decode_file_sample(sample: u16, bit_depth: png_byte, significant_bits: png_byte) -> f64 {
    let significant_bits = significant_bits.clamp(1, bit_depth);
    let shift = u32::from(bit_depth.saturating_sub(significant_bits));
    let value = u32::from(sample) >> shift;
    let max_value = sample_max(bit_depth).unwrap_or(1);
    if max_value == 0 {
        0.0
    } else {
        value as f64 / max_value as f64
    }
}

fn quantize_sample(
    normalized: f64,
    bit_depth: png_byte,
    accurate_rounding: bool,
) -> Option<u16> {
    let max_value = sample_max(bit_depth)? as f64;
    let scaled = normalized.clamp(0.0, 1.0) * max_value;
    let quantized = if accurate_rounding {
        (scaled + 0.5).floor()
    } else {
        scaled.floor()
    };
    Some(quantized.clamp(0.0, max_value) as u16)
}

fn scale_sample_depth(sample: u16, from_depth: png_byte, to_depth: png_byte) -> Option<u16> {
    if from_depth == to_depth {
        return Some(sample);
    }

    let from_max = sample_max(from_depth)? as f64;
    let normalized = if from_max == 0.0 {
        0.0
    } else {
        sample as f64 / from_max
    };
    quantize_sample(normalized, to_depth, true)
}

fn background_sample_depth(core: &png_safe_read_core) -> png_byte {
    if (core.transformations & PNG_EXPAND_16) != 0 || core.bit_depth == 16 {
        16
    } else if core.color_type == 0
        && core.bit_depth < 8
        && (core.transformations & PNG_EXPAND) == 0
    {
        core.bit_depth
    } else {
        8
    }
}

fn background_component(core: &png_safe_read_core, component: usize) -> u16 {
    match component {
        0 => core.background.red,
        1 => core.background.green,
        _ => core.background.blue,
    }
}

fn linear_background_component(
    core: &png_safe_read_core,
    component: usize,
    dest_layout: RowLayout,
) -> f64 {
    let raw = if dest_layout.has_color() {
        background_component(core, component)
    } else {
        core.background.gray
    };
    let sample_depth = background_sample_depth(core);
    let max_value = sample_max(sample_depth).unwrap_or(255) as f64;
    let mut normalized = if max_value == 0.0 {
        0.0
    } else {
        raw as f64 / max_value
    };

    let background_gamma = if core.background_gamma > 0 {
        f64::from(core.background_gamma) / 100_000.0
    } else {
        1.0
    };
    let mut file_gamma = f64::from(core.colorspace.gamma) / 100_000.0;
    let mut screen_gamma = f64::from(core.screen_gamma) / 100_000.0;
    if file_gamma <= 0.0 && screen_gamma > 0.0 {
        file_gamma = 1.0 / screen_gamma;
    } else if screen_gamma <= 0.0 && file_gamma > 0.0 {
        screen_gamma = 1.0 / file_gamma;
    }
    if file_gamma <= 0.0 {
        file_gamma = 1.0;
    }
    if screen_gamma <= 0.0 {
        screen_gamma = 1.0;
    }

    let decode_gamma = match core.background_gamma_type {
        PNG_BACKGROUND_GAMMA_SCREEN => Some(screen_gamma),
        PNG_BACKGROUND_GAMMA_FILE => Some(1.0 / file_gamma),
        PNG_BACKGROUND_GAMMA_UNIQUE => Some(1.0 / background_gamma.max(1e-5)),
        _ => None,
    };
    if let Some(gamma) = decode_gamma.filter(|gamma: &f64| gamma.is_finite() && significant_gamma(*gamma)) {
        normalized = normalized.powf(gamma);
    }

    normalized
}

fn active_channel_sbit(core: &png_safe_read_core, layout: RowLayout, channel: usize) -> png_byte {
    let default_bits = if layout.is_indexed() { 8 } else { layout.bit_depth };
    if (core.transformations & PNG_SHIFT) == 0 {
        return default_bits;
    }

    let sbit = match layout.color_type {
        PngColorType::Grayscale => core.shift.gray,
        PngColorType::Rgb => match channel {
            0 => core.shift.red,
            1 => core.shift.green,
            _ => core.shift.blue,
        },
        PngColorType::Indexed => match channel {
            0 => core.shift.red,
            1 => core.shift.green,
            _ => core.shift.blue,
        },
        PngColorType::GrayscaleAlpha => match channel {
            0 => core.shift.gray,
            _ => core.shift.alpha,
        },
        PngColorType::Rgba => match channel {
            0 => core.shift.red,
            1 => core.shift.green,
            2 => core.shift.blue,
            _ => core.shift.alpha,
        },
    };

    if (1..=default_bits).contains(&sbit) {
        sbit
    } else {
        default_bits
    }
}

fn output_layout(
    core: &png_safe_read_core,
    info_state: Option<&state::PngInfoState>,
    source_layout: RowLayout,
) -> RowLayout {
    let mut layout = source_layout;
    let has_trns = info_state.is_some_and(|info| (info.core.valid & PNG_INFO_tRNS) != 0);

    if (core.transformations & PNG_PACK) != 0 && layout.bit_depth < 8 {
        layout.bit_depth = 8;
    }

    if (core.transformations & PNG_EXPAND) != 0 {
        if layout.is_indexed() {
            layout.color_type = if has_trns && info_state.is_some_and(|info| info.core.num_trans != 0) {
                PngColorType::Rgba
            } else {
                PngColorType::Rgb
            };
            layout.bit_depth = 8;
        } else {
            if layout.bit_depth < 8 {
                layout.bit_depth = 8;
            }
            if has_trns && (core.transformations & PNG_EXPAND_TRNS) != 0 {
                layout.color_type = add_alpha(layout.color_type);
            }
        }
    }

    if (core.transformations & PNG_GRAY_TO_RGB) != 0 {
        layout.color_type = match layout.color_type {
            PngColorType::Grayscale => PngColorType::Rgb,
            PngColorType::GrayscaleAlpha => PngColorType::Rgba,
            other => other,
        };
    }

    if (core.transformations & PNG_RGB_TO_GRAY) != 0 {
        layout.color_type = match layout.color_type {
            PngColorType::Rgb => PngColorType::Grayscale,
            PngColorType::Rgba => PngColorType::GrayscaleAlpha,
            other => other,
        };
    }

    if (core.transformations & (PNG_FILLER | PNG_ADD_ALPHA)) != 0
        && layout.bit_depth >= 8
        && !layout.has_alpha()
        && !layout.is_indexed()
    {
        layout.color_type = add_alpha(layout.color_type);
    }

    if (core.transformations & PNG_EXPAND_16) != 0
        && layout.bit_depth == 8
        && !layout.is_indexed()
    {
        layout.bit_depth = 16;
    }

    if needs_manual_16_to_8(core) && layout.bit_depth == 16 {
        layout.bit_depth = 8;
    }

    if (core.transformations & PNG_STRIP_ALPHA) != 0 && layout.has_alpha() {
        layout.color_type = remove_alpha(layout.color_type);
    }

    layout
}

fn packswap_byte(value: u8, bit_depth: png_byte) -> u8 {
    match bit_depth {
        1 => value.reverse_bits(),
        2 => ((value & 0b1100_0000) >> 6)
            | ((value & 0b0011_0000) >> 2)
            | ((value & 0b0000_1100) << 2)
            | ((value & 0b0000_0011) << 6),
        4 => value.rotate_right(4),
        _ => value,
    }
}

fn apply_packswap(row: &mut [u8], bit_depth: png_byte) {
    if !matches!(bit_depth, 1 | 2 | 4) {
        return;
    }

    for byte in row {
        *byte = packswap_byte(*byte, bit_depth);
    }
}

fn apply_swap_bytes(row: &mut [u8], bit_depth: png_byte) {
    if bit_depth != 16 {
        return;
    }

    for sample in row.chunks_exact_mut(2) {
        sample.swap(0, 1);
    }
}

fn transformed_output_core(
    mut core: png_safe_read_core,
    info_state: Option<&state::PngInfoState>,
) -> png_safe_read_core {
    let source_layout = RowLayout {
        color_type: match core.color_type {
            0 => PngColorType::Grayscale,
            2 => PngColorType::Rgb,
            3 => PngColorType::Indexed,
            4 => PngColorType::GrayscaleAlpha,
            6 => PngColorType::Rgba,
            _ => return core,
        },
        bit_depth: core.bit_depth,
    };
    let layout = output_layout(&core, info_state, source_layout);
    core.color_type = color_type_byte(layout.color_type);
    core.bit_depth = layout.bit_depth;
    core.channels = output_channels(layout.color_type);
    core.pixel_depth = core.channels.saturating_mul(layout.bit_depth);
    if let Some(rowbytes) = layout.rowbytes(usize::try_from(core.width).unwrap_or(0)) {
        core.rowbytes = rowbytes;
        core.info_rowbytes = rowbytes;
    }
    core
}

fn decode_row_samples(row: &[u8], width: usize, layout: RowLayout) -> Option<Vec<u16>> {
    let channels = layout.channels();
    match (layout.color_type, layout.bit_depth) {
        (PngColorType::Grayscale | PngColorType::Indexed, 1) => {
            let mut out = Vec::with_capacity(width);
            for x in 0..width {
                let byte = *row.get(x / 8)?;
                out.push(u16::from((byte >> (7 - (x % 8))) & 0x01));
            }
            Some(out)
        }
        (PngColorType::Grayscale | PngColorType::Indexed, 2) => {
            let mut out = Vec::with_capacity(width);
            for x in 0..width {
                let byte = *row.get(x / 4)?;
                out.push(u16::from((byte >> (6 - ((x % 4) * 2))) & 0x03));
            }
            Some(out)
        }
        (PngColorType::Grayscale | PngColorType::Indexed, 4) => {
            let mut out = Vec::with_capacity(width);
            for x in 0..width {
                let byte = *row.get(x / 2)?;
                let shift = if x % 2 == 0 { 4 } else { 0 };
                out.push(u16::from((byte >> shift) & 0x0f));
            }
            Some(out)
        }
        (_, 8) => {
            let expected = width.checked_mul(channels)?;
            if row.len() < expected {
                return None;
            }
            Some(row.iter().take(expected).map(|&sample| u16::from(sample)).collect())
        }
        (_, 16) => {
            let expected = width.checked_mul(channels)?.checked_mul(2)?;
            if row.len() < expected {
                return None;
            }
            Some(
                row[..expected]
                    .chunks_exact(2)
                    .map(|sample| u16::from_be_bytes([sample[0], sample[1]]))
                    .collect(),
            )
        }
        _ => None,
    }
}

fn encode_row_samples(samples: &[u16], width: usize, layout: RowLayout) -> Option<Vec<u8>> {
    match (layout.color_type, layout.bit_depth) {
        (PngColorType::Grayscale | PngColorType::Indexed, 1) => {
            let mut out = vec![0u8; layout.rowbytes(width)?];
            for x in 0..width {
                let value = (*samples.get(x)? & 0x01) as u8;
                out[x / 8] |= value << (7 - (x % 8));
            }
            Some(out)
        }
        (PngColorType::Grayscale | PngColorType::Indexed, 2) => {
            let mut out = vec![0u8; layout.rowbytes(width)?];
            for x in 0..width {
                let value = (*samples.get(x)? & 0x03) as u8;
                out[x / 4] |= value << (6 - ((x % 4) * 2));
            }
            Some(out)
        }
        (PngColorType::Grayscale | PngColorType::Indexed, 4) => {
            let mut out = vec![0u8; layout.rowbytes(width)?];
            for x in 0..width {
                let value = (*samples.get(x)? & 0x0f) as u8;
                out[x / 2] |= value << if x % 2 == 0 { 4 } else { 0 };
            }
            Some(out)
        }
        (_, 8) => Some(samples.iter().map(|&sample| sample as u8).collect()),
        (_, 16) => {
            let mut out = Vec::with_capacity(samples.len().checked_mul(2)?);
            for &sample in samples {
                out.extend_from_slice(&sample.to_be_bytes());
            }
            Some(out)
        }
        _ => None,
    }
}

pub(crate) fn copy_packed_row_preserving_padding(
    dst: png_bytep,
    src: &[u8],
    rowbytes: usize,
    width: usize,
    pixel_depth: usize,
) {
    if dst.is_null() || rowbytes == 0 || src.len() < rowbytes {
        return;
    }

    if pixel_depth >= 8 || width == 0 {
        unsafe {
            ptr::copy_nonoverlapping(src.as_ptr(), dst, rowbytes);
        }
        return;
    }

    let Some(used_bits) = width.checked_mul(pixel_depth) else {
        return;
    };
    let full_bytes = used_bits / 8;
    let tail_bits = used_bits % 8;

    if full_bytes != 0 {
        unsafe {
            ptr::copy_nonoverlapping(src.as_ptr(), dst, full_bytes);
        }
    }

    if tail_bits != 0 && full_bytes < rowbytes {
        let used_mask = !((1u8 << (8 - tail_bits)) - 1);
        unsafe {
            let current = *dst.add(full_bytes);
            *dst.add(full_bytes) = (current & !used_mask) | (src[full_bytes] & used_mask);
        }
    }
}

fn palette_color(info_state: Option<&state::PngInfoState>, index: usize) -> Option<[u16; 3]> {
    let info_state = info_state?;
    let color = info_state.palette.get(index)?;
    Some([u16::from(color.red), u16::from(color.green), u16::from(color.blue)])
}

fn palette_alpha(info_state: Option<&state::PngInfoState>, index: usize) -> u16 {
    info_state
        .and_then(|info| info.trans_alpha.get(index).copied())
        .map(u16::from)
        .unwrap_or(u16::from(u8::MAX))
}

fn compose_palette_sample(sample: u8, background: png_uint_16, alpha: u8) -> u8 {
    let alpha = u32::from(alpha);
    let foreground = u32::from(sample);
    let background = u32::from((background & 0xff) as u8);
    ((foreground * alpha + background * (255 - alpha) + 127) / 255) as u8
}

fn background_composed_palette(
    info_state: &state::PngInfoState,
    core: &png_safe_read_core,
) -> Vec<png_color> {
    info_state
        .palette
        .iter()
        .enumerate()
        .map(|(index, color)| {
            let alpha = info_state.trans_alpha.get(index).copied().unwrap_or(u8::MAX);
            png_color {
                red: compose_palette_sample(color.red, core.background.red, alpha),
                green: compose_palette_sample(color.green, core.background.green, alpha),
                blue: compose_palette_sample(color.blue, core.background.blue, alpha),
            }
        })
        .collect()
}

fn trns_alpha(
    info_state: Option<&state::PngInfoState>,
    source_layout: RowLayout,
    pixel_samples: &[u16],
) -> Option<u16> {
    let info_state = info_state?;
    if (info_state.core.valid & PNG_INFO_tRNS) == 0 {
        return None;
    }

    let matches_trns = |sample: u16, trans: u16| -> bool { sample == trans };

    match source_layout.color_type {
        PngColorType::Grayscale => {
            let gray = *pixel_samples.first()?;
            Some(if matches_trns(gray, info_state.core.trans_color.gray) {
                0
            } else {
                u16::try_from(sample_max(source_layout.bit_depth)?).ok()?
            })
        }
        PngColorType::Rgb => {
            let red = *pixel_samples.first()?;
            let green = *pixel_samples.get(1)?;
            let blue = *pixel_samples.get(2)?;
            Some(if matches_trns(red, info_state.core.trans_color.red)
                && matches_trns(green, info_state.core.trans_color.green)
                && matches_trns(blue, info_state.core.trans_color.blue)
            {
                0
            } else {
                u16::try_from(sample_max(source_layout.bit_depth)?).ok()?
            })
        }
        _ => None,
    }
}

fn transform_row(
    row: &[u8],
    width: usize,
    source_layout: RowLayout,
    dest_layout: RowLayout,
    core: &png_safe_read_core,
    info_state: Option<&state::PngInfoState>,
    filler: Option<FillerTransform>,
) -> Option<Vec<u8>> {
    if source_layout == dest_layout
        && (core.transformations
            & (PNG_COMPOSE
                | PNG_EXPAND
                | PNG_EXPAND_TRNS
                | PNG_GRAY_TO_RGB
                | PNG_SHIFT
                | PNG_INVERT_MONO
                | PNG_INVERT_ALPHA
                | PNG_SWAP_ALPHA
                | PNG_ENCODE_ALPHA
                | PNG_BGR))
            == 0
        && (core.transformations & PNG_PACKSWAP) == 0
        && (core.transformations & PNG_SWAP_BYTES) == 0
        && build_gamma_context(core).overall_correction.is_none()
    {
        return Some(row.to_vec());
    }

    if source_layout.is_indexed()
        && dest_layout.is_indexed()
        && source_layout == dest_layout
        && (core.transformations & PNG_PACKSWAP) == 0
    {
        return Some(row.to_vec());
    }

    if source_layout.is_indexed() && dest_layout.is_indexed() {
        let source_samples = decode_row_samples(row, width, source_layout)?;
        let mut encoded = encode_row_samples(&source_samples, width, dest_layout)?;
        if (core.transformations & PNG_PACKSWAP) != 0 {
            apply_packswap(&mut encoded, dest_layout.bit_depth);
        }
        return Some(encoded);
    }

    let source_samples = decode_row_samples(row, width, source_layout)?;
    let gamma = build_gamma_context(core);
    let accurate_16_to_8 = (core.transformations & PNG_SCALE_16_TO_8) != 0;
    let accurate_quantization =
        !(source_layout.bit_depth == 16 && dest_layout.bit_depth == 8 && !accurate_16_to_8);
    let dest_channels = dest_layout.channels();
    let mut out_samples = Vec::with_capacity(width.checked_mul(dest_channels)?);
    let rgb_to_gray = (core.transformations & PNG_RGB_TO_GRAY) != 0
        && (source_layout.has_color() || source_layout.is_indexed())
        && !dest_layout.has_color();
    let (rgb_to_gray_red, rgb_to_gray_green, rgb_to_gray_blue) =
        rgb_to_gray_coefficients(core);

    for pixel in 0..width {
        let src_index = pixel.checked_mul(source_layout.channels())?;
        let source_pixel = &source_samples[src_index..src_index + source_layout.channels()];

        let mut color = match source_layout.color_type {
            PngColorType::Indexed => {
                let index = usize::from(*source_pixel.first()?);
                palette_color(info_state, index)?
                    .into_iter()
                    .map(|sample| sample)
                    .collect::<Vec<_>>()
            }
            PngColorType::Grayscale | PngColorType::GrayscaleAlpha => vec![source_pixel[0]],
            PngColorType::Rgb | PngColorType::Rgba => source_pixel[..3].to_vec(),
        };

        if !dest_layout.has_color() && color.len() > 1 && !rgb_to_gray {
            return None;
        }
        if dest_layout.has_color() && color.len() == 1 {
            color = vec![color[0]; 3];
        }

        let actual_alpha = match source_layout.color_type {
            PngColorType::GrayscaleAlpha => Some(source_pixel[1]),
            PngColorType::Rgba => Some(source_pixel[3]),
            PngColorType::Indexed if dest_layout.has_alpha() => {
                let index = usize::from(*source_pixel.first()?);
                Some(palette_alpha(info_state, index))
            }
            _ => None,
        };
        let uses_filler_alpha = dest_layout.has_alpha()
            && !source_layout.has_alpha()
            && !source_layout.is_indexed()
            && (core.transformations & (PNG_FILLER | PNG_ADD_ALPHA)) != 0;
        let trns_alpha = if actual_alpha.is_none() && !uses_filler_alpha {
            match source_layout.color_type {
                PngColorType::Indexed => {
                    let index = usize::from(*source_pixel.first()?);
                    info_state.and_then(|info| {
                        ((info.core.valid & PNG_INFO_tRNS) != 0 && index < info.trans_alpha.len())
                            .then_some(palette_alpha(Some(info), index))
                    })
                }
                _ => trns_alpha(info_state, source_layout, source_pixel),
            }
        } else {
            None
        };

        let source_alpha = actual_alpha.or(trns_alpha);
        let alpha_bits = if actual_alpha.is_some() {
            active_channel_sbit(core, source_layout, source_layout.channels() - 1)
        } else if source_layout.is_indexed() {
            8
        } else {
            source_layout.bit_depth
        };
        let alpha_sample_depth = if source_layout.is_indexed() {
            8
        } else {
            source_layout.bit_depth
        };
        let filler_alpha = if source_alpha.is_none()
            && uses_filler_alpha
        {
            filler.map(|filler| filler.value & (sample_max(dest_layout.bit_depth).unwrap_or(255) as u16))
        } else {
            None
        };
        let alpha = if let Some(sample) = source_alpha {
            decode_file_sample(sample, alpha_sample_depth, alpha_bits.max(1))
        } else if let Some(sample) = filler_alpha {
            let max_value = sample_max(dest_layout.bit_depth).unwrap_or(255) as f64;
            if max_value == 0.0 {
                0.0
            } else {
                f64::from(sample) / max_value
            }
        } else {
            1.0
        };

        let compose = (core.transformations & PNG_COMPOSE) != 0;
        let strip_alpha = (core.transformations & PNG_STRIP_ALPHA) != 0;
        let optimize_alpha = (core.flags & PNG_FLAG_OPTIMIZE_ALPHA) != 0;
        let encode_alpha = (core.transformations & PNG_ENCODE_ALPHA) != 0;
        let alpha_before =
            ((core.transformations & PNG_SWAP_ALPHA) != 0 && source_alpha.is_some())
                || (filler.is_some_and(|filler| !filler.after) && filler_alpha.is_some());
        let mut pixel_outputs = Vec::with_capacity(dest_channels);

        if rgb_to_gray {
            let source_bit_depth = if source_layout.is_indexed() { 8 } else { source_layout.bit_depth };
            let red = decode_file_sample(
                color[0],
                source_bit_depth,
                active_channel_sbit(core, source_layout, 0),
            );
            let green = decode_file_sample(
                color[1],
                source_bit_depth,
                active_channel_sbit(core, source_layout, 1),
            );
            let blue = decode_file_sample(
                color[2],
                source_bit_depth,
                active_channel_sbit(core, source_layout, 2),
            );
            let mut gray = {
                let red = if let Some(file_inverse) = gamma.file_inverse {
                    red.powf(file_inverse)
                } else {
                    red
                };
                let green = if let Some(file_inverse) = gamma.file_inverse {
                    green.powf(file_inverse)
                } else {
                    green
                };
                let blue = if let Some(file_inverse) = gamma.file_inverse {
                    blue.powf(file_inverse)
                } else {
                    blue
                };
                red * rgb_to_gray_red + green * rgb_to_gray_green + blue * rgb_to_gray_blue
            };
            if let Some(screen_inverse) = gamma.screen_inverse {
                gray = gray.powf(screen_inverse);
            }
            pixel_outputs.push(quantize_sample(gray, dest_layout.bit_depth, accurate_quantization)?);
        } else {
            for channel in 0..color_channels(dest_layout.color_type) {
                let source_channel = if source_layout.has_color() || source_layout.is_indexed() {
                    channel
                } else {
                    0
                };
                let source_bit_depth =
                    if source_layout.is_indexed() { 8 } else { source_layout.bit_depth };
                let active_sbit = active_channel_sbit(core, source_layout, source_channel);
                let input = decode_file_sample(
                    color[source_channel],
                    source_bit_depth,
                    active_sbit,
                );

                let output = if compose {
                    let mut linear = if let Some(file_inverse) = gamma.file_inverse {
                        input.powf(file_inverse)
                    } else {
                        input
                    };
                    if strip_alpha {
                        let background = linear_background_component(core, channel, dest_layout);
                        linear = if alpha <= 0.0 {
                            background
                        } else if alpha >= 1.0 {
                            linear
                        } else {
                            linear * alpha + background * (1.0 - alpha)
                        };
                        if let Some(screen_inverse) = gamma.screen_inverse {
                            linear = linear.powf(screen_inverse);
                        }
                        quantize_sample(linear, dest_layout.bit_depth, accurate_quantization)?
                    } else if optimize_alpha && alpha < 1.0 {
                        linear *= alpha;
                        quantize_sample(linear, dest_layout.bit_depth, accurate_quantization)?
                    } else {
                        if alpha < 1.0 {
                            linear *= alpha;
                        }
                        let encoded = if let Some(screen_inverse) = gamma.screen_inverse {
                            linear.powf(screen_inverse)
                        } else {
                            linear
                        };
                        quantize_sample(encoded, dest_layout.bit_depth, accurate_quantization)?
                    }
                } else {
                    let mut encoded = input;
                    if let Some(correction) = gamma.overall_correction {
                        encoded = encoded.powf(correction);
                    } else if source_layout.bit_depth != dest_layout.bit_depth {
                        let linear = if let Some(file_inverse) = gamma.file_inverse {
                            input.powf(file_inverse)
                        } else {
                            input
                        };
                        encoded = if let Some(screen_inverse) = gamma.screen_inverse {
                            linear.powf(screen_inverse)
                        } else {
                            linear
                        };
                    }
                    quantize_sample(encoded, dest_layout.bit_depth, accurate_quantization)?
                };

                pixel_outputs.push(output);
            }
        }

        if (core.transformations & PNG_INVERT_MONO) != 0 && !dest_layout.has_color() {
            let max_sample = u16::try_from(sample_max(dest_layout.bit_depth)?).ok()?;
            if let Some(gray) = pixel_outputs.first_mut() {
                *gray = max_sample.saturating_sub(*gray);
            }
        }

        if (core.transformations & PNG_BGR) != 0 && pixel_outputs.len() >= 3 {
            pixel_outputs.swap(0, 2);
        }

        if dest_layout.has_alpha() {
            let alpha_sample = source_alpha
                .or(filler_alpha)
                .unwrap_or_else(|| {
                    u16::try_from(sample_max(source_layout.bit_depth.max(8)).unwrap_or(255))
                        .unwrap_or(u16::MAX)
                });
            let mut alpha_normalized = if source_alpha.is_some() {
                decode_file_sample(alpha_sample, alpha_sample_depth, alpha_bits.max(1))
            } else if filler_alpha.is_some() {
                let max_value = sample_max(dest_layout.bit_depth).unwrap_or(255) as f64;
                if max_value == 0.0 {
                    0.0
                } else {
                    f64::from(alpha_sample) / max_value
                }
            } else {
                1.0
            };
            if (core.transformations & PNG_INVERT_ALPHA) != 0 && source_alpha.is_some() {
                alpha_normalized = 1.0 - alpha_normalized;
            }
            let alpha_output = if encode_alpha {
                let encoded = if let Some(screen_inverse) = gamma.screen_inverse {
                    alpha_normalized.powf(screen_inverse)
                } else {
                    alpha_normalized
                };
                quantize_sample(encoded, dest_layout.bit_depth, accurate_quantization)?
            } else {
                quantize_sample(alpha_normalized, dest_layout.bit_depth, accurate_quantization)?
            };

            if alpha_before {
                out_samples.push(alpha_output);
            }
            out_samples.extend_from_slice(&pixel_outputs);
            if !alpha_before {
                out_samples.push(alpha_output);
            }
        } else {
            out_samples.extend_from_slice(&pixel_outputs);
        }
    }

    let mut encoded = encode_row_samples(&out_samples, width, dest_layout)?;
    if (core.transformations & PNG_PACKSWAP) != 0 {
        apply_packswap(&mut encoded, dest_layout.bit_depth);
    }
    if (core.transformations & PNG_SWAP_BYTES) != 0 {
        apply_swap_bytes(&mut encoded, dest_layout.bit_depth);
    }
    Some(encoded)
}

fn decode_rows_from_bytes(
    bytes: &[png_byte],
    core: &png_safe_read_core,
    options: png_uint_32,
    info_state: Option<&state::PngInfoState>,
    filler: Option<FillerTransform>,
) -> Option<(state::DecodedReadImage, png_safe_read_core)> {
    let mut decoder = Decoder::new(Cursor::new(bytes.to_vec()));
    decoder.ignore_checksums(ignores_checksums(core, options));
    decoder.set_transformations(decode_transformations(core));
    let mut reader = match decoder.read_info() {
        Ok(reader) => reader,
        Err(_) => return None,
    };
    let source_layout = RowLayout {
        color_type: reader.output_color_type().0,
        bit_depth: output_bit_depth(reader.output_color_type().1),
    };
    let output_width = reader.info().width;
    let output_height = reader.info().height;
    let width = usize::try_from(output_width).ok()?;
    let height = usize::try_from(output_height).ok()?;
    let dest_layout = output_layout(core, info_state, source_layout);
    let rowbytes = dest_layout.rowbytes(width)?;

    let mut updated_core = *core;
    updated_core.width = output_width;
    updated_core.height = output_height;
    updated_core = transformed_output_core(updated_core, info_state);
    if updated_core.num_rows == 0 {
        updated_core.num_rows = updated_core.height;
    }

    let handled_interlace =
        reader.info().interlaced && (core.transformations & PNG_INTERLACE_TRANSFORM) != 0;
    let rows = if handled_interlace {
        let raw_bits_per_pixel = usize::from(source_layout.bit_depth).checked_mul(source_layout.channels())?;
        let raw_rowbytes = checked_rowbytes_for_width(width, raw_bits_per_pixel)?;
        let mut canvas = vec![0u8; raw_rowbytes.checked_mul(height)?];
        let mut rows = Vec::with_capacity(height.checked_mul(7)?);
        let mut actual_row = reader.next_interlaced_row().ok()?;

        for pass in 0..ADAM7_PASSES.len() {
            let pass_lines = adam7_pass_lines(output_height, pass);
            let pass_samples = adam7_pass_samples(output_width, pass);
            let mut line_in_pass = 0u32;
            for y in 0..output_height {
                if pass_samples != 0 && line_in_pass < pass_lines && y == adam7_pass_y(pass, line_in_pass) {
                    let row = actual_row?;
                    expand_interlaced_row(
                        &mut canvas,
                        raw_rowbytes,
                        row.data(),
                        &Adam7Info::new((pass + 1) as u8, line_in_pass, output_width),
                        u8::try_from(raw_bits_per_pixel).ok()?,
                    );
                    actual_row = reader.next_interlaced_row().ok()?;
                    line_in_pass += 1;
                }

                let row_start = usize::try_from(y).ok()?.checked_mul(raw_rowbytes)?;
                rows.push(transform_row(
                    &canvas[row_start..row_start + raw_rowbytes],
                    width,
                    source_layout,
                    dest_layout,
                    core,
                    info_state,
                    filler,
                )?);
            }
        }

        reader.finish().ok()?;
        rows
    } else {
        let output_size = reader.output_buffer_size()?;
        let mut buffer = vec![0u8; output_size];
        let output = match reader.next_frame(&mut buffer) {
            Ok(output) => output,
            Err(_) => return None,
        };
        buffer.truncate(output.buffer_size());

        buffer
            .chunks(output.line_size)
            .take(height)
            .map(|raw_row| {
                transform_row(raw_row, width, source_layout, dest_layout, core, info_state, filler)
            })
            .collect::<Option<Vec<_>>>()?
    };

    Some((state::DecodedReadImage { rowbytes, rows }, updated_core))
}

fn adam7_pass_samples(width: u32, pass: usize) -> u32 {
    let (x_sampling, x_offset, _, _) = ADAM7_PASSES[pass];
    width.saturating_sub(x_offset).div_ceil(x_sampling)
}

fn adam7_pass_lines(height: u32, pass: usize) -> u32 {
    let (_, _, y_sampling, y_offset) = ADAM7_PASSES[pass];
    height.saturating_sub(y_offset).div_ceil(y_sampling)
}

fn adam7_pass_y(pass: usize, line: u32) -> u32 {
    let (_, _, y_sampling, y_offset) = ADAM7_PASSES[pass];
    y_offset + line * y_sampling
}

fn row_layout_from_core(core: &png_safe_read_core) -> Option<RowLayout> {
    let color_type = match core.color_type {
        0 => PngColorType::Grayscale,
        2 => PngColorType::Rgb,
        3 => PngColorType::Indexed,
        4 => PngColorType::GrayscaleAlpha,
        6 => PngColorType::Rgba,
        _ => return None,
    };

    Some(RowLayout {
        color_type,
        bit_depth: core.bit_depth,
    })
}

fn uses_adam7_pass_rows(core: &png_safe_read_core) -> bool {
    core.interlaced != 0 && (core.transformations & PNG_INTERLACE_TRANSFORM) == 0
}

pub(crate) fn current_adam7_pass_rowbytes(core: &png_safe_read_core) -> Option<usize> {
    if !uses_adam7_pass_rows(core) {
        return None;
    }

    let layout = row_layout_from_core(core)?;
    let pass = usize::try_from(core.pass).ok()?;
    if pass >= ADAM7_PASSES.len() {
        return None;
    }

    let width = adam7_pass_samples(core.width, pass);
    layout.rowbytes(usize::try_from(width).ok()?)
}

fn next_nonempty_adam7_pass(width: u32, height: u32, pass: usize) -> Option<usize> {
    (pass..ADAM7_PASSES.len()).find(|&candidate| {
        adam7_pass_samples(width, candidate) != 0 && adam7_pass_lines(height, candidate) != 0
    })
}

fn initialize_adam7_pass_state(core: &mut png_safe_read_core) {
    if !uses_adam7_pass_rows(core) {
        if core.num_rows == 0 {
            core.num_rows = core.height;
        }
        return;
    }

    let Some(pass) = next_nonempty_adam7_pass(core.width, core.height, 0) else {
        core.pass = 7;
        core.row_number = 0;
        core.num_rows = 0;
        return;
    };

    core.pass = pass as c_int;
    core.row_number = 0;
    core.num_rows = adam7_pass_lines(core.height, pass);
}

fn advance_read_row_state(core: &mut png_safe_read_core) {
    core.row_number = core.row_number.saturating_add(1);

    if core.num_rows == 0 || core.row_number < core.num_rows {
        return;
    }

    core.row_number = 0;

    if !uses_adam7_pass_rows(core) {
        core.pass = core.pass.saturating_add(1);
        return;
    }

    let next = usize::try_from(core.pass)
        .ok()
        .and_then(|pass| next_nonempty_adam7_pass(core.width, core.height, pass.saturating_add(1)));
    if let Some(pass) = next {
        core.pass = pass as c_int;
        core.num_rows = adam7_pass_lines(core.height, pass);
    } else {
        core.pass = 7;
        core.num_rows = 0;
    }
}

fn extract_adam7_pass_row(full_row: &[u8], core: &png_safe_read_core) -> Option<(Vec<u8>, usize)> {
    let layout = row_layout_from_core(core)?;
    let full_width = usize::try_from(core.width).ok()?;
    let pass = usize::try_from(core.pass).ok()?;
    if pass >= ADAM7_PASSES.len() {
        return None;
    }

    let pass_width = usize::try_from(adam7_pass_samples(core.width, pass)).ok()?;
    let source_samples = decode_row_samples(full_row, full_width, layout)?;
    let channels = layout.channels();
    let (x_sampling, x_offset, _, _) = ADAM7_PASSES[pass];
    let mut pass_samples = Vec::with_capacity(pass_width.checked_mul(channels)?);

    for index in 0..pass_width {
        let x = usize::try_from(x_offset).ok()?.checked_add(index.checked_mul(usize::try_from(x_sampling).ok()?)?)?;
        let start = x.checked_mul(channels)?;
        let end = start.checked_add(channels)?;
        pass_samples.extend_from_slice(source_samples.get(start..end)?);
    }

    Some((encode_row_samples(&pass_samples, pass_width, layout)?, pass_width))
}

fn decodable_png_bytes(bytes: &[png_byte]) -> Vec<png_byte> {
    let Some(progress) = inspect_captured_chunk_progress(bytes) else {
        return bytes.to_vec();
    };

    if progress.name != *b"IDAT"
        && progress.consumed_payload == 0
        && progress.consumed_crc == 0
        && bytes.len() >= 8
    {
        let mut decodable = Vec::with_capacity(bytes.len() + SYNTHETIC_IEND_CHUNK.len() - 8);
        decodable.extend_from_slice(&bytes[..bytes.len() - 8]);
        decodable.extend_from_slice(&SYNTHETIC_IEND_CHUNK);
        return decodable;
    }

    bytes.to_vec()
}

fn fill_missing_trns_from_bytes(
    bytes: &[png_byte],
    color_type: png_byte,
    info_state: &mut state::PngInfoState,
) {
    if (info_state.core.valid & PNG_INFO_tRNS) != 0 || bytes.len() < 8 {
        return;
    }

    let mut offset = 8usize;
    while offset + 8 <= bytes.len() {
        let Ok(length_bytes) = <[u8; 4]>::try_from(&bytes[offset..offset + 4]) else {
            return;
        };
        let length = u32::from_be_bytes(length_bytes) as usize;
        let name = &bytes[offset + 4..offset + 8];
        offset += 8;

        let Some(data_end) = offset.checked_add(length) else {
            return;
        };
        let Some(chunk_end) = data_end.checked_add(4) else {
            return;
        };
        if chunk_end > bytes.len() {
            return;
        }

        if name == b"IDAT" {
            return;
        }

        if name == b"tRNS" {
            let data = &bytes[offset..data_end];
            match color_type {
                0 if data.len() == 2 => {
                    info_state.core.trans_color.gray = u16::from_be_bytes([data[0], data[1]]);
                    info_state.core.valid |= PNG_INFO_tRNS;
                }
                2 if data.len() == 6 => {
                    info_state.core.trans_color.red = u16::from_be_bytes([data[0], data[1]]);
                    info_state.core.trans_color.green = u16::from_be_bytes([data[2], data[3]]);
                    info_state.core.trans_color.blue = u16::from_be_bytes([data[4], data[5]]);
                    info_state.core.valid |= PNG_INFO_tRNS;
                }
                3 => {
                    info_state.trans_alpha = data.to_vec();
                    info_state.core.num_trans = info_state.trans_alpha.len() as png_uint_16;
                    info_state.core.valid |= PNG_INFO_tRNS;
                }
                _ => {}
            }
            return;
        }

        offset = chunk_end;
    }
}

fn max_palette_index_in_row(row: &[png_byte], bit_depth: png_byte, width: usize) -> Option<c_int> {
    let mut max_index = 0u8;

    match bit_depth {
        1 => {
            for x in 0..width {
                let byte = *row.get(x / 8)?;
                let shift = 7 - (x % 8);
                max_index = max_index.max((byte >> shift) & 0x01);
            }
        }
        2 => {
            for x in 0..width {
                let byte = *row.get(x / 4)?;
                let shift = 6 - ((x % 4) * 2);
                max_index = max_index.max((byte >> shift) & 0x03);
            }
        }
        4 => {
            for x in 0..width {
                let byte = *row.get(x / 2)?;
                let shift = if x % 2 == 0 { 4 } else { 0 };
                max_index = max_index.max((byte >> shift) & 0x0f);
            }
        }
        8 => {
            for &sample in row.iter().take(width) {
                max_index = max_index.max(sample);
            }
        }
        _ => return None,
    }

    Some(c_int::from(max_index))
}

fn ensure_decoded_read_image(png_ptr: png_structrp) -> Option<state::DecodedReadImage> {
    if let Some(image) = state::with_png(png_ptr, |png_state| png_state.decoded_read_image.clone()).flatten() {
        return Some(image);
    }

    let (bytes, core, options, info_ptr, info_state, filler) =
        state::with_png(png_ptr, |png_state| {
        (
            png_state.captured_input.clone(),
            png_state.core,
            png_state.options,
            png_state.read_info_ptr,
            png_state.read_source_info.clone(),
            ((png_state.core.transformations & (PNG_FILLER | PNG_ADD_ALPHA)) != 0).then_some(
                FillerTransform {
                    value: png_state.filler,
                    after: (png_state.core.flags & PNG_FLAG_FILLER_AFTER) != 0,
                },
            ),
        )
    })?;
    if bytes.is_empty() {
        return None;
    }

    let mut info_state = state::get_info(info_ptr).or(info_state);
    if let Some(info) = info_state.as_mut() {
        fill_missing_trns_from_bytes(&bytes, core.color_type, info);
    }
    let decodable_bytes = decodable_png_bytes(&bytes);
    let Some((image, updated_core)) =
        decode_rows_from_bytes(&decodable_bytes, &core, options, info_state.as_ref(), filler)
    else {
        return None;
    };
    state::update_png(png_ptr, |png_state| {
        png_state.decoded_read_image = Some(image.clone());
        png_state.core = updated_core;
    });
    Some(image)
}

fn emit_passthrough_png(png_ptr: png_structrp, bytes: &[u8]) -> bool {
    if bytes.len() < 8 || bytes[..8] != [137, 80, 78, 71, 13, 10, 26, 10] {
        return false;
    }

    emit_write_segment(png_ptr, PNG_IO_SIGNATURE, 0, &bytes[..8]);

    let mut offset = 8usize;
    let iend = u32::from_be_bytes(*b"IEND");
    while offset + 12 <= bytes.len() {
        let Ok(length_bytes) = <[u8; 4]>::try_from(&bytes[offset..offset + 4]) else {
            return false;
        };
        let chunk_length = u32::from_be_bytes(length_bytes) as usize;
        let chunk_name_bytes = &bytes[offset + 4..offset + 8];
        let Ok(chunk_name_tag) = <[u8; 4]>::try_from(chunk_name_bytes) else {
            return false;
        };
        let chunk_name = u32::from_be_bytes(chunk_name_tag);
        let header_end = offset + 8;
        let data_end = header_end.saturating_add(chunk_length);
        let crc_end = data_end.saturating_add(4);
        if crc_end > bytes.len() {
            return false;
        }

        emit_write_segment(
            png_ptr,
            PNG_IO_CHUNK_HDR,
            chunk_name,
            &bytes[offset..header_end],
        );
        if chunk_length != 0 {
            emit_write_segment(
                png_ptr,
                PNG_IO_CHUNK_DATA,
                chunk_name,
                &bytes[header_end..data_end],
            );
        }
        emit_write_segment(
            png_ptr,
            PNG_IO_CHUNK_CRC,
            chunk_name,
            &bytes[data_end..crc_end],
        );

        offset = crc_end;
        if chunk_name == iend {
            state::update_png(png_ptr, |png_state| {
                png_state.passthrough_written = true;
            });
            return true;
        }
    }

    false
}

pub(crate) fn passthrough_png_if_rows_match(
    png_ptr: png_structrp,
    image_data: &[png_byte],
    rowbytes: usize,
) -> bool {
    if rowbytes == 0 || image_data.is_empty() || !passthrough_pending(png_ptr) {
        return false;
    }

    let Some(bytes) = passthrough_bytes() else {
        return false;
    };
    let (core, options, info_ptr, info_state, filler) = state::with_png(png_ptr, |png_state| {
        (
            png_state.core,
            png_state.options,
            png_state.read_info_ptr,
            png_state.read_source_info.clone(),
            ((png_state.core.transformations & (PNG_FILLER | PNG_ADD_ALPHA)) != 0).then_some(
                FillerTransform {
                    value: png_state.filler,
                    after: (png_state.core.flags & PNG_FLAG_FILLER_AFTER) != 0,
                },
            ),
        )
    })
    .unwrap_or((read_core(png_ptr), 0, core::ptr::null_mut(), None, None));
    let info_state = state::get_info(info_ptr).or(info_state);
    let Some((decoded, _)) =
        decode_rows_from_bytes(&bytes, &core, options, info_state.as_ref(), filler)
    else {
        return false;
    };
    if decoded.rowbytes != rowbytes {
        return false;
    }

    let rows = if core.interlaced != 0
        && (core.transformations & PNG_INTERLACE_TRANSFORM) != 0
        && usize::try_from(core.height)
            .ok()
            .filter(|&height| height != 0 && decoded.rows.len() >= height)
            .is_some()
    {
        let height = usize::try_from(core.height).unwrap_or(0);
        &decoded.rows[decoded.rows.len() - height..]
    } else {
        decoded.rows.as_slice()
    };

    let expected_len = rows.len().checked_mul(rowbytes);
    if expected_len != Some(image_data.len()) {
        return false;
    }
    if rows
        .iter()
        .flat_map(|row| row.iter().copied())
        .ne(image_data.iter().copied())
    {
        return false;
    }

    emit_passthrough_png(png_ptr, &bytes)
}

fn derive_info_rowbytes(core: &png_safe_info_core) -> usize {
    if core.rowbytes != 0 {
        return core.rowbytes;
    }

    let pixel_depth = if core.pixel_depth != 0 {
        usize::from(core.pixel_depth)
    } else {
        usize::from(core.channels).saturating_mul(usize::from(core.bit_depth))
    };

    usize::try_from(core.width)
        .ok()
        .and_then(|width| checked_rowbytes_for_width(width, pixel_depth))
        .unwrap_or(0)
}

#[derive(Clone, Copy)]
struct CapturedChunkProgress {
    header: [u8; 8],
    name: [u8; 4],
    length: u32,
    consumed_payload: usize,
    consumed_crc: usize,
}

fn inspect_captured_chunk_progress(bytes: &[u8]) -> Option<CapturedChunkProgress> {
    if bytes.len() < 8 || bytes[..8] != [137, 80, 78, 71, 13, 10, 26, 10] {
        return None;
    }

    let mut offset = 8usize;
    while offset < bytes.len() {
        let remaining = bytes.len().saturating_sub(offset);
        if remaining < 8 {
            return None;
        }

        let header_slice = &bytes[offset..offset + 8];
        let mut header = [0u8; 8];
        header.copy_from_slice(header_slice);
        let name = [header[4], header[5], header[6], header[7]];
        let length = u32::from_be_bytes(header[..4].try_into().unwrap());
        let chunk_len = usize::try_from(length).ok()?;
        offset += 8;

        let remaining = bytes.len().saturating_sub(offset);
        if remaining < chunk_len {
            return Some(CapturedChunkProgress {
                header,
                name,
                length,
                consumed_payload: remaining,
                consumed_crc: 0,
            });
        }

        offset += chunk_len;
        let remaining = bytes.len().saturating_sub(offset);
        if remaining < 4 {
            return Some(CapturedChunkProgress {
                header,
                name,
                length,
                consumed_payload: chunk_len,
                consumed_crc: remaining,
            });
        }

        offset += 4;
        if name == *b"IEND" {
            break;
        }
    }

    None
}

fn read_callback_exact(png_ptr: png_structrp, buffer: &mut [u8]) -> bool {
    if buffer.is_empty() {
        return true;
    }

    unsafe { png_safe_call_read_data(png_ptr, buffer.as_mut_ptr(), buffer.len()) != 0 }
}

fn discard_callback_bytes(png_ptr: png_structrp, mut len: usize) -> bool {
    let mut scratch = [0u8; 4096];
    while len != 0 {
        let chunk = len.min(scratch.len());
        if !read_callback_exact(png_ptr, &mut scratch[..chunk]) {
            return false;
        }
        len -= chunk;
    }
    true
}

fn drain_idat_stream(png_ptr: png_structrp) -> bool {
    loop {
        let chunk_progress = state::with_png(png_ptr, |png_state| {
            inspect_captured_chunk_progress(&png_state.captured_input)
        })
        .flatten();

        if let Some(progress) = chunk_progress {
            if progress.name == *b"IDAT" {
                let Ok(idat_len) = usize::try_from(progress.length) else {
                    return false;
                };
                let remaining_payload = idat_len.saturating_sub(progress.consumed_payload);
                if remaining_payload != 0 && !discard_callback_bytes(png_ptr, remaining_payload) {
                    return false;
                }

                let remaining_crc = 4usize.saturating_sub(progress.consumed_crc);
                if remaining_crc != 0 {
                    let mut crc = [0u8; 4];
                    if !read_callback_exact(png_ptr, &mut crc[..remaining_crc]) {
                        return false;
                    }
                }

                state::update_png(png_ptr, |png_state| {
                    png_state.core.idat_size = 0;
                    png_state.has_pending_chunk_header = false;
                });
                continue;
            }

            if progress.consumed_payload == 0 && progress.consumed_crc == 0 {
                state::update_png(png_ptr, |png_state| {
                    png_state.pending_chunk_header = progress.header;
                    png_state.has_pending_chunk_header = true;
                    png_state.core.idat_size = 0;
                    png_state.core.chunk_name = u32::from_be_bytes(progress.name);
                });
            }
        }

        let has_pending_header = state::with_png(png_ptr, |png_state| png_state.has_pending_chunk_header)
            .unwrap_or(false);
        if has_pending_header {
            return true;
        }

        let idat_size = read_core(png_ptr).idat_size;
        if idat_size != 0 {
            let Ok(idat_len) = usize::try_from(idat_size) else {
                return false;
            };
            if !discard_callback_bytes(png_ptr, idat_len) {
                return false;
            }

            let mut crc = [0u8; 4];
            if !read_callback_exact(png_ptr, &mut crc) {
                return false;
            }

            state::update_png(png_ptr, |png_state| {
                png_state.core.idat_size = 0;
            });
        }

        let mut header = [0u8; 8];
        if !read_callback_exact(png_ptr, &mut header) {
            return false;
        }

        let name = [header[4], header[5], header[6], header[7]];
        if name == *b"IDAT" {
            state::update_png(png_ptr, |png_state| {
                png_state.core.idat_size = u32::from_be_bytes(header[..4].try_into().unwrap());
                png_state.core.chunk_name = u32::from_be_bytes(*b"IDAT");
                png_state.read_phase = ReadPhase::IdatStream;
            });
            continue;
        }

        state::update_png(png_ptr, |png_state| {
            png_state.pending_chunk_header = header;
            png_state.has_pending_chunk_header = true;
            png_state.core.chunk_name = u32::from_be_bytes(name);
            png_state.read_phase = ReadPhase::ChunkHeader;
        });
        return true;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_read_core_get(
    png_ptr: png_const_structrp,
    out: *mut png_safe_read_core,
) {
    if out.is_null() {
        return;
    }
    unsafe { *out = read_core(png_ptr) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_read_core_set(
    png_ptr: png_structrp,
    input: *const png_safe_read_core,
) {
    if input.is_null() {
        return;
    }
    unsafe { write_core(png_ptr, &*input) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_info_core_get(
    info_ptr: png_const_inforp,
    out: *mut png_safe_info_core,
) {
    if out.is_null() {
        return;
    }
    unsafe { *out = read_info_core(info_ptr) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_info_core_set(
    info_ptr: png_inforp,
    input: *const png_safe_info_core,
) {
    if input.is_null() {
        return;
    }
    unsafe { write_info_core(info_ptr, &*input) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_parse_snapshot_capture(
    _png_ptr: png_const_structrp,
    _info_ptr: png_const_inforp,
) -> *mut core::ffi::c_void {
    ptr::null_mut()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_parse_snapshot_restore(
    _png_ptr: png_structrp,
    _info_ptr: png_inforp,
    _snapshot: *const core::ffi::c_void,
) {
}

fn refresh_text_cache(info_state: &mut state::PngInfoState) {
    info_state.text_cache.clear();
    info_state.text_key_storage.clear();
    info_state.text_value_storage.clear();
    info_state.text_lang_storage.clear();
    info_state.text_lang_key_storage.clear();

    for chunk in &info_state.text_chunks {
        let key_bytes = owned_cstring_bytes(string_to_latin1_bytes_lossy(&chunk.keyword));
        let value_bytes = match chunk.compression {
            PNG_ITXT_COMPRESSION_NONE | PNG_ITXT_COMPRESSION_ZTXT => {
                owned_cstring_bytes(chunk.text.as_bytes().to_vec())
            }
            _ => owned_cstring_bytes(string_to_latin1_bytes_lossy(&chunk.text)),
        };
        let lang_bytes = if chunk.language_tag.is_empty() {
            Vec::new()
        } else {
            owned_cstring_bytes(chunk.language_tag.as_bytes().to_vec())
        };
        let lang_key_bytes = if chunk.translated_keyword.is_empty() {
            Vec::new()
        } else {
            owned_cstring_bytes(chunk.translated_keyword.as_bytes().to_vec())
        };

        info_state.text_key_storage.push(key_bytes);
        info_state.text_value_storage.push(value_bytes);
        info_state.text_lang_storage.push(lang_bytes);
        info_state.text_lang_key_storage.push(lang_key_bytes);

        let index = info_state.text_cache.len();
        info_state.text_cache.push(png_text {
            compression: chunk.compression,
            key: info_state.text_key_storage[index].as_mut_ptr().cast(),
            text: info_state.text_value_storage[index].as_mut_ptr().cast(),
            text_length: info_state.text_value_storage[index].len().saturating_sub(1),
            itxt_length: match chunk.compression {
                PNG_ITXT_COMPRESSION_NONE | PNG_ITXT_COMPRESSION_ZTXT => {
                    info_state.text_value_storage[index].len().saturating_sub(1)
                }
                _ => 0,
            },
            lang: if info_state.text_lang_storage[index].is_empty() {
                core::ptr::null_mut()
            } else {
                info_state.text_lang_storage[index].as_mut_ptr().cast()
            },
            lang_key: if info_state.text_lang_key_storage[index].is_empty() {
                core::ptr::null_mut()
            } else {
                info_state.text_lang_key_storage[index].as_mut_ptr().cast()
            },
        });
    }
}

unsafe fn sync_png_info_aliases_impl(info_ptr: png_inforp) {
    if info_ptr.is_null() {
        return;
    }

    state::with_info_mut(info_ptr, |info_state| {
        refresh_text_cache(info_state);

        let alias = unsafe { &mut *info_ptr.cast::<png_info_alias_text_prefix>() };
        alias.width = info_state.core.width;
        alias.height = info_state.core.height;
        alias.valid = info_state.core.valid;
        alias.rowbytes = derive_info_rowbytes(&info_state.core);
        alias.palette = if info_state.palette.is_empty() {
            core::ptr::null_mut()
        } else {
            info_state.palette.as_mut_ptr()
        };
        alias.num_palette = info_state.core.num_palette;
        alias.num_trans = info_state.core.num_trans;
        alias.bit_depth = info_state.core.bit_depth;
        alias.color_type = info_state.core.color_type;
        alias.compression_type = info_state.core.compression_type;
        alias.filter_type = info_state.core.filter_type;
        alias.interlace_type = info_state.core.interlace_type;
        alias.channels = info_state.core.channels;
        alias.pixel_depth = info_state.core.pixel_depth;
        alias.spare_byte = 0;
        alias.signature = [0; 8];
        alias.colorspace = info_state.core.colorspace;
        alias.iccp_name = if info_state.iccp_name.is_empty() {
            core::ptr::null_mut()
        } else {
            info_state.iccp_name.as_mut_ptr().cast()
        };
        alias.iccp_profile = if info_state.iccp_profile.is_empty() {
            core::ptr::null_mut()
        } else {
            info_state.iccp_profile.as_mut_ptr()
        };
        alias.iccp_proflen = png_uint_32::try_from(info_state.iccp_profile.len()).unwrap_or(0);
        alias.num_text = c_int::try_from(info_state.text_chunks.len()).unwrap_or(c_int::MAX);
        alias.max_text = alias.num_text;
        alias.text = if info_state.text_cache.is_empty() {
            core::ptr::null_mut()
        } else {
            info_state.text_cache.as_mut_ptr()
        };
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_parse_snapshot_free(_snapshot: *mut core::ffi::c_void) {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_sync_png_info_aliases(
    _png_ptr: png_structrp,
    info_ptr: png_const_inforp,
) {
    unsafe { sync_png_info_aliases_impl(info_ptr.cast_mut()) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_valid(
    _png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
    flag: png_uint_32,
) -> png_uint_32 {
    info_valid(info_ptr, flag)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_rowbytes(
    _png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> usize {
    let rowbytes = derive_info_rowbytes(&read_info_core(info_ptr));
    if rowbytes != 0 {
        state::update_info(info_ptr.cast_mut(), |info_state| {
            info_state.core.rowbytes = rowbytes;
        });
    }
    rowbytes
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_rows(
    _png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_bytepp {
    read_info_core(info_ptr).row_pointers
}

macro_rules! info_field_getter {
    ($name:ident, $field:ident, $ty:ty) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(
            _png_ptr: png_const_structrp,
            info_ptr: png_const_inforp,
        ) -> $ty {
            read_info_core(info_ptr).$field
        }
    };
}

info_field_getter!(bridge_png_get_image_width, width, png_uint_32);
info_field_getter!(bridge_png_get_image_height, height, png_uint_32);
info_field_getter!(bridge_png_get_bit_depth, bit_depth, png_byte);
info_field_getter!(bridge_png_get_color_type, color_type, png_byte);
info_field_getter!(bridge_png_get_filter_type, filter_type, png_byte);
info_field_getter!(bridge_png_get_interlace_type, interlace_type, png_byte);
info_field_getter!(bridge_png_get_compression_type, compression_type, png_byte);
info_field_getter!(bridge_png_get_channels, channels, png_byte);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_user_width_max(png_ptr: png_const_structrp) -> png_uint_32 {
    state::with_png(png_ptr.cast_mut(), |png_state| png_state.user_width_max).unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_user_height_max(
    png_ptr: png_const_structrp,
) -> png_uint_32 {
    state::with_png(png_ptr.cast_mut(), |png_state| png_state.user_height_max).unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_chunk_cache_max(
    png_ptr: png_const_structrp,
) -> png_uint_32 {
    state::with_png(png_ptr.cast_mut(), |png_state| png_state.user_chunk_cache_max).unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_chunk_malloc_max(
    png_ptr: png_const_structrp,
) -> png_alloc_size_t {
    state::with_png(png_ptr.cast_mut(), |png_state| png_state.user_chunk_malloc_max).unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_bKGD(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    background: *mut png_color_16p,
) -> png_uint_32 {
    state::with_info_mut(info_ptr, |info_state| {
        if (info_state.core.valid & PNG_INFO_bKGD) == 0 {
            return 0;
        }
        if !background.is_null() {
            unsafe { *background = &mut info_state.core.background };
        }
        PNG_INFO_bKGD
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_cHRM(
    _png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
    white_x: png_doublep,
    white_y: png_doublep,
    red_x: png_doublep,
    red_y: png_doublep,
    green_x: png_doublep,
    green_y: png_doublep,
    blue_x: png_doublep,
    blue_y: png_doublep,
) -> png_uint_32 {
    let core = state::get_info(info_ptr.cast_mut())
        .map(|info| info.core)
        .unwrap_or_else(|| read_info_core(info_ptr));
    if (core.valid & PNG_INFO_cHRM) == 0 {
        return 0;
    }
    unsafe {
        if !white_x.is_null() {
            *white_x = double_from_fixed(core.colorspace.end_points_xy.whitex);
        }
        if !white_y.is_null() {
            *white_y = double_from_fixed(core.colorspace.end_points_xy.whitey);
        }
        if !red_x.is_null() {
            *red_x = double_from_fixed(core.colorspace.end_points_xy.redx);
        }
        if !red_y.is_null() {
            *red_y = double_from_fixed(core.colorspace.end_points_xy.redy);
        }
        if !green_x.is_null() {
            *green_x = double_from_fixed(core.colorspace.end_points_xy.greenx);
        }
        if !green_y.is_null() {
            *green_y = double_from_fixed(core.colorspace.end_points_xy.greeny);
        }
        if !blue_x.is_null() {
            *blue_x = double_from_fixed(core.colorspace.end_points_xy.bluex);
        }
        if !blue_y.is_null() {
            *blue_y = double_from_fixed(core.colorspace.end_points_xy.bluey);
        }
    }
    PNG_INFO_cHRM
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_cHRM_fixed(
    _png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
    white_x: png_fixed_point_p,
    white_y: png_fixed_point_p,
    red_x: png_fixed_point_p,
    red_y: png_fixed_point_p,
    green_x: png_fixed_point_p,
    green_y: png_fixed_point_p,
    blue_x: png_fixed_point_p,
    blue_y: png_fixed_point_p,
) -> png_uint_32 {
    let core = state::get_info(info_ptr.cast_mut())
        .map(|info| info.core)
        .unwrap_or_else(|| read_info_core(info_ptr));
    if (core.valid & PNG_INFO_cHRM) == 0 {
        return 0;
    }
    unsafe {
        if !white_x.is_null() {
            *white_x = core.colorspace.end_points_xy.whitex;
        }
        if !white_y.is_null() {
            *white_y = core.colorspace.end_points_xy.whitey;
        }
        if !red_x.is_null() {
            *red_x = core.colorspace.end_points_xy.redx;
        }
        if !red_y.is_null() {
            *red_y = core.colorspace.end_points_xy.redy;
        }
        if !green_x.is_null() {
            *green_x = core.colorspace.end_points_xy.greenx;
        }
        if !green_y.is_null() {
            *green_y = core.colorspace.end_points_xy.greeny;
        }
        if !blue_x.is_null() {
            *blue_x = core.colorspace.end_points_xy.bluex;
        }
        if !blue_y.is_null() {
            *blue_y = core.colorspace.end_points_xy.bluey;
        }
    }
    PNG_INFO_cHRM
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_eXIf(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    exif: *mut png_bytep,
) -> png_uint_32 {
    state::with_info_mut(info_ptr, |info_state| {
        if (info_state.core.valid & PNG_INFO_eXIf) == 0 || info_state.exif.is_empty() {
            return 0;
        }
        if !exif.is_null() {
            unsafe { *exif = info_state.exif.as_mut_ptr() };
        }
        PNG_INFO_eXIf
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_eXIf_1(
    _png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
    num_exif: *mut png_uint_32,
    exif: *mut png_bytep,
) -> png_uint_32 {
    state::with_info_mut(info_ptr.cast_mut(), |info_state| {
        if (info_state.core.valid & PNG_INFO_eXIf) == 0 || info_state.exif.is_empty() {
            return 0;
        }
        if !num_exif.is_null() {
            unsafe { *num_exif = info_state.exif.len() as png_uint_32 };
        }
        if !exif.is_null() {
            unsafe { *exif = info_state.exif.as_mut_ptr() };
        }
        PNG_INFO_eXIf
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_gAMA(
    _png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
    file_gamma: png_doublep,
) -> png_uint_32 {
    let core = read_info_core(info_ptr);
    if (core.valid & PNG_INFO_gAMA) == 0 {
        return 0;
    }
    if !file_gamma.is_null() {
        unsafe { *file_gamma = double_from_fixed(core.colorspace.gamma) };
    }
    PNG_INFO_gAMA
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_gAMA_fixed(
    _png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
    file_gamma: png_fixed_point_p,
) -> png_uint_32 {
    let core = read_info_core(info_ptr);
    if (core.valid & PNG_INFO_gAMA) == 0 {
        return 0;
    }
    if !file_gamma.is_null() {
        unsafe { *file_gamma = core.colorspace.gamma };
    }
    PNG_INFO_gAMA
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_hIST(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    hist: *mut png_uint_16p,
) -> png_uint_32 {
    state::with_info_mut(info_ptr, |info_state| {
        if (info_state.core.valid & PNG_INFO_hIST) == 0 || info_state.hist.is_empty() {
            return 0;
        }
        if !hist.is_null() {
            unsafe { *hist = info_state.hist.as_mut_ptr() };
        }
        PNG_INFO_hIST
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_IHDR(
    _png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
    width: *mut png_uint_32,
    height: *mut png_uint_32,
    bit_depth: *mut c_int,
    color_type: *mut c_int,
    interlace_method: *mut c_int,
    compression_method: *mut c_int,
    filter_method: *mut c_int,
) -> png_uint_32 {
    let core = read_info_core(info_ptr);
    unsafe {
        if !width.is_null() {
            *width = core.width;
        }
        if !height.is_null() {
            *height = core.height;
        }
        if !bit_depth.is_null() {
            *bit_depth = c_int::from(core.bit_depth);
        }
        if !color_type.is_null() {
            *color_type = c_int::from(core.color_type);
        }
        if !interlace_method.is_null() {
            *interlace_method = c_int::from(core.interlace_type);
        }
        if !compression_method.is_null() {
            *compression_method = c_int::from(core.compression_type);
        }
        if !filter_method.is_null() {
            *filter_method = c_int::from(core.filter_type);
        }
    }
    if core.width != 0 && core.height != 0 { 1 } else { 0 }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_oFFs(
    _png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
    offset_x: *mut png_int_32,
    offset_y: *mut png_int_32,
    unit_type: *mut c_int,
) -> png_uint_32 {
    state::with_info(info_ptr.cast_mut(), |info_state| info_state.offs)
        .flatten()
        .map(|(x, y, unit)| unsafe {
            if !offset_x.is_null() {
                *offset_x = x;
            }
            if !offset_y.is_null() {
                *offset_y = y;
            }
            if !unit_type.is_null() {
                *unit_type = unit;
            }
            PNG_INFO_oFFs
        })
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_pCAL(
    _png_ptr: png_const_structrp,
    _info_ptr: png_inforp,
    _purpose: *mut png_charp,
    _x0: *mut png_int_32,
    _x1: *mut png_int_32,
    _kind: *mut c_int,
    _nparams: *mut c_int,
    _units: *mut png_charp,
    _params: *mut png_charpp,
) -> png_uint_32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_pHYs(
    _png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
    res_x: *mut png_uint_32,
    res_y: *mut png_uint_32,
    unit_type: *mut c_int,
) -> png_uint_32 {
    state::with_info(info_ptr.cast_mut(), |info_state| info_state.phys)
        .flatten()
        .map(|(x, y, unit)| unsafe {
            if !res_x.is_null() {
                *res_x = x;
            }
            if !res_y.is_null() {
                *res_y = y;
            }
            if !unit_type.is_null() {
                *unit_type = unit;
            }
            PNG_INFO_pHYs
        })
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_PLTE(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    palette: *mut png_colorp,
    num_palette: *mut c_int,
) -> png_uint_32 {
    state::with_info_mut(info_ptr, |info_state| {
        if (info_state.core.valid & PNG_INFO_PLTE) == 0 || info_state.palette.is_empty() {
            return 0;
        }
        if !palette.is_null() {
            unsafe { *palette = info_state.palette.as_mut_ptr() };
        }
        if !num_palette.is_null() {
            unsafe { *num_palette = info_state.palette.len() as c_int };
        }
        PNG_INFO_PLTE
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_sBIT(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    sig_bit: *mut png_color_8p,
) -> png_uint_32 {
    state::with_info_mut(info_ptr, |info_state| {
        if (info_state.core.valid & PNG_INFO_sBIT) == 0 {
            return 0;
        }
        if !sig_bit.is_null() {
            unsafe { *sig_bit = &mut info_state.core.sig_bit };
        }
        PNG_INFO_sBIT
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_sRGB(
    _png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
    file_srgb_intent: *mut c_int,
) -> png_uint_32 {
    let core = read_info_core(info_ptr);
    if (core.valid & PNG_INFO_sRGB) == 0 {
        return 0;
    }
    if !file_srgb_intent.is_null() {
        unsafe { *file_srgb_intent = c_int::from(core.colorspace.rendering_intent) };
    }
    PNG_INFO_sRGB
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_iCCP(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    name: *mut png_charp,
    compression_type: *mut c_int,
    profile: *mut png_bytep,
    proflen: *mut png_uint_32,
) -> png_uint_32 {
    state::with_info_mut(info_ptr, |info_state| {
        if (info_state.core.valid & PNG_INFO_iCCP) == 0 || info_state.iccp_profile.is_empty() {
            return 0;
        }
        if !name.is_null() {
            unsafe { *name = info_state.iccp_name.as_mut_ptr().cast() };
        }
        if !compression_type.is_null() {
            unsafe { *compression_type = 0 };
        }
        if !profile.is_null() {
            unsafe { *profile = info_state.iccp_profile.as_mut_ptr() };
        }
        if !proflen.is_null() {
            unsafe { *proflen = info_state.iccp_profile.len() as png_uint_32 };
        }
        PNG_INFO_iCCP
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_sPLT(
    _png_ptr: png_const_structrp,
    _info_ptr: png_inforp,
    _entries: png_sPLT_tpp,
) -> c_int {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_text(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    text_ptr: *mut png_textp,
    num_text: *mut c_int,
) -> c_int {
    unsafe { sync_png_info_aliases_impl(info_ptr) };
    if info_ptr.is_null() {
        if !num_text.is_null() {
            unsafe { *num_text = 0 };
        }
        if !text_ptr.is_null() {
            unsafe { *text_ptr = core::ptr::null_mut() };
        }
        return 0;
    }

    let alias = unsafe { &mut *info_ptr.cast::<png_info_alias_text_prefix>() };
    if !num_text.is_null() {
        unsafe { *num_text = alias.num_text };
    }
    if !text_ptr.is_null() {
        unsafe { *text_ptr = alias.text };
    }
    alias.num_text
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_tIME(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    mod_time: *mut png_timep,
) -> png_uint_32 {
    state::with_info_mut(info_ptr, |info_state| {
        let Some(time) = info_state.time.as_mut() else {
            return 0;
        };
        if !mod_time.is_null() {
            unsafe { *mod_time = time };
        }
        PNG_INFO_tIME
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_tRNS(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    trans_alpha: *mut png_bytep,
    num_trans: *mut c_int,
    trans_color: *mut png_color_16p,
) -> png_uint_32 {
    state::with_info_mut(info_ptr, |info_state| {
        if (info_state.core.valid & PNG_INFO_tRNS) == 0 {
            return 0;
        }
        if info_state.core.color_type == 3 {
            if trans_alpha.is_null() && num_trans.is_null() {
                return 0;
            }
            if !trans_alpha.is_null() {
                unsafe {
                    *trans_alpha = if info_state.trans_alpha.is_empty() {
                        ptr::null_mut()
                    } else {
                        info_state.trans_alpha.as_mut_ptr()
                    };
                }
            }
            if !num_trans.is_null() {
                unsafe { *num_trans = info_state.trans_alpha.len() as c_int };
            }
            if !trans_color.is_null() {
                unsafe { *trans_color = ptr::null_mut() };
            }
        } else {
            if !trans_alpha.is_null() {
                unsafe { *trans_alpha = ptr::null_mut() };
            }
            if !num_trans.is_null() {
                unsafe { *num_trans = 1 };
            }
            if !trans_color.is_null() {
                *info_state.trns_color_cache = info_state.core.trans_color;
                unsafe { *trans_color = &mut *info_state.trns_color_cache };
            }
        }
        PNG_INFO_tRNS
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_sCAL(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
    unit: *mut c_int,
    width: png_doublep,
    height: png_doublep,
) -> png_uint_32 {
    state::with_info(info_ptr.cast_mut(), |info_state| {
        if png_ptr.is_null() || !has_scal(info_state) {
            return 0;
        }
        if !unit.is_null() {
            unsafe { *unit = info_state.scal_unit };
        }
        if !width.is_null() {
            unsafe {
                *width = libc::atof(info_state.scal_width.as_ptr().cast());
            }
        }
        if !height.is_null() {
            unsafe {
                *height = libc::atof(info_state.scal_height.as_ptr().cast());
            }
        }
        PNG_INFO_sCAL
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_sCAL_fixed(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
    unit: *mut c_int,
    width: png_fixed_point_p,
    height: png_fixed_point_p,
) -> png_uint_32 {
    state::with_info(info_ptr.cast_mut(), |info_state| {
        if png_ptr.is_null() || !has_scal(info_state) {
            return 0;
        }
        if !unit.is_null() {
            unsafe { *unit = info_state.scal_unit };
        }
        if !width.is_null() {
            unsafe {
                *width = fixed_from_double(libc::atof(info_state.scal_width.as_ptr().cast()));
            }
        }
        if !height.is_null() {
            unsafe {
                *height = fixed_from_double(libc::atof(info_state.scal_height.as_ptr().cast()));
            }
        }
        PNG_INFO_sCAL
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_sCAL_s(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
    unit: *mut c_int,
    swidth: *mut png_charp,
    sheight: *mut png_charp,
) -> png_uint_32 {
    state::with_info_mut(info_ptr.cast_mut(), |info_state| {
        if png_ptr.is_null() || !has_scal(info_state) {
            return 0;
        }
        if !unit.is_null() {
            unsafe { *unit = info_state.scal_unit };
        }
        if !swidth.is_null() {
            unsafe { *swidth = info_state.scal_width.as_mut_ptr().cast() };
        }
        if !sheight.is_null() {
            unsafe { *sheight = info_state.scal_height.as_mut_ptr().cast() };
        }
        PNG_INFO_sCAL
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_sig_bytes(png_ptr: png_structrp, num_bytes: c_int) {
    state::set_sig_bytes(png_ptr, num_bytes);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_rows(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    row_pointers: png_bytepp,
) {
    state::update_info(info_ptr, |info_state| {
        info_state.core.row_pointers = row_pointers;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_user_limits(
    png_ptr: png_structrp,
    user_width_max: png_uint_32,
    user_height_max: png_uint_32,
) {
    state::update_png(png_ptr, |png_state| {
        png_state.user_width_max = user_width_max;
        png_state.user_height_max = user_height_max;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_chunk_cache_max(
    png_ptr: png_structrp,
    user_chunk_cache_max: png_uint_32,
) {
    state::update_png(png_ptr, |png_state| {
        png_state.user_chunk_cache_max = user_chunk_cache_max;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_chunk_malloc_max(
    png_ptr: png_structrp,
    user_chunk_malloc_max: png_alloc_size_t,
) {
    state::update_png(png_ptr, |png_state| {
        png_state.user_chunk_malloc_max = user_chunk_malloc_max;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_benign_errors(png_ptr: png_structrp, allowed: c_int) {
    state::update_png(png_ptr, |png_state| {
        png_state.benign_errors = if allowed != 0 { 1 } else { 0 };
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_option(
    png_ptr: png_structrp,
    option: c_int,
    onoff: c_int,
) -> c_int {
    if png_ptr.is_null() || option < 0 || option >= PNG_OPTION_NEXT || (option & 1) != 0 {
        return PNG_OPTION_INVALID;
    }

    state::with_png_mut(png_ptr, |png_state| {
        let mask = 3u32 << option;
        let setting = (2u32 + u32::from(onoff != 0)) << option;
        let current = png_state.options;
        png_state.options = (current & !mask) | setting;
        ((current & mask) >> option) as c_int
    })
    .unwrap_or(PNG_OPTION_INVALID)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_bKGD(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    background: png_const_color_16p,
) {
    if background.is_null() {
        return;
    }
    state::update_info(info_ptr, |info_state| {
        info_state.core.background = unsafe { *background };
        info_state.core.valid |= PNG_INFO_bKGD;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_cHRM(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    white_x: f64,
    white_y: f64,
    red_x: f64,
    red_y: f64,
    green_x: f64,
    green_y: f64,
    blue_x: f64,
    blue_y: f64,
) {
    unsafe {
        bridge_png_set_cHRM_fixed(
            ptr::null(),
            info_ptr,
            fixed_from_double(white_x),
            fixed_from_double(white_y),
            fixed_from_double(red_x),
            fixed_from_double(red_y),
            fixed_from_double(green_x),
            fixed_from_double(green_y),
            fixed_from_double(blue_x),
            fixed_from_double(blue_y),
        );
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_cHRM_fixed(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    white_x: png_fixed_point,
    white_y: png_fixed_point,
    red_x: png_fixed_point,
    red_y: png_fixed_point,
    green_x: png_fixed_point,
    green_y: png_fixed_point,
    blue_x: png_fixed_point,
    blue_y: png_fixed_point,
) {
    state::update_info(info_ptr, |info_state| {
        let xy = png_xy {
            redx: red_x,
            redy: red_y,
            greenx: green_x,
            greeny: green_y,
            bluex: blue_x,
            bluey: blue_y,
            whitex: white_x,
            whitey: white_y,
        };

        info_state.core.colorspace.end_points_xy = xy;
        if let Some(xyz) = xyz_from_chrm_xy(xy) {
            info_state.core.colorspace.end_points_XYZ = xyz;
            info_state.core.colorspace.flags &= !PNG_COLORSPACE_INVALID;
            info_state.core.colorspace.flags |=
                PNG_COLORSPACE_HAVE_ENDPOINTS | PNG_COLORSPACE_FROM_CHRM;
            info_state.core.valid |= PNG_INFO_cHRM;
        } else {
            info_state.core.colorspace.end_points_XYZ = png_XYZ::default();
            info_state.core.colorspace.flags |= PNG_COLORSPACE_INVALID;
            info_state.core.colorspace.flags &= !PNG_COLORSPACE_HAVE_ENDPOINTS;
            info_state.core.valid &= !PNG_INFO_cHRM;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_eXIf(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    exif: png_bytep,
) {
    if exif.is_null() {
        return;
    }
    state::update_info(info_ptr, |info_state| {
        info_state.exif = unsafe { core::slice::from_raw_parts(exif, info_state.exif.len()) }.to_vec();
        info_state.core.valid |= PNG_INFO_eXIf;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_eXIf_1(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    num_exif: png_uint_32,
    exif: png_bytep,
) {
    if exif.is_null() {
        return;
    }
    state::update_info(info_ptr, |info_state| {
        info_state.exif = unsafe { core::slice::from_raw_parts(exif, num_exif as usize) }.to_vec();
        info_state.core.valid |= PNG_INFO_eXIf;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_gAMA(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    file_gamma: f64,
) {
    unsafe { bridge_png_set_gAMA_fixed(ptr::null(), info_ptr, fixed_from_double(file_gamma)) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_gAMA_fixed(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    file_gamma: png_fixed_point,
) {
    state::update_info(info_ptr, |info_state| {
        info_state.core.colorspace.gamma = file_gamma;
        info_state.core.colorspace.flags &= !PNG_COLORSPACE_INVALID;
        info_state.core.colorspace.flags |= PNG_COLORSPACE_HAVE_GAMMA;
        info_state.core.valid |= PNG_INFO_gAMA;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_hIST(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    hist: png_const_uint_16p,
) {
    if hist.is_null() {
        return;
    }
    let len = read_info_core(info_ptr).num_palette as usize;
    state::update_info(info_ptr, |info_state| {
        info_state.hist = unsafe { core::slice::from_raw_parts(hist, len) }.to_vec();
        info_state.core.valid |= PNG_INFO_hIST;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_IHDR(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    width: png_uint_32,
    height: png_uint_32,
    bit_depth: c_int,
    color_type: c_int,
    interlace_method: c_int,
    compression_method: c_int,
    filter_method: c_int,
) {
    let channels: png_byte = match color_type {
        0 | 3 => 1,
        2 => 3,
        4 => 2,
        6 => 4,
        _ => 0,
    };
    let pixel_depth = channels.saturating_mul(bit_depth as png_byte);
    let rowbytes = usize::try_from(width)
        .ok()
        .and_then(|width| checked_rowbytes_for_width(width, usize::from(pixel_depth)))
        .unwrap_or(0);

    state::update_info(info_ptr, |info_state| {
        info_state.core.width = width;
        info_state.core.height = height;
        info_state.core.bit_depth = bit_depth as png_byte;
        info_state.core.color_type = color_type as png_byte;
        info_state.core.interlace_type = interlace_method as png_byte;
        info_state.core.compression_type = compression_method as png_byte;
        info_state.core.filter_type = filter_method as png_byte;
        info_state.core.channels = channels;
        info_state.core.pixel_depth = pixel_depth;
        info_state.core.rowbytes = rowbytes;
    });
    state::update_png(png_ptr.cast_mut(), |png_state| {
        png_state.core.width = width;
        png_state.core.height = height;
        png_state.core.bit_depth = bit_depth as png_byte;
        png_state.core.color_type = color_type as png_byte;
        png_state.core.interlaced = interlace_method as png_byte;
        png_state.core.compression_type = compression_method as png_byte;
        png_state.core.filter_type = filter_method as png_byte;
        png_state.core.channels = channels;
        png_state.core.pixel_depth = pixel_depth;
        png_state.core.info_rowbytes = rowbytes;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_oFFs(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    offset_x: png_int_32,
    offset_y: png_int_32,
    unit_type: c_int,
) {
    state::update_info(info_ptr, |info_state| {
        info_state.offs = Some((offset_x, offset_y, unit_type));
        info_state.core.valid |= PNG_INFO_oFFs;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_pCAL(
    _png_ptr: png_const_structrp,
    _info_ptr: png_inforp,
    _purpose: png_const_charp,
    _x0: png_int_32,
    _x1: png_int_32,
    _kind: c_int,
    _nparams: c_int,
    _units: png_const_charp,
    _params: png_charpp,
) {
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_pHYs(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    res_x: png_uint_32,
    res_y: png_uint_32,
    unit_type: c_int,
) {
    state::update_info(info_ptr, |info_state| {
        info_state.phys = Some((res_x, res_y, unit_type));
        info_state.core.valid |= PNG_INFO_pHYs;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_PLTE(
    _png_ptr: png_structrp,
    info_ptr: png_inforp,
    palette: png_const_colorp,
    num_palette: c_int,
) {
    if palette.is_null() || num_palette <= 0 {
        return;
    }
    state::update_info(info_ptr, |info_state| {
        info_state.palette =
            unsafe { core::slice::from_raw_parts(palette, num_palette as usize) }.to_vec();
        info_state.core.num_palette = num_palette as png_uint_16;
        info_state.core.valid |= PNG_INFO_PLTE;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_sBIT(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    sig_bit: png_const_color_8p,
) {
    if sig_bit.is_null() {
        return;
    }
    let sig_bit_value = unsafe { *sig_bit };
    state::update_info(info_ptr, |info_state| {
        info_state.core.sig_bit = sig_bit_value;
        info_state.core.valid |= PNG_INFO_sBIT;
    });
    if !png_ptr.is_null() {
        state::update_png(png_ptr.cast_mut(), |png_state| {
            png_state.core.shift = sig_bit_value;
        });
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_sRGB(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    srgb_intent: c_int,
) {
    state::update_info(info_ptr, |info_state| {
        info_state.core.colorspace.rendering_intent = srgb_intent as png_uint_16;
        info_state.core.colorspace.gamma = 220_000;
        info_state.core.colorspace.flags &= !PNG_COLORSPACE_INVALID;
        info_state.core.colorspace.flags |= PNG_COLORSPACE_HAVE_GAMMA | PNG_COLORSPACE_HAVE_INTENT;
        info_state.core.valid |= PNG_INFO_sRGB | PNG_INFO_gAMA;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_sRGB_gAMA_and_cHRM(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    srgb_intent: c_int,
) {
    unsafe { bridge_png_set_sRGB(png_ptr, info_ptr, srgb_intent) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_iCCP(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    name: png_const_charp,
    _compression_type: c_int,
    profile: png_const_bytep,
    proflen: png_uint_32,
) {
    if name.is_null() || profile.is_null() {
        return;
    }
    let name_len = unsafe { libc::strlen(name) };
    state::update_info(info_ptr, |info_state| {
        info_state.iccp_name = unsafe { core::slice::from_raw_parts(name.cast::<u8>(), name_len + 1) }.to_vec();
        info_state.iccp_profile =
            unsafe { core::slice::from_raw_parts(profile, proflen as usize) }.to_vec();
        info_state.core.valid |= PNG_INFO_iCCP;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_sPLT(
    _png_ptr: png_const_structrp,
    _info_ptr: png_inforp,
    _entries: png_const_sPLT_tp,
    _nentries: c_int,
) {
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_text(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    text_ptr: png_const_textp,
    num_text: c_int,
) {
    if info_ptr.is_null() || text_ptr.is_null() || num_text <= 0 {
        return;
    }

    let Ok(count) = usize::try_from(num_text) else {
        return;
    };
    let texts = unsafe { core::slice::from_raw_parts(text_ptr, count) };
    state::update_info(info_ptr, |info_state| {
        for text in texts {
            let keyword = if text.key.is_null() {
                String::new()
            } else {
                let len = unsafe { libc::strlen(text.key) };
                let bytes = unsafe { core::slice::from_raw_parts(text.key.cast::<u8>(), len) };
                latin1_bytes_to_string(bytes)
            };

            let content_len = match text.compression {
                PNG_ITXT_COMPRESSION_NONE | PNG_ITXT_COMPRESSION_ZTXT => {
                    if text.itxt_length != 0 {
                        text.itxt_length
                    } else if text.text_length != 0 {
                        text.text_length
                    } else if text.text.is_null() {
                        0
                    } else {
                        unsafe { libc::strlen(text.text) }
                    }
                }
                _ => {
                    if text.text_length != 0 {
                        text.text_length
                    } else if text.text.is_null() {
                        0
                    } else {
                        unsafe { libc::strlen(text.text) }
                    }
                }
            };
            let text_bytes = unsafe { text_field_bytes(text, content_len) };
            let content = match text.compression {
                PNG_ITXT_COMPRESSION_NONE | PNG_ITXT_COMPRESSION_ZTXT => {
                    String::from_utf8_lossy(&text_bytes).into_owned()
                }
                _ => latin1_bytes_to_string(&text_bytes),
            };

            let language_tag = if text.lang.is_null() {
                String::new()
            } else {
                let len = unsafe { libc::strlen(text.lang) };
                let bytes = unsafe { core::slice::from_raw_parts(text.lang.cast::<u8>(), len) };
                String::from_utf8_lossy(bytes).into_owned()
            };
            let translated_keyword = if text.lang_key.is_null() {
                String::new()
            } else {
                let len = unsafe { libc::strlen(text.lang_key) };
                let bytes = unsafe { core::slice::from_raw_parts(text.lang_key.cast::<u8>(), len) };
                String::from_utf8_lossy(bytes).into_owned()
            };

            info_state.text_chunks.push(state::OwnedTextChunk {
                compression: text.compression,
                keyword,
                text: content,
                language_tag,
                translated_keyword,
            });
        }
        info_state.core.free_me |= PNG_FREE_TEXT;
    });
    unsafe { sync_png_info_aliases_impl(info_ptr) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_tIME(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    mod_time: png_const_timep,
) {
    if mod_time.is_null() {
        return;
    }
    state::update_info(info_ptr, |info_state| {
        info_state.time = Some(unsafe { *mod_time });
        info_state.core.valid |= PNG_INFO_tIME;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_tRNS(
    _png_ptr: png_structrp,
    info_ptr: png_inforp,
    trans_alpha: png_const_bytep,
    num_trans: c_int,
    trans_color: png_const_color_16p,
) {
    state::update_info(info_ptr, |info_state| {
        if !trans_alpha.is_null() && num_trans > 0 {
            info_state.trans_alpha =
                unsafe { core::slice::from_raw_parts(trans_alpha, num_trans as usize) }.to_vec();
            info_state.core.num_trans = num_trans as png_uint_16;
        } else {
            info_state.trans_alpha.clear();
            info_state.core.num_trans = 0;
        }
        if !trans_color.is_null() {
            info_state.core.trans_color = unsafe { *trans_color };
        }
        info_state.core.valid |= PNG_INFO_tRNS;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_sCAL(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    unit: c_int,
    width: f64,
    height: f64,
) {
    if png_ptr.is_null() || info_ptr.is_null() {
        return;
    }

    if width <= 0.0 {
        unsafe {
            crate::error::png_warning(png_ptr, c"Invalid sCAL width ignored".as_ptr());
        }
        return;
    }

    if height <= 0.0 {
        unsafe {
            crate::error::png_warning(png_ptr, c"Invalid sCAL height ignored".as_ptr());
        }
        return;
    }

    state::update_info(info_ptr, |info_state| {
        store_scal(
            info_state,
            unit,
            scal_format_float(width),
            scal_format_float(height),
        );
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_sCAL_fixed(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    unit: c_int,
    width: png_fixed_point,
    height: png_fixed_point,
) {
    if png_ptr.is_null() || info_ptr.is_null() {
        return;
    }

    if width <= 0 {
        unsafe {
            crate::error::png_warning(png_ptr, c"Invalid sCAL width ignored".as_ptr());
        }
        return;
    }

    if height <= 0 {
        unsafe {
            crate::error::png_warning(png_ptr, c"Invalid sCAL height ignored".as_ptr());
        }
        return;
    }

    state::update_info(info_ptr, |info_state| {
        store_scal(
            info_state,
            unit,
            scal_format_fixed(width),
            scal_format_fixed(height),
        );
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_sCAL_s(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    unit: c_int,
    swidth: png_const_charp,
    sheight: png_const_charp,
) {
    if png_ptr.is_null() || info_ptr.is_null() {
        return;
    }

    if unit != 1 && unit != 2 {
        unsafe {
            crate::error::png_error(png_ptr, c"Invalid sCAL unit".as_ptr());
        }
    }

    if swidth.is_null() {
        unsafe {
            crate::error::png_error(png_ptr, c"Invalid sCAL width".as_ptr());
        }
    }

    if sheight.is_null() {
        unsafe {
            crate::error::png_error(png_ptr, c"Invalid sCAL height".as_ptr());
        }
    }

    let swidth_len = unsafe { libc::strlen(swidth) };
    let sheight_len = unsafe { libc::strlen(sheight) };
    let swidth_bytes = unsafe { core::slice::from_raw_parts(swidth.cast::<u8>(), swidth_len) };
    let sheight_bytes = unsafe { core::slice::from_raw_parts(sheight.cast::<u8>(), sheight_len) };

    if swidth_len == 0 || swidth_bytes[0] == b'-' || !scal_check_fp_string(swidth_bytes) {
        unsafe {
            crate::error::png_error(png_ptr, c"Invalid sCAL width".as_ptr());
        }
    }

    if sheight_len == 0 || sheight_bytes[0] == b'-' || !scal_check_fp_string(sheight_bytes) {
        unsafe {
            crate::error::png_error(png_ptr, c"Invalid sCAL height".as_ptr());
        }
    }

    state::update_info(info_ptr, |info_state| {
        store_scal(
            info_state,
            unit,
            unsafe { core::slice::from_raw_parts(swidth.cast::<u8>(), swidth_len + 1) }.to_vec(),
            unsafe { core::slice::from_raw_parts(sheight.cast::<u8>(), sheight_len + 1) }.to_vec(),
        );
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_set_unknown_chunks(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    unknowns: png_unknown_chunkp,
    num_unknowns: c_int,
) -> c_int {
    unsafe { crate::compat_exports::store_unknown_chunks_impl(png_ptr, info_ptr, unknowns, num_unknowns) };
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_set_IHDR(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    width: png_uint_32,
    height: png_uint_32,
    bit_depth: c_int,
    color_type: c_int,
    interlace_type: c_int,
    compression_type: c_int,
    filter_type: c_int,
) -> c_int {
    unsafe {
        bridge_png_set_IHDR(
            png_ptr,
            info_ptr,
            width,
            height,
            bit_depth,
            color_type,
            interlace_type,
            compression_type,
            filter_type,
        );
    }
    1
}

macro_rules! safe_setter_wrapper {
    ($(fn $name:ident($($arg:ident : $ty:ty),* $(,)?) => $bridge:ident;)+) => {
        $(
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn $name($($arg : $ty),*) -> c_int {
                unsafe { $bridge($($arg),*); }
                1
            }
        )+
    };
}

safe_setter_wrapper! {
    fn png_safe_set_PLTE(png_ptr: png_structrp, info_ptr: png_inforp, palette: png_colorp, num_palette: c_int) => bridge_png_set_PLTE;
    fn png_safe_set_tRNS(png_ptr: png_structrp, info_ptr: png_inforp, trans_alpha: png_bytep, num_trans: c_int, trans_color: png_color_16p) => bridge_png_set_tRNS;
    fn png_safe_set_bKGD(png_ptr: png_const_structrp, info_ptr: png_inforp, background: png_color_16p) => bridge_png_set_bKGD;
    fn png_safe_set_cHRM_fixed(png_ptr: png_const_structrp, info_ptr: png_inforp, white_x: png_fixed_point, white_y: png_fixed_point, red_x: png_fixed_point, red_y: png_fixed_point, green_x: png_fixed_point, green_y: png_fixed_point, blue_x: png_fixed_point, blue_y: png_fixed_point) => bridge_png_set_cHRM_fixed;
    fn png_safe_set_eXIf_1(png_ptr: png_const_structrp, info_ptr: png_inforp, num_exif: png_uint_32, exif: png_bytep) => bridge_png_set_eXIf_1;
    fn png_safe_set_gAMA_fixed(png_ptr: png_const_structrp, info_ptr: png_inforp, file_gamma: png_fixed_point) => bridge_png_set_gAMA_fixed;
    fn png_safe_set_hIST(png_ptr: png_const_structrp, info_ptr: png_inforp, hist: png_const_uint_16p) => bridge_png_set_hIST;
    fn png_safe_set_oFFs(png_ptr: png_const_structrp, info_ptr: png_inforp, offset_x: png_int_32, offset_y: png_int_32, unit_type: c_int) => bridge_png_set_oFFs;
    fn png_safe_set_pCAL(png_ptr: png_const_structrp, info_ptr: png_inforp, purpose: png_charp, x0: png_int_32, x1: png_int_32, type_: c_int, nparams: c_int, units: png_charp, params: png_charpp) => bridge_png_set_pCAL;
    fn png_safe_set_pHYs(png_ptr: png_const_structrp, info_ptr: png_inforp, res_x: png_uint_32, res_y: png_uint_32, unit_type: c_int) => bridge_png_set_pHYs;
    fn png_safe_set_sBIT(png_ptr: png_const_structrp, info_ptr: png_inforp, sig_bit: png_color_8p) => bridge_png_set_sBIT;
    fn png_safe_set_sCAL_s(png_ptr: png_const_structrp, info_ptr: png_inforp, unit: c_int, swidth: png_const_charp, sheight: png_const_charp) => bridge_png_set_sCAL_s;
    fn png_safe_set_sPLT(png_ptr: png_const_structrp, info_ptr: png_inforp, entries: png_sPLT_tp, num_entries: c_int) => bridge_png_set_sPLT;
    fn png_safe_set_sRGB(png_ptr: png_const_structrp, info_ptr: png_inforp, srgb_intent: c_int) => bridge_png_set_sRGB;
    fn png_safe_set_iCCP(png_ptr: png_const_structrp, info_ptr: png_inforp, name: png_const_charp, compression_type: c_int, profile: png_const_bytep, proflen: png_uint_32) => bridge_png_set_iCCP;
    fn png_safe_set_text(png_ptr: png_const_structrp, info_ptr: png_inforp, text_ptr: png_textp, num_text: c_int) => bridge_png_set_text;
    fn png_safe_set_tIME(png_ptr: png_const_structrp, info_ptr: png_inforp, mod_time: png_timep) => bridge_png_set_tIME;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_call_read_data(
    png_ptr: png_structrp,
    buffer: png_bytep,
    size: usize,
) -> c_int {
    let callback = state::with_png(png_ptr, |png_state| png_state.read_data_fn).flatten();
    if let Some(callback) = callback {
        unsafe { callback(png_ptr, buffer, size) };
        let progressive_short_read = state::with_png(png_ptr, |png_state| {
            png_state.progressive_state.short_read
        })
        .unwrap_or(false);
        if progressive_short_read {
            return 0;
        }
        if !buffer.is_null() && size != 0 {
            let bytes = unsafe { core::slice::from_raw_parts(buffer, size) };
            state::append_captured_read_data(png_ptr, bytes);
        }
        1
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_prepare_idat(
    png_ptr: png_structrp,
    length: png_uint_32,
) -> c_int {
    state::update_png(png_ptr, |png_state| {
        png_state.core.idat_size = length;
    });
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_complete_idat(png_ptr: png_structrp) -> c_int {
    state::update_png(png_ptr, |png_state| {
        png_state.core.flags |= PNG_FLAG_ZSTREAM_ENDED;
    });
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_call_read_start_row(png_ptr: png_structrp) -> c_int {
    state::update_png(png_ptr, |png_state| {
        png_state.core.flags |= PNG_FLAG_ROW_INIT;
        initialize_adam7_pass_state(&mut png_state.core);
    });
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_call_read_transform_info(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
) -> c_int {
    let source_info = state::with_png(png_ptr, |png_state| png_state.read_source_info.clone()).flatten();
    let core = transformed_output_core(read_core(png_ptr), source_info.as_ref());
    if !info_ptr.is_null() {
        state::update_info(info_ptr, |info_state| {
            if core.rowbytes != 0 {
                info_state.core.rowbytes = core.rowbytes;
            }
            if core.width != 0 {
                info_state.core.width = core.width;
            }
            if core.height != 0 {
                info_state.core.height = core.height;
            }
            if core.bit_depth != 0 {
                info_state.core.bit_depth = core.bit_depth;
            }
            if core.color_type != 0 || core.channels != 0 {
                info_state.core.color_type = core.color_type;
            }
            if core.channels != 0 {
                info_state.core.channels = core.channels;
            }
            if core.pixel_depth != 0 {
                info_state.core.pixel_depth = core.pixel_depth;
            }
            if let Some(source_info) = source_info.as_ref() {
                if (read_core(png_ptr).transformations & PNG_COMPOSE) != 0
                    && read_core(png_ptr).color_type == 3
                {
                    info_state.palette = background_composed_palette(source_info, &read_core(png_ptr));
                    info_state.trans_alpha.clear();
                    info_state.core.num_trans = 0;
                    info_state.core.valid &= !PNG_INFO_tRNS;
                }
            }
        });
    }
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_call_read_row(
    png_ptr: png_structrp,
    row: png_bytep,
    display_row: png_bytep,
) -> c_int {
    if !drain_idat_stream(png_ptr) {
        return 0;
    }

    let Some(image) = ensure_decoded_read_image(png_ptr) else {
        return 0;
    };
    let core_before = read_core(png_ptr);
    let rowbytes = image.rowbytes;
    let pixel_depth = usize::from(core_before.pixel_depth);
    let uses_pass_rows = uses_adam7_pass_rows(&core_before);
    let row_index = if uses_pass_rows {
        let pass = usize::try_from(core_before.pass).unwrap_or(0);
        usize::try_from(adam7_pass_y(pass, core_before.row_number)).unwrap_or(0)
    } else if core_before.interlaced != 0
        && (core_before.transformations & PNG_INTERLACE_TRANSFORM) != 0
    {
        let height = usize::try_from(core_before.height).unwrap_or(0);
        usize::try_from(core_before.pass)
            .unwrap_or(0)
            .saturating_mul(height)
            .saturating_add(usize::try_from(core_before.row_number).unwrap_or(0))
    } else {
        usize::try_from(core_before.row_number).unwrap_or(0)
    };
    let row_data = image.rows.get(row_index).and_then(|decoded_row| {
        if uses_pass_rows {
            extract_adam7_pass_row(decoded_row, &core_before).map(|(pass_row, pass_width)| {
                let row_palette_max = if core_before.color_type == 3 {
                    max_palette_index_in_row(&pass_row, core_before.bit_depth, pass_width)
                } else {
                    None
                };
                (pass_row, pass_width, row_palette_max)
            })
        } else {
            let width = usize::try_from(core_before.width).unwrap_or(0);
            let row_palette_max = if core_before.color_type == 3 {
                max_palette_index_in_row(decoded_row, core_before.bit_depth, width)
            } else {
                None
            };
            Some((decoded_row.clone(), width, row_palette_max))
        }
    });

    let (copy_bytes, copy_width, indexed_row_palette_max) = row_data.unwrap_or_else(|| {
        let width = if uses_pass_rows {
            usize::try_from(
                usize::try_from(core_before.pass)
                    .ok()
                    .and_then(|pass| Some(adam7_pass_samples(core_before.width, pass)))
                    .unwrap_or(0),
            )
            .unwrap_or(0)
        } else {
            usize::try_from(core_before.width).unwrap_or(0)
        };
        let row_len = if uses_pass_rows {
            current_adam7_pass_rowbytes(&core_before).unwrap_or(0)
        } else {
            rowbytes
        };
        (vec![0; row_len], width, None)
    });

    if !copy_bytes.is_empty() {
        let copy_len = copy_bytes.len();
        if !row.is_null() {
            copy_packed_row_preserving_padding(row, &copy_bytes, copy_len, copy_width, pixel_depth);
        }
        if !display_row.is_null() && display_row != row {
            copy_packed_row_preserving_padding(
                display_row,
                &copy_bytes,
                copy_len,
                copy_width,
                pixel_depth,
            );
        }
    }
    crate::interlace::sanitize_row_padding_for_core(&core_before, row, display_row);
    state::update_png(png_ptr, |png_state| {
        if let Some(row_palette_max) = indexed_row_palette_max
            && png_state.check_for_invalid_index > 0
            && png_state.core.num_palette_max >= 0
        {
            png_state.core.num_palette_max = png_state.core.num_palette_max.max(row_palette_max);
        }
        advance_read_row_state(&mut png_state.core);
    });
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_resume_finish_idat(_png_ptr: png_structrp) {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_progressive_buffer_read_bridge(
    png_ptr: png_structp,
    out: png_bytep,
    length: usize,
) {
    let _ = unsafe { crate::read_progressive::png_safe_rust_progressive_buffer_read(png_ptr, out, length) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_call_set_quantize(
    _png_ptr: png_structrp,
    _palette: png_colorp,
    _num_palette: c_int,
    _maximum_colors: c_int,
    _histogram: png_const_uint_16p,
    _full_quantize: c_int,
) -> c_int {
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_process_data_pause(
    png_ptr: png_structrp,
    _save: c_int,
) -> usize {
    state::with_png(png_ptr, |png_state| png_state.progressive_state.last_pause_bytes).unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_process_data_skip(png_ptr: png_structrp) -> png_uint_32 {
    unsafe {
        crate::error::png_warning(
            png_ptr,
            c"png_process_data_skip is not implemented in any current version of libpng"
                .as_ptr(),
        );
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_write_info_before_PLTE(
    png_ptr: png_structrp,
    info_ptr: png_const_inforp,
) {
    unsafe { crate::write_runtime::begin_write_info(png_ptr, info_ptr) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_write_info(
    png_ptr: png_structrp,
    info_ptr: png_const_inforp,
) {
    unsafe { crate::write_runtime::begin_write_info(png_ptr, info_ptr) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_write_row(png_ptr: png_structrp, row: png_const_bytep) {
    unsafe { crate::write_runtime::write_row(png_ptr, row) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_write_rows(
    png_ptr: png_structrp,
    row: png_bytepp,
    num_rows: png_uint_32,
) {
    unsafe { crate::write_runtime::write_rows(png_ptr, row, num_rows) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_write_image(png_ptr: png_structrp, image: png_bytepp) {
    unsafe { crate::write_runtime::write_image(png_ptr, image) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_write_end(png_ptr: png_structrp, _info_ptr: png_inforp) {
    let warn_invalid_index = state::with_png(png_ptr, |png_state| {
        png_state.core.color_type == 3
            && state::with_info(_info_ptr, |info_state| {
                png_state.core.num_palette_max >= i32::from(info_state.core.num_palette)
            })
            .unwrap_or(false)
    })
    .unwrap_or(false);
    if warn_invalid_index {
        unsafe {
            crate::error::png_benign_error(
                png_ptr,
                b"Wrote palette index exceeding num_palette\0".as_ptr().cast(),
            );
        }
    }

    if unsafe { crate::write_runtime::write_end(png_ptr, _info_ptr) } {
        return;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_write_png(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    transforms: png_uint_32,
    _params: png_voidp,
) {
    if png_ptr.is_null() || info_ptr.is_null() {
        return;
    }

    unsafe {
        bridge_png_write_info(png_ptr, info_ptr);
        let shift = state::with_info(info_ptr, |info_state| {
            ((info_state.core.valid & PNG_INFO_sBIT) != 0).then_some(info_state.core.sig_bit)
        })
        .flatten();
        state::update_png(png_ptr, |png_state| {
            if (transforms & PNG_TRANSFORM_INVERT_MONO) != 0 {
                png_state.core.transformations |= PNG_INVERT_MONO;
            }
            if (transforms & PNG_TRANSFORM_SHIFT) != 0 {
                if let Some(sig_bit) = shift {
                    png_state.core.transformations |= PNG_SHIFT;
                    png_state.core.shift = sig_bit;
                }
            }
            if (transforms & PNG_TRANSFORM_PACKING) != 0 && png_state.core.bit_depth < 8 {
                png_state.core.transformations |= PNG_PACK;
            }
            if (transforms & PNG_TRANSFORM_SWAP_ALPHA) != 0 {
                png_state.core.transformations |= PNG_SWAP_ALPHA;
            }
            if (transforms & PNG_TRANSFORM_STRIP_FILLER_AFTER) != 0 {
                if matches!(png_state.core.color_type, 0 | 2) && png_state.core.bit_depth >= 8 {
                    png_state.core.transformations |= PNG_FILLER;
                    png_state.filler = 0;
                    png_state.core.flags |= PNG_FLAG_FILLER_AFTER;
                }
            } else if (transforms & PNG_TRANSFORM_STRIP_FILLER_BEFORE) != 0 {
                if matches!(png_state.core.color_type, 0 | 2) && png_state.core.bit_depth >= 8 {
                    png_state.core.transformations |= PNG_FILLER;
                    png_state.filler = 0;
                    png_state.core.flags &= !PNG_FLAG_FILLER_AFTER;
                }
            }
            if (transforms & PNG_TRANSFORM_BGR) != 0 {
                png_state.core.transformations |= PNG_BGR;
            }
            if (transforms & PNG_TRANSFORM_SWAP_ENDIAN) != 0 && png_state.core.bit_depth == 16 {
                png_state.core.transformations |= PNG_SWAP_BYTES;
            }
            if (transforms & PNG_TRANSFORM_PACKSWAP) != 0 && png_state.core.bit_depth < 8 {
                png_state.core.transformations |= PNG_PACKSWAP;
            }
            if (transforms & PNG_TRANSFORM_INVERT_ALPHA) != 0 {
                png_state.core.transformations |= PNG_INVERT_ALPHA;
            }
        });

        bridge_png_write_end(png_ptr, info_ptr);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_flush(png_ptr: png_structrp, nrows: c_int) {
    state::update_png(png_ptr, |png_state| {
        png_state.flush_rows = nrows;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_write_flush(png_ptr: png_structrp) {
    let Some((_, _, flush_fn, _)) = io::write_callback_registration(png_ptr) else {
        return;
    };
    if let Some(callback) = flush_fn {
        unsafe { callback(png_ptr) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_compression_buffer_size(
    png_ptr: png_const_structrp,
) -> usize {
    crate::zlib::write_zlib_settings(png_ptr)
        .map(|settings| settings.buffer_size)
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_write_sig(png_ptr: png_structrp) {
    emit_write_segment(
        png_ptr,
        PNG_IO_SIGNATURE,
        0,
        &[137, 80, 78, 71, 13, 10, 26, 10],
    );
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_write_chunk(
    png_ptr: png_structrp,
    chunk_name: png_const_bytep,
    data: png_const_bytep,
    length: usize,
) {
    if passthrough_pending(png_ptr) {
        return;
    }
    if chunk_name.is_null() {
        return;
    }
    let name = unsafe { core::slice::from_raw_parts(chunk_name, 4) };
    let payload = if data.is_null() || length == 0 {
        &[][..]
    } else {
        unsafe { core::slice::from_raw_parts(data, length) }
    };
    let len = (length as u32).to_be_bytes();
    emit_write_bytes(png_ptr, &len);
    emit_write_bytes(png_ptr, name);
    emit_write_bytes(png_ptr, payload);
    let crc = crate::read_util::png_crc32([name[0], name[1], name[2], name[3]], payload).to_be_bytes();
    emit_write_bytes(png_ptr, &crc);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_write_chunk_start(
    _png_ptr: png_structrp,
    _chunk_name: png_const_bytep,
    _length: png_uint_32,
) {
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_write_chunk_data(
    _png_ptr: png_structrp,
    _data: png_const_bytep,
    _length: usize,
) {
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_write_chunk_end(_png_ptr: png_structrp) {}

macro_rules! zlib_write_setting {
    ($(fn $name:ident($png_ptr:ident : $png_ty:ty, $value:ident : $value_ty:ty) => $field:ident;)+) => {
        $(
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn $name($png_ptr: $png_ty, $value: $value_ty) {
                crate::zlib::update_write_zlib_settings($png_ptr, |settings| {
                    settings.$field = $value;
                });
            }
        )+
    };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_compression_buffer_size(
    png_ptr: png_structrp,
    size: usize,
) {
    crate::zlib::update_write_zlib_settings(png_ptr, |settings| {
        settings.buffer_size = size;
    });
}

zlib_write_setting! {
    fn bridge_png_set_compression_level(png_ptr: png_structrp, level: c_int) => level;
    fn bridge_png_set_compression_mem_level(png_ptr: png_structrp, mem_level: c_int) => mem_level;
    fn bridge_png_set_compression_method(png_ptr: png_structrp, method: c_int) => method;
    fn bridge_png_set_compression_strategy(png_ptr: png_structrp, strategy: c_int) => strategy;
    fn bridge_png_set_compression_window_bits(png_ptr: png_structrp, window_bits: c_int) => window_bits;
    fn bridge_png_set_text_compression_level(png_ptr: png_structrp, level: c_int) => text_level;
    fn bridge_png_set_text_compression_mem_level(png_ptr: png_structrp, mem_level: c_int) => text_mem_level;
    fn bridge_png_set_text_compression_method(png_ptr: png_structrp, method: c_int) => text_method;
    fn bridge_png_set_text_compression_strategy(png_ptr: png_structrp, strategy: c_int) => text_strategy;
    fn bridge_png_set_text_compression_window_bits(png_ptr: png_structrp, window_bits: c_int) => text_window_bits;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_filter(
    _png_ptr: png_structrp,
    _method: c_int,
    _filters: c_int,
) {
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_filter_heuristics(
    _png_ptr: png_structrp,
    _heuristic_method: c_int,
    _num_weights: c_int,
    _filter_weights: png_const_doublep,
    _filter_costs: png_const_doublep,
) {
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_filter_heuristics_fixed(
    _png_ptr: png_structrp,
    _heuristic_method: c_int,
    _num_weights: c_int,
    _filter_weights: png_const_fixed_point_p,
    _filter_costs: png_const_fixed_point_p,
) {
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_add_alpha(
    png_ptr: png_structrp,
    filler: png_uint_32,
    flags: c_int,
) {
    unsafe { bridge_png_set_filler(png_ptr, filler, flags) };
    if png_ptr.is_null() {
        return;
    }

    state::update_png(png_ptr, |png_state| {
        if (png_state.core.mode & PNG_IS_READ_STRUCT) != 0 {
            png_state.core.transformations |= PNG_ADD_ALPHA;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_filler(
    png_ptr: png_structrp,
    filler: png_uint_32,
    flags: c_int,
) {
    if png_ptr.is_null() {
        return;
    }

    state::update_png(png_ptr, |png_state| {
        if (png_state.core.mode & PNG_IS_READ_STRUCT) != 0 {
            png_state.filler = filler as png_uint_16;
            png_state.core.transformations |= PNG_FILLER;
            if flags == 1 {
                png_state.core.flags |= PNG_FLAG_FILLER_AFTER;
            } else {
                png_state.core.flags &= !PNG_FLAG_FILLER_AFTER;
            }
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_packing(png_ptr: png_structrp) {
    if png_ptr.is_null() {
        return;
    }

    state::update_png(png_ptr, |png_state| {
        if png_state.core.bit_depth < 8 {
            png_state.core.transformations |= PNG_PACK;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_packswap(png_ptr: png_structrp) {
    if png_ptr.is_null() {
        return;
    }

    state::update_png(png_ptr, |png_state| {
        if png_state.core.bit_depth < 8 {
            png_state.core.transformations |= PNG_PACKSWAP;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_swap(png_ptr: png_structrp) {
    if png_ptr.is_null() {
        return;
    }

    state::update_png(png_ptr, |png_state| {
        if png_state.core.bit_depth == 16 {
            png_state.core.transformations |= PNG_SWAP_BYTES;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_image_begin_read_from_file(
    image: png_imagep,
    file_name: png_const_charp,
) -> c_int {
    unsafe { crate::simplified_runtime::begin_read_from_file(image, file_name) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_image_begin_read_from_stdio(
    image: png_imagep,
    file: *mut FILE,
) -> c_int {
    unsafe { crate::simplified_runtime::begin_read_from_stdio(image, file) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_image_begin_read_from_memory(
    image: png_imagep,
    memory: png_const_voidp,
    size: usize,
) -> c_int {
    unsafe { crate::simplified_runtime::begin_read_from_memory(image, memory, size) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_image_finish_read(
    image: png_imagep,
    _background: png_const_colorp,
    buffer: png_voidp,
    row_stride: png_int_32,
    colormap: png_voidp,
) -> c_int {
    unsafe { crate::simplified_runtime::finish_read(image, _background, buffer, row_stride, colormap) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_image_write_to_file(
    image: png_imagep,
    file_name: png_const_charp,
    convert_to_8bit: c_int,
    buffer: png_const_voidp,
    row_stride: png_int_32,
    colormap: png_const_voidp,
) -> c_int {
    unsafe {
        crate::simplified_runtime::write_to_file(
            image,
            file_name,
            convert_to_8bit,
            buffer,
            row_stride,
            colormap,
        )
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_image_write_to_stdio(
    image: png_imagep,
    file: *mut FILE,
    convert_to_8bit: c_int,
    buffer: png_const_voidp,
    row_stride: png_int_32,
    colormap: png_const_voidp,
) -> c_int {
    unsafe {
        crate::simplified_runtime::write_to_stdio(
            image,
            file,
            convert_to_8bit,
            buffer,
            row_stride,
            colormap,
        )
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_image_write_to_memory(
    image: png_imagep,
    memory: png_voidp,
    memory_bytes: *mut png_alloc_size_t,
    convert_to_8bit: c_int,
    buffer: png_const_voidp,
    row_stride: png_int_32,
    colormap: png_const_voidp,
) -> c_int {
    unsafe {
        crate::simplified_runtime::write_to_memory(
            image,
            memory,
            memory_bytes,
            convert_to_8bit,
            buffer,
            row_stride,
            colormap,
        )
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_image_free(image: png_imagep) {
    unsafe { crate::simplified_runtime::image_free(image) }
}

pub(crate) unsafe fn write_info_before_palette(png_ptr: png_structrp, info_ptr: png_const_inforp) {
    unsafe { bridge_png_write_info_before_PLTE(png_ptr, info_ptr) }
}

pub(crate) unsafe fn write_info(png_ptr: png_structrp, info_ptr: png_const_inforp) {
    unsafe { bridge_png_write_info(png_ptr, info_ptr) }
}

pub(crate) unsafe fn write_row(png_ptr: png_structrp, row: png_const_bytep) {
    unsafe { bridge_png_write_row(png_ptr, row) }
}

pub(crate) unsafe fn write_rows(png_ptr: png_structrp, row: png_bytepp, num_rows: png_uint_32) {
    unsafe { bridge_png_write_rows(png_ptr, row, num_rows) }
}

pub(crate) unsafe fn write_image(png_ptr: png_structrp, image: png_bytepp) {
    unsafe { bridge_png_write_image(png_ptr, image) }
}

pub(crate) unsafe fn write_end(png_ptr: png_structrp, info_ptr: png_inforp) {
    unsafe { bridge_png_write_end(png_ptr, info_ptr) }
}

pub(crate) unsafe fn write_png(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    transforms: png_uint_32,
    params: png_voidp,
) {
    unsafe { bridge_png_write_png(png_ptr, info_ptr, transforms, params) }
}

pub(crate) unsafe fn set_flush_rows(png_ptr: png_structrp, nrows: c_int) {
    unsafe { bridge_png_set_flush(png_ptr, nrows) }
}

pub(crate) unsafe fn flush_output(png_ptr: png_structrp) {
    unsafe { bridge_png_write_flush(png_ptr) }
}

pub(crate) unsafe fn compression_buffer_size(png_ptr: png_const_structrp) -> usize {
    unsafe { bridge_png_get_compression_buffer_size(png_ptr) }
}

pub(crate) unsafe fn write_signature(png_ptr: png_structrp) {
    unsafe { bridge_png_write_sig(png_ptr) }
}

pub(crate) unsafe fn write_chunk(
    png_ptr: png_structrp,
    chunk_name: png_const_bytep,
    data: png_const_bytep,
    length: usize,
) {
    unsafe { bridge_png_write_chunk(png_ptr, chunk_name, data, length) }
}

pub(crate) unsafe fn start_chunk(
    png_ptr: png_structrp,
    chunk_name: png_const_bytep,
    length: png_uint_32,
) {
    unsafe { bridge_png_write_chunk_start(png_ptr, chunk_name, length) }
}

pub(crate) unsafe fn write_chunk_data(png_ptr: png_structrp, data: png_const_bytep, length: usize) {
    unsafe { bridge_png_write_chunk_data(png_ptr, data, length) }
}

pub(crate) unsafe fn finish_chunk(png_ptr: png_structrp) {
    unsafe { bridge_png_write_chunk_end(png_ptr) }
}

pub(crate) unsafe fn set_compression_buffer_size(png_ptr: png_structrp, size: usize) {
    unsafe { bridge_png_set_compression_buffer_size(png_ptr, size) }
}

pub(crate) unsafe fn set_compression_level(png_ptr: png_structrp, level: c_int) {
    unsafe { bridge_png_set_compression_level(png_ptr, level) }
}

pub(crate) unsafe fn set_compression_mem_level(png_ptr: png_structrp, mem_level: c_int) {
    unsafe { bridge_png_set_compression_mem_level(png_ptr, mem_level) }
}

pub(crate) unsafe fn set_compression_method(png_ptr: png_structrp, method: c_int) {
    unsafe { bridge_png_set_compression_method(png_ptr, method) }
}

pub(crate) unsafe fn set_compression_strategy(png_ptr: png_structrp, strategy: c_int) {
    unsafe { bridge_png_set_compression_strategy(png_ptr, strategy) }
}

pub(crate) unsafe fn set_compression_window_bits(png_ptr: png_structrp, window_bits: c_int) {
    unsafe { bridge_png_set_compression_window_bits(png_ptr, window_bits) }
}

pub(crate) unsafe fn set_text_compression_level(png_ptr: png_structrp, level: c_int) {
    unsafe { bridge_png_set_text_compression_level(png_ptr, level) }
}

pub(crate) unsafe fn set_text_compression_mem_level(png_ptr: png_structrp, mem_level: c_int) {
    unsafe { bridge_png_set_text_compression_mem_level(png_ptr, mem_level) }
}

pub(crate) unsafe fn set_text_compression_method(png_ptr: png_structrp, method: c_int) {
    unsafe { bridge_png_set_text_compression_method(png_ptr, method) }
}

pub(crate) unsafe fn set_text_compression_strategy(png_ptr: png_structrp, strategy: c_int) {
    unsafe { bridge_png_set_text_compression_strategy(png_ptr, strategy) }
}

pub(crate) unsafe fn set_text_compression_window_bits(png_ptr: png_structrp, window_bits: c_int) {
    unsafe { bridge_png_set_text_compression_window_bits(png_ptr, window_bits) }
}

pub(crate) unsafe fn set_filter(png_ptr: png_structrp, method: c_int, filters: c_int) {
    unsafe { bridge_png_set_filter(png_ptr, method, filters) }
}

pub(crate) unsafe fn set_filter_heuristics(
    png_ptr: png_structrp,
    heuristic_method: c_int,
    num_weights: c_int,
    filter_weights: png_const_doublep,
    filter_costs: png_const_doublep,
) {
    unsafe {
        bridge_png_set_filter_heuristics(
            png_ptr,
            heuristic_method,
            num_weights,
            filter_weights,
            filter_costs,
        )
    }
}

pub(crate) unsafe fn set_filter_heuristics_fixed(
    png_ptr: png_structrp,
    heuristic_method: c_int,
    num_weights: c_int,
    filter_weights: png_const_fixed_point_p,
    filter_costs: png_const_fixed_point_p,
) {
    unsafe {
        bridge_png_set_filter_heuristics_fixed(
            png_ptr,
            heuristic_method,
            num_weights,
            filter_weights,
            filter_costs,
        )
    }
}

pub(crate) unsafe fn set_add_alpha(png_ptr: png_structrp, filler: png_uint_32, flags: c_int) {
    unsafe { bridge_png_set_add_alpha(png_ptr, filler, flags) }
}

pub(crate) unsafe fn set_filler(png_ptr: png_structrp, filler: png_uint_32, flags: c_int) {
    unsafe { bridge_png_set_filler(png_ptr, filler, flags) }
}

pub(crate) unsafe fn set_packing(png_ptr: png_structrp) {
    unsafe { bridge_png_set_packing(png_ptr) }
}

pub(crate) unsafe fn set_packswap(png_ptr: png_structrp) {
    unsafe { bridge_png_set_packswap(png_ptr) }
}

pub(crate) unsafe fn set_swap(png_ptr: png_structrp) {
    unsafe { bridge_png_set_swap(png_ptr) }
}

pub(crate) unsafe fn image_begin_read_from_file(
    image: png_imagep,
    file_name: png_const_charp,
) -> c_int {
    unsafe { bridge_png_image_begin_read_from_file(image, file_name) }
}

pub(crate) unsafe fn image_begin_read_from_stdio(image: png_imagep, file: *mut FILE) -> c_int {
    unsafe { bridge_png_image_begin_read_from_stdio(image, file) }
}

pub(crate) unsafe fn image_begin_read_from_memory(
    image: png_imagep,
    memory: png_const_voidp,
    size: usize,
) -> c_int {
    unsafe { bridge_png_image_begin_read_from_memory(image, memory, size) }
}

pub(crate) unsafe fn image_finish_read(
    image: png_imagep,
    background: png_const_colorp,
    buffer: png_voidp,
    row_stride: png_int_32,
    colormap: png_voidp,
) -> c_int {
    unsafe { bridge_png_image_finish_read(image, background, buffer, row_stride, colormap) }
}

pub(crate) unsafe fn image_write_to_file(
    image: png_imagep,
    file_name: png_const_charp,
    convert_to_8bit: c_int,
    buffer: png_const_voidp,
    row_stride: png_int_32,
    colormap: png_const_voidp,
) -> c_int {
    unsafe {
        bridge_png_image_write_to_file(
            image,
            file_name,
            convert_to_8bit,
            buffer,
            row_stride,
            colormap,
        )
    }
}

pub(crate) unsafe fn image_write_to_stdio(
    image: png_imagep,
    file: *mut FILE,
    convert_to_8bit: c_int,
    buffer: png_const_voidp,
    row_stride: png_int_32,
    colormap: png_const_voidp,
) -> c_int {
    unsafe {
        bridge_png_image_write_to_stdio(
            image,
            file,
            convert_to_8bit,
            buffer,
            row_stride,
            colormap,
        )
    }
}

pub(crate) unsafe fn image_write_to_memory(
    image: png_imagep,
    memory: png_voidp,
    memory_bytes: *mut png_alloc_size_t,
    convert_to_8bit: c_int,
    buffer: png_const_voidp,
    row_stride: png_int_32,
    colormap: png_const_voidp,
) -> c_int {
    unsafe {
        bridge_png_image_write_to_memory(
            image,
            memory,
            memory_bytes,
            convert_to_8bit,
            buffer,
            row_stride,
            colormap,
        )
    }
}

pub(crate) unsafe fn image_free(image: png_imagep) {
    unsafe { bridge_png_image_free(image) }
}
