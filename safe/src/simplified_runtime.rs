use crate::types::*;
use core::ffi::{c_char, c_int};
use core::ptr;
use libc::FILE;
use png::{
    BitDepth as PngBitDepth, ColorType as PngColorType, Compression, Decoder, Encoder,
    ScaledFloat, SrgbRenderingIntent, Transformations,
};
use std::collections::HashMap;
use std::ffi::CStr;
use std::fs;
use std::io::Cursor;
use std::slice;
use std::sync::OnceLock;

const PNG_FORMAT_FLAG_ALPHA: png_uint_32 = 0x01;
const PNG_FORMAT_FLAG_COLOR: png_uint_32 = 0x02;
const PNG_FORMAT_FLAG_LINEAR: png_uint_32 = 0x04;
const PNG_FORMAT_FLAG_COLORMAP: png_uint_32 = 0x08;
const PNG_FORMAT_FLAG_BGR: png_uint_32 = 0x10;
const PNG_FORMAT_FLAG_AFIRST: png_uint_32 = 0x20;

const PNG_IMAGE_FLAG_FAST: png_uint_32 = 0x02;
const PNG_IMAGE_FLAG_16BIT_sRGB: png_uint_32 = 0x04;
const PNG_DEFAULT_SRGB_GAMMA: f64 = 45_455.0 / 100_000.0;
const PNG_GAMMA_SRGB_FIXED: u32 = 220_000;
const PNG_MAX_GAMMA_8_SHIFT: usize = 5;
const PNG_MAX_GAMMA_8_SUBTABLES: usize = 1 << (8 - PNG_MAX_GAMMA_8_SHIFT);
const PNG_MAX_GAMMA_8_TABLE_LEN: usize = PNG_MAX_GAMMA_8_SUBTABLES * 256;
const DEFAULT_BACKGROUND_U8: u8 = 73;
const RGB_CUBE_VALUES: [u8; 6] = [0, 51, 102, 153, 204, 255];
const RGB_MID_VALUES: [u8; 3] = [0, 127, 255];

struct SimplifiedImageState {
    bytes: Vec<u8>,
    source_format: png_uint_32,
}

#[derive(Clone, Copy)]
struct ParsedHeader {
    width: png_uint_32,
    height: png_uint_32,
    format: png_uint_32,
    flags: png_uint_32,
    colormap_entries: png_uint_32,
}

#[derive(Clone, Copy, Debug)]
enum Transfer {
    Srgb,
    Gamma(f64),
}

#[derive(Clone, Copy, Default)]
struct CanonicalPixel {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

#[derive(Clone, Copy)]
struct LinearBackground {
    r: f64,
    g: f64,
    b: f64,
}

struct DecodedImage {
    width: usize,
    height: usize,
    line_size: usize,
    color_type: PngColorType,
    bit_depth: PngBitDepth,
    is_srgb: bool,
    file_gamma: Option<f64>,
    transfer: Transfer,
    nonlinear_encode: Transfer,
    direct_transfer: Transfer,
    direct_nonlinear_encode: Transfer,
    data: Vec<u8>,
}

fn sample_channels(format: png_uint_32) -> usize {
    ((format & (PNG_FORMAT_FLAG_COLOR | PNG_FORMAT_FLAG_ALPHA)) + 1) as usize
}

fn sample_component_size(format: png_uint_32) -> usize {
    (((format & PNG_FORMAT_FLAG_LINEAR) >> 2) + 1) as usize
}

fn pixel_channels(format: png_uint_32) -> usize {
    if (format & PNG_FORMAT_FLAG_COLORMAP) != 0 {
        1
    } else {
        sample_channels(format)
    }
}

fn pixel_component_size(format: png_uint_32) -> usize {
    if (format & PNG_FORMAT_FLAG_COLORMAP) != 0 {
        1
    } else {
        sample_component_size(format)
    }
}

fn direct_pixel_size(format: png_uint_32) -> usize {
    pixel_channels(format) * pixel_component_size(format)
}

fn direct_entry_format(format: png_uint_32) -> png_uint_32 {
    format & !PNG_FORMAT_FLAG_COLORMAP
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn srgb_to_linear(value: f64) -> f64 {
    let value = clamp01(value);
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(value: f64) -> f64 {
    let value = clamp01(value);
    if value <= 0.003_130_8 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}

fn encode_u8(value: f64) -> u8 {
    (clamp01(value) * 255.0).round() as u8
}

fn encode_u16(value: f64) -> u16 {
    (clamp01(value) * 65_535.0).round() as u16
}

fn decode_u16_be(bytes: &[u8]) -> u16 {
    u16::from_be_bytes([bytes[0], bytes[1]])
}

fn decode_u16_native(bytes: &[u8]) -> u16 {
    u16::from_ne_bytes([bytes[0], bytes[1]])
}

fn write_u16_native(dst: &mut [u8], value: u16) {
    dst[..2].copy_from_slice(&value.to_ne_bytes());
}

fn write_u16_be(dst: &mut [u8], value: u16) {
    dst[..2].copy_from_slice(&value.to_be_bytes());
}

fn luminance(pixel: CanonicalPixel) -> f64 {
    (0.2126 * pixel.r) + (0.7152 * pixel.g) + (0.0722 * pixel.b)
}

impl Transfer {
    fn to_linear(self, encoded: f64) -> f64 {
        match self {
            Self::Srgb => srgb_to_linear(encoded),
            Self::Gamma(gamma) => clamp01(encoded).powf(1.0 / gamma),
        }
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

fn clear_image_status(image: png_imagep) {
    if image.is_null() {
        return;
    }

    unsafe {
        (*image).warning_or_error = 0;
        (*image).message.fill(0);
    }
}

fn set_image_error(image: png_imagep, message: impl AsRef<str>) -> c_int {
    if image.is_null() {
        return 0;
    }

    let message = message.as_ref().as_bytes();
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
        free_simplified_image_state(image);
    }

    0
}

fn install_state(image: png_imagep, header: ParsedHeader, bytes: Vec<u8>) -> c_int {
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
            bytes,
            source_format: header.format,
        }))
        .cast();
    }

    1
}

fn read_file_bytes(file_name: png_const_charp) -> Result<Vec<u8>, String> {
    if file_name.is_null() {
        return Err("invalid argument".into());
    }

    let path = unsafe { CStr::from_ptr(file_name) };
    let path = path.to_str().map_err(|_| "invalid path".to_string())?;
    fs::read(path).map_err(|err| err.to_string())
}

fn read_stdio_bytes(file: *mut FILE) -> Result<Vec<u8>, String> {
    if file.is_null() {
        return Err("invalid argument".into());
    }

    let start = unsafe { libc::ftell(file) };
    if start < 0 {
        return Err("ftell failed".into());
    }
    if unsafe { libc::fseek(file, 0, libc::SEEK_END) } != 0 {
        return Err("fseek failed".into());
    }
    let end = unsafe { libc::ftell(file) };
    if end < start {
        return Err("ftell failed".into());
    }
    if unsafe { libc::fseek(file, start, libc::SEEK_SET) } != 0 {
        return Err("fseek failed".into());
    }

    let len = usize::try_from(end - start).map_err(|_| "input too large".to_string())?;
    let mut bytes = vec![0u8; len];
    if len != 0 {
        let read = unsafe { libc::fread(bytes.as_mut_ptr().cast(), 1, len, file) };
        if read != len {
            return Err("fread failed".into());
        }
    }

    Ok(bytes)
}

fn parse_header(bytes: &[u8]) -> Result<ParsedHeader, String> {
    let decoder = Decoder::new(Cursor::new(bytes));
    let reader = decoder.read_info().map_err(|err| err.to_string())?;
    let info = reader.info();

    let mut format = match info.color_type {
        PngColorType::Grayscale => 0,
        PngColorType::Rgb => PNG_FORMAT_FLAG_COLOR,
        PngColorType::Indexed => PNG_FORMAT_FLAG_COLOR | PNG_FORMAT_FLAG_COLORMAP,
        PngColorType::GrayscaleAlpha => PNG_FORMAT_FLAG_ALPHA,
        PngColorType::Rgba => PNG_FORMAT_FLAG_COLOR | PNG_FORMAT_FLAG_ALPHA,
    };
    if info.bit_depth == PngBitDepth::Sixteen && info.color_type != PngColorType::Indexed {
        format |= PNG_FORMAT_FLAG_LINEAR;
    }
    if info.trns.is_some() && matches!(info.color_type, PngColorType::Grayscale | PngColorType::Rgb | PngColorType::Indexed) {
        format |= PNG_FORMAT_FLAG_ALPHA;
    }

    Ok(ParsedHeader {
        width: info.width,
        height: info.height,
        format,
        flags: 0,
        colormap_entries: if info.color_type == PngColorType::Indexed {
            info.palette.as_ref().map(|palette| (palette.len() / 3) as png_uint_32).unwrap_or(0)
        } else {
            256
        },
    })
}

fn info_gamma(info: &png::Info<'_>) -> Option<f64> {
    info.gama_chunk.map(|gamma| gamma.into_value().into())
}

fn source_transfer(info: &png::Info<'_>, image_flags: png_uint_32) -> Transfer {
    if info.srgb.is_some() {
        Transfer::Srgb
    } else if let Some(gamma) = info_gamma(info) {
        Transfer::Gamma(gamma)
    } else if info.bit_depth == PngBitDepth::Sixteen && (image_flags & PNG_IMAGE_FLAG_16BIT_sRGB) == 0 {
        Transfer::Gamma(1.0)
    } else if info.bit_depth == PngBitDepth::Sixteen {
        Transfer::Srgb
    } else {
        Transfer::Srgb
    }
}

fn direct_source_transfer(info: &png::Info<'_>, image_flags: png_uint_32) -> Transfer {
    if info.srgb.is_some() {
        Transfer::Srgb
    } else if let Some(gamma) = info_gamma(info) {
        Transfer::Gamma(gamma)
    } else if info.bit_depth == PngBitDepth::Sixteen && (image_flags & PNG_IMAGE_FLAG_16BIT_sRGB) == 0 {
        Transfer::Gamma(1.0)
    } else if info.bit_depth == PngBitDepth::Sixteen {
        Transfer::Srgb
    } else {
        Transfer::Srgb
    }
}

fn decode_png(bytes: &[u8], image_flags: png_uint_32) -> Result<DecodedImage, String> {
    let mut decoder = Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(Transformations::EXPAND | Transformations::ALPHA);
    let mut reader = decoder.read_info().map_err(|err| err.to_string())?;
    let info = reader.info();
    let transfer = source_transfer(info, image_flags);
    let direct_transfer = direct_source_transfer(info, image_flags);
    let is_srgb = info.srgb.is_some();
    let file_gamma = info_gamma(info);
    let nonlinear_encode = Transfer::Srgb;
    let direct_nonlinear_encode = if info.bit_depth == PngBitDepth::Sixteen
        && !is_srgb
        && (image_flags & PNG_IMAGE_FLAG_16BIT_sRGB) == 0
    {
        Transfer::Gamma(PNG_DEFAULT_SRGB_GAMMA)
    } else {
        Transfer::Srgb
    };
    let mut buffer = vec![
        0;
        reader
            .output_buffer_size()
            .ok_or_else(|| "output buffer too large".to_string())?
    ];
    let output = reader.next_frame(&mut buffer).map_err(|err| err.to_string())?;
    buffer.truncate(output.buffer_size());

    Ok(DecodedImage {
        width: output.width as usize,
        height: output.height as usize,
        line_size: output.line_size,
        color_type: output.color_type,
        bit_depth: output.bit_depth,
        is_srgb,
        file_gamma,
        transfer,
        nonlinear_encode,
        direct_transfer,
        direct_nonlinear_encode,
        data: buffer,
    })
}

fn decoded_pixel_with_transfer(
    decoded: &DecodedImage,
    transfer: Transfer,
    x: usize,
    y: usize,
) -> CanonicalPixel {
    let sample_bytes = if decoded.bit_depth == PngBitDepth::Sixteen { 2 } else { 1 };
    let channels = match decoded.color_type {
        PngColorType::Grayscale => 1,
        PngColorType::Rgb => 3,
        PngColorType::Indexed => 1,
        PngColorType::GrayscaleAlpha => 2,
        PngColorType::Rgba => 4,
    };
    let start = y * decoded.line_size + x * channels * sample_bytes;
    let pixel = &decoded.data[start..start + channels * sample_bytes];
    let max = if decoded.bit_depth == PngBitDepth::Sixteen {
        65_535.0
    } else {
        255.0
    };

    let sample = |offset: usize| -> u16 {
        if sample_bytes == 1 {
            u16::from(pixel[offset])
        } else {
            decode_u16_be(&pixel[offset..offset + 2])
        }
    };
    let linear = |offset: usize| transfer.to_linear(f64::from(sample(offset)) / max);
    let alpha = |offset: usize| f64::from(sample(offset)) / max;

    match decoded.color_type {
        PngColorType::Grayscale => {
            let gray = linear(0);
            CanonicalPixel { r: gray, g: gray, b: gray, a: 1.0 }
        }
        PngColorType::GrayscaleAlpha => {
            let gray = linear(0);
            CanonicalPixel { r: gray, g: gray, b: gray, a: alpha(sample_bytes) }
        }
        PngColorType::Rgb => CanonicalPixel {
            r: linear(0),
            g: linear(sample_bytes),
            b: linear(sample_bytes * 2),
            a: 1.0,
        },
        PngColorType::Rgba => CanonicalPixel {
            r: linear(0),
            g: linear(sample_bytes),
            b: linear(sample_bytes * 2),
            a: alpha(sample_bytes * 3),
        },
        PngColorType::Indexed => CanonicalPixel::default(),
    }
}

fn decoded_pixel(decoded: &DecodedImage, x: usize, y: usize) -> CanonicalPixel {
    decoded_pixel_with_transfer(decoded, decoded.transfer, x, y)
}

fn decoded_direct_pixel(decoded: &DecodedImage, x: usize, y: usize) -> CanonicalPixel {
    decoded_pixel_with_transfer(decoded, decoded.direct_transfer, x, y)
}

fn read_direct_pixel_with_transfer(
    format: png_uint_32,
    bytes: &[u8],
    nonlinear_decode: Transfer,
) -> CanonicalPixel {
    let component_size = pixel_component_size(format);
    let has_alpha = (format & PNG_FORMAT_FLAG_ALPHA) != 0;
    let has_color = (format & PNG_FORMAT_FLAG_COLOR) != 0;
    let bgr = (format & PNG_FORMAT_FLAG_BGR) != 0;
    let alpha_first = has_alpha && (format & PNG_FORMAT_FLAG_AFIRST) != 0;
    let linear = (format & PNG_FORMAT_FLAG_LINEAR) != 0;
    let mut offset = 0usize;

    let decode_component = |slice: &[u8]| -> f64 {
        if component_size == 1 {
            nonlinear_decode.to_linear(f64::from(slice[0]) / 255.0)
        } else {
            f64::from(decode_u16_native(slice)) / 65_535.0
        }
    };
    let decode_alpha = |slice: &[u8]| -> f64 {
        if component_size == 1 {
            f64::from(slice[0]) / 255.0
        } else {
            f64::from(decode_u16_native(slice)) / 65_535.0
        }
    };

    let mut pixel = CanonicalPixel { a: 1.0, ..CanonicalPixel::default() };
    if alpha_first {
        pixel.a = decode_alpha(&bytes[offset..offset + component_size]);
        offset += component_size;
    }

    if has_color {
        let first = decode_component(&bytes[offset..offset + component_size]);
        offset += component_size;
        let second = decode_component(&bytes[offset..offset + component_size]);
        offset += component_size;
        let third = decode_component(&bytes[offset..offset + component_size]);
        offset += component_size;
        if bgr {
            pixel.b = first;
            pixel.g = second;
            pixel.r = third;
        } else {
            pixel.r = first;
            pixel.g = second;
            pixel.b = third;
        }
    } else {
        let gray = decode_component(&bytes[offset..offset + component_size]);
        offset += component_size;
        pixel.r = gray;
        pixel.g = gray;
        pixel.b = gray;
    }

    if has_alpha && !alpha_first {
        pixel.a = decode_alpha(&bytes[offset..offset + component_size]);
    }

    if linear && has_alpha && pixel.a > 0.0 && pixel.a < 1.0 {
        pixel.r = clamp01(pixel.r / pixel.a);
        pixel.g = clamp01(pixel.g / pixel.a);
        pixel.b = clamp01(pixel.b / pixel.a);
    } else if linear && has_alpha && pixel.a <= 0.0 {
        pixel.r = 0.0;
        pixel.g = 0.0;
        pixel.b = 0.0;
    }

    pixel
}

fn read_direct_pixel(format: png_uint_32, bytes: &[u8]) -> CanonicalPixel {
    read_direct_pixel_with_transfer(format, bytes, Transfer::Srgb)
}

fn read_write_direct_pixel(format: png_uint_32, bytes: &[u8]) -> CanonicalPixel {
    read_direct_pixel_with_transfer(format, bytes, Transfer::Srgb)
}

fn encode_nonlinear_byte(transfer: Transfer, value: f64) -> u8 {
    match transfer {
        Transfer::Srgb => encode_u8(linear_to_srgb(value)),
        Transfer::Gamma(gamma) => (clamp01(value).powf(gamma) * 255.0) as u8,
    }
}

fn encode_nonlinear_byte_trunc(transfer: Transfer, value: f64) -> u8 {
    match transfer {
        Transfer::Srgb => (clamp01(linear_to_srgb(value)) * 255.0) as u8,
        Transfer::Gamma(gamma) => (clamp01(value).powf(gamma) * 255.0) as u8,
    }
}

fn write_direct_pixel_with_transfer(
    format: png_uint_32,
    pixel: CanonicalPixel,
    nonlinear_encode: Transfer,
    out: &mut [u8],
) {
    let component_size = pixel_component_size(format);
    let has_alpha = (format & PNG_FORMAT_FLAG_ALPHA) != 0;
    let has_color = (format & PNG_FORMAT_FLAG_COLOR) != 0;
    let bgr = (format & PNG_FORMAT_FLAG_BGR) != 0;
    let alpha_first = has_alpha && (format & PNG_FORMAT_FLAG_AFIRST) != 0;
    let linear = (format & PNG_FORMAT_FLAG_LINEAR) != 0;
    let mut offset = 0usize;

    if alpha_first {
        if component_size == 1 {
            out[offset] = encode_u8(pixel.a);
        } else {
            write_u16_native(&mut out[offset..offset + 2], encode_u16(pixel.a));
        }
        offset += component_size;
    }

    if has_color {
        let components = if linear && has_alpha {
            [pixel.r * pixel.a, pixel.g * pixel.a, pixel.b * pixel.a]
        } else {
            [pixel.r, pixel.g, pixel.b]
        };
        let ordered = if bgr {
            [components[2], components[1], components[0]]
        } else {
            components
        };
        for component in ordered {
            if component_size == 1 {
                out[offset] = encode_nonlinear_byte(nonlinear_encode, component);
            } else {
                write_u16_native(&mut out[offset..offset + 2], encode_u16(component));
            }
            offset += component_size;
        }
    } else {
        let gray = if linear && has_alpha {
            luminance(pixel) * pixel.a
        } else {
            luminance(pixel)
        };
        if component_size == 1 {
            out[offset] = encode_nonlinear_byte(nonlinear_encode, gray);
        } else {
            write_u16_native(&mut out[offset..offset + 2], encode_u16(gray));
        }
        offset += component_size;
    }

    if has_alpha && !alpha_first {
        if component_size == 1 {
            out[offset] = encode_u8(pixel.a);
        } else {
            write_u16_native(&mut out[offset..offset + 2], encode_u16(pixel.a));
        }
    }
}

fn write_direct_pixel(format: png_uint_32, pixel: CanonicalPixel, out: &mut [u8]) {
    write_direct_pixel_with_transfer(format, pixel, Transfer::Srgb, out);
}

fn encode_direct_entry(
    entry_format: png_uint_32,
    pixel: CanonicalPixel,
    nonlinear_encode: Transfer,
) -> Vec<u8> {
    let mut entry = vec![0u8; direct_pixel_size(entry_format)];
    write_direct_pixel_with_transfer(entry_format, pixel, nonlinear_encode, &mut entry);
    entry
}

fn encode_background_entry(entry_format: png_uint_32, background: png_const_colorp) -> Vec<u8> {
    let mut entry = vec![0u8; direct_pixel_size(entry_format)];
    write_direct_pixel(entry_format, background_pixel(entry_format, background), &mut entry);
    entry
}

fn buffer_background(format: png_uint_32, bytes: &[u8]) -> LinearBackground {
    let pixel = read_direct_pixel(format, bytes);
    LinearBackground {
        r: pixel.r,
        g: pixel.g,
        b: pixel.b,
    }
}

fn supplied_background(format: png_uint_32, background: png_const_colorp) -> LinearBackground {
    if background.is_null() {
        return LinearBackground {
            r: srgb_to_linear(f64::from(DEFAULT_BACKGROUND_U8) / 255.0),
            g: srgb_to_linear(f64::from(DEFAULT_BACKGROUND_U8) / 255.0),
            b: srgb_to_linear(f64::from(DEFAULT_BACKGROUND_U8) / 255.0),
        };
    }

    let background = unsafe { &*background };
    if (format & PNG_FORMAT_FLAG_COLOR) != 0 {
        LinearBackground {
            r: srgb_to_linear(f64::from(background.red) / 255.0),
            g: srgb_to_linear(f64::from(background.green) / 255.0),
            b: srgb_to_linear(f64::from(background.blue) / 255.0),
        }
    } else {
        let gray = srgb_to_linear(f64::from(background.green) / 255.0);
        LinearBackground { r: gray, g: gray, b: gray }
    }
}

fn supplied_background_rgb(background: png_const_colorp) -> LinearBackground {
    if background.is_null() {
        return LinearBackground {
            r: srgb_to_linear(f64::from(DEFAULT_BACKGROUND_U8) / 255.0),
            g: srgb_to_linear(f64::from(DEFAULT_BACKGROUND_U8) / 255.0),
            b: srgb_to_linear(f64::from(DEFAULT_BACKGROUND_U8) / 255.0),
        };
    }

    let background = unsafe { &*background };
    LinearBackground {
        r: srgb_to_linear(f64::from(background.red) / 255.0),
        g: srgb_to_linear(f64::from(background.green) / 255.0),
        b: srgb_to_linear(f64::from(background.blue) / 255.0),
    }
}

fn composite(pixel: CanonicalPixel, background: LinearBackground) -> CanonicalPixel {
    let a = clamp01(pixel.a);
    CanonicalPixel {
        r: pixel.r * a + background.r * (1.0 - a),
        g: pixel.g * a + background.g * (1.0 - a),
        b: pixel.b * a + background.b * (1.0 - a),
        a: 1.0,
    }
}

fn background_pixel(format: png_uint_32, background: png_const_colorp) -> CanonicalPixel {
    let background = supplied_background(format, background);
    CanonicalPixel {
        r: background.r,
        g: background.g,
        b: background.b,
        a: 1.0,
    }
}

fn nonlinear8_components(pixel: CanonicalPixel, nonlinear_encode: Transfer) -> (u8, u8, u8, u8) {
    (
        encode_nonlinear_byte(nonlinear_encode, pixel.r),
        encode_nonlinear_byte(nonlinear_encode, pixel.g),
        encode_nonlinear_byte(nonlinear_encode, pixel.b),
        encode_u8(pixel.a),
    )
}

fn compat_16_to_8_table(gamma_fixed: u32) -> Vec<u16> {
    let mut table = vec![u16::MAX; PNG_MAX_GAMMA_8_TABLE_LEN];
    let max = (1u32 << (16 - PNG_MAX_GAMMA_8_SHIFT)) - 1;
    let mut last = 0usize;

    for out in 0u16..255 {
        let out16 = out * 257;
        let mut bound = u32::from(gamma_correct_16bit_fixed(out16 + 128, gamma_fixed));
        bound = ((bound * max) + 32_768) / 65_535 + 1;
        let limit = bound.min((PNG_MAX_GAMMA_8_SUBTABLES << 8) as u32) as usize;
        while last < limit {
            let low = last & (0xff >> PNG_MAX_GAMMA_8_SHIFT);
            let high = last >> (8 - PNG_MAX_GAMMA_8_SHIFT);
            table[low * 256 + high] = out16;
            last += 1;
        }
    }

    table
}

fn linear_default_gamma_16_to_8_table() -> &'static [u16] {
    static TABLE: OnceLock<Vec<u16>> = OnceLock::new();
    TABLE.get_or_init(|| compat_16_to_8_table(PNG_GAMMA_SRGB_FIXED))
}

fn gamma_correct_16bit_fixed(value: u16, gamma_fixed: u32) -> u16 {
    if value == 0 || value == u16::MAX {
        return value;
    }

    let gamma = f64::from(gamma_fixed) * 0.00001;
    (65_535.0 * (f64::from(value) / 65_535.0).powf(gamma) + 0.5).floor() as u16
}

fn compat_16_to_8_lookup(table: &[u16], sample: u16) -> u16 {
    let sample = usize::from(sample);
    let low = (sample & 0xff) >> PNG_MAX_GAMMA_8_SHIFT;
    let high = sample >> 8;
    table[low * 256 + high]
}

fn encode_default_gamma_8bit_from_linear(value: f64) -> u8 {
    let sample = encode_u16(clamp01(value));
    sample_to_u8(compat_16_to_8_lookup(linear_default_gamma_16_to_8_table(), sample), 2)
}

fn compat_default_gamma_from_transfer(transfer: Transfer) -> Option<u32> {
    match transfer {
        Transfer::Gamma(gamma) => {
            let file_gamma_fixed = (gamma * 100_000.0).round() as u32;
            Some((((file_gamma_fixed as u64) * (PNG_GAMMA_SRGB_FIXED as u64) + 50_000) / 100_000) as u32)
        }
        Transfer::Srgb => None,
    }
}

fn gamma_correct_8bit(value: u8, gamma: f64) -> u8 {
    if value == 0 || value == u8::MAX {
        value
    } else {
        (255.0 * (f64::from(value) / 255.0).powf(gamma) + 0.5).floor() as u8
    }
}

fn rgb_to_gray8(r: u8, g: u8, b: u8) -> u8 {
    if r == g && g == b {
        r
    } else {
        (((6968u32 * u32::from(r) + 23_434u32 * u32::from(g) + 2366u32 * u32::from(b)) >> 15) & 0xff) as u8
    }
}

fn is_default_gamma(transfer: Transfer) -> bool {
    matches!(transfer, Transfer::Gamma(gamma) if (gamma - PNG_DEFAULT_SRGB_GAMMA).abs() < 1e-12)
}

fn source_8bit_components(pixel: &[u8], color_type: PngColorType) -> (u8, u8, u8, u8) {
    match color_type {
        PngColorType::Grayscale => {
            let gray = pixel[0];
            (gray, gray, gray, 255)
        }
        PngColorType::GrayscaleAlpha => {
            let gray = pixel[0];
            (gray, gray, gray, pixel[1])
        }
        PngColorType::Rgb => (pixel[0], pixel[1], pixel[2], 255),
        PngColorType::Rgba => (pixel[0], pixel[1], pixel[2], pixel[3]),
        PngColorType::Indexed => (0, 0, 0, 255),
    }
}

fn write_direct_nonlinear_8bit_pixel(format: png_uint_32, rgba: (u8, u8, u8, u8), out: &mut [u8]) {
    let (r, g, b, a) = rgba;
    let has_alpha = (format & PNG_FORMAT_FLAG_ALPHA) != 0;
    let has_color = (format & PNG_FORMAT_FLAG_COLOR) != 0;
    let bgr = (format & PNG_FORMAT_FLAG_BGR) != 0;
    let alpha_first = has_alpha && (format & PNG_FORMAT_FLAG_AFIRST) != 0;
    let gray = rgb_to_gray8(r, g, b);
    let (first, second, third) = if has_color {
        if bgr { (b, g, r) } else { (r, g, b) }
    } else {
        (gray, gray, gray)
    };
    let mut offset = 0usize;

    if alpha_first {
        out[offset] = a;
        offset += 1;
    }

    if has_color {
        out[offset] = first;
        out[offset + 1] = second;
        out[offset + 2] = third;
        offset += 3;
    } else {
        out[offset] = gray;
        offset += 1;
    }

    if has_alpha && !alpha_first {
        out[offset] = a;
    }
}

fn finish_direct_read_nonlinear_8bit(
    image: &png_image,
    source_format: png_uint_32,
    decoded: &DecodedImage,
    background: png_const_colorp,
    buffer: png_voidp,
    row_stride: png_int_32,
) -> bool {
    let target_format = image.format;
    let target_has_alpha = (target_format & PNG_FORMAT_FLAG_ALPHA) != 0;
    let target_has_color = (target_format & PNG_FORMAT_FLAG_COLOR) != 0;
    let target_linear = (target_format & PNG_FORMAT_FLAG_LINEAR) != 0;
    let source_has_alpha = matches!(decoded.color_type, PngColorType::GrayscaleAlpha | PngColorType::Rgba);
    let source_has_color = matches!(decoded.color_type, PngColorType::Rgb | PngColorType::Rgba);

    if decoded.bit_depth != PngBitDepth::Eight
        || (source_format & PNG_FORMAT_FLAG_LINEAR) != 0
        || target_linear
        || !background.is_null()
        || (target_format & (PNG_FORMAT_FLAG_BGR | PNG_FORMAT_FLAG_AFIRST | PNG_FORMAT_FLAG_COLORMAP)) != 0
        || source_has_color != target_has_color
        || (source_has_alpha && !target_has_alpha)
        || !(decoded.file_gamma.is_none() || decoded.is_srgb)
        || matches!(decoded.color_type, PngColorType::Indexed)
    {
        return false;
    }

    let target_pixel_size = direct_pixel_size(target_format);
    let source_channels = match decoded.color_type {
        PngColorType::Grayscale => 1,
        PngColorType::GrayscaleAlpha => 2,
        PngColorType::Rgb => 3,
        PngColorType::Rgba => 4,
        PngColorType::Indexed => 0,
    };

    for y in 0..decoded.height {
        let row = target_row(image, buffer, row_stride, y);
        let source_row = &decoded.data[y * decoded.line_size..(y + 1) * decoded.line_size];
        for x in 0..decoded.width {
            let source_start = x * source_channels;
            let source_pixel = &source_row[source_start..source_start + source_channels];
            let start = x * target_pixel_size;
            let end = start + target_pixel_size;
            write_direct_nonlinear_8bit_pixel(
                target_format,
                source_8bit_components(source_pixel, decoded.color_type),
                &mut row[start..end],
            );
        }
    }

    true
}

fn finish_direct_read_scaled_linear_8bit(
    image: &png_image,
    source_format: png_uint_32,
    decoded: &DecodedImage,
    background: png_const_colorp,
    buffer: png_voidp,
    row_stride: png_int_32,
) -> bool {
    let target_format = image.format;
    let target_has_alpha = (target_format & PNG_FORMAT_FLAG_ALPHA) != 0;
    let target_has_color = (target_format & PNG_FORMAT_FLAG_COLOR) != 0;
    let target_linear = (target_format & PNG_FORMAT_FLAG_LINEAR) != 0;
    let source_has_alpha = matches!(decoded.color_type, PngColorType::GrayscaleAlpha | PngColorType::Rgba);
    let source_has_color = matches!(decoded.color_type, PngColorType::Rgb | PngColorType::Rgba);
    let gamma = match decoded.direct_transfer {
        Transfer::Gamma(source_gamma) => PNG_DEFAULT_SRGB_GAMMA / source_gamma,
        Transfer::Srgb => 1.0,
    };

    if decoded.bit_depth != PngBitDepth::Eight
        || (source_format & PNG_FORMAT_FLAG_LINEAR) == 0
        || target_linear
        || !background.is_null()
        || (target_format & (PNG_FORMAT_FLAG_BGR | PNG_FORMAT_FLAG_AFIRST | PNG_FORMAT_FLAG_COLORMAP)) != 0
        || source_has_color != target_has_color
        || (source_has_alpha && !target_has_alpha)
        || matches!(decoded.color_type, PngColorType::Indexed)
    {
        return false;
    }

    let target_pixel_size = direct_pixel_size(target_format);
    let source_channels = match decoded.color_type {
        PngColorType::Grayscale => 1,
        PngColorType::GrayscaleAlpha => 2,
        PngColorType::Rgb => 3,
        PngColorType::Rgba => 4,
        PngColorType::Indexed => 0,
    };

    for y in 0..decoded.height {
        let row = target_row(image, buffer, row_stride, y);
        let source_row = &decoded.data[y * decoded.line_size..(y + 1) * decoded.line_size];
        for x in 0..decoded.width {
            let source_start = x * source_channels;
            let source_pixel = &source_row[source_start..source_start + source_channels];
            let (r, g, b, a) = source_8bit_components(source_pixel, decoded.color_type);
            let corrected = (
                gamma_correct_8bit(r, gamma),
                gamma_correct_8bit(g, gamma),
                gamma_correct_8bit(b, gamma),
                a,
            );
            let start = x * target_pixel_size;
            let end = start + target_pixel_size;
            write_direct_nonlinear_8bit_pixel(target_format, corrected, &mut row[start..end]);
        }
    }

    true
}

fn write_direct_raw_16bit_default_gamma_pixel(
    format: png_uint_32,
    source_has_alpha: bool,
    target_has_alpha: bool,
    source_pixel: &[u8],
    color_type: PngColorType,
    compat_table: &[u16],
    out: &mut [u8],
) {
    let has_color = (format & PNG_FORMAT_FLAG_COLOR) != 0;
    let bgr = (format & PNG_FORMAT_FLAG_BGR) != 0;
    let alpha_first = target_has_alpha && (format & PNG_FORMAT_FLAG_AFIRST) != 0;
    let mut offset = 0usize;

    let sample = |index: usize| -> u16 {
        let start = index * 2;
        decode_u16_be(&source_pixel[start..start + 2])
    };
    let encode_color = |value: u16| -> u8 {
        sample_to_u8(compat_16_to_8_lookup(compat_table, value), 2)
    };
    let encode_alpha = |value: u16| -> u8 { sample_to_u8(value, 2) };

    let alpha = if source_has_alpha {
        encode_alpha(sample(match color_type {
            PngColorType::GrayscaleAlpha => 1,
            PngColorType::Rgba => 3,
            _ => 0,
        }))
    } else {
        255
    };

    if alpha_first {
        out[offset] = alpha;
        offset += 1;
    }

    match color_type {
        PngColorType::Grayscale | PngColorType::GrayscaleAlpha => {
            let gray = encode_color(sample(0));
            out[offset] = gray;
            offset += 1;
            if has_color {
                out[offset] = gray;
                out[offset + 1] = gray;
                offset += 2;
            }
        }
        PngColorType::Rgb | PngColorType::Rgba => {
            let r = encode_color(sample(0));
            let g = encode_color(sample(1));
            let b = encode_color(sample(2));
            let ordered = if bgr { [b, g, r] } else { [r, g, b] };
            out[offset] = ordered[0];
            out[offset + 1] = ordered[1];
            out[offset + 2] = ordered[2];
            offset += 3;
        }
        PngColorType::Indexed => unreachable!(),
    }

    if target_has_alpha && !alpha_first {
        out[offset] = alpha;
    }
}

fn finish_direct_read_raw_16bit_default_gamma(
    image: &png_image,
    source_format: png_uint_32,
    decoded: &DecodedImage,
    background: png_const_colorp,
    buffer: png_voidp,
    row_stride: png_int_32,
) -> bool {
    let target_format = image.format;
    let target_has_alpha = (target_format & PNG_FORMAT_FLAG_ALPHA) != 0;
    let target_linear = (target_format & PNG_FORMAT_FLAG_LINEAR) != 0;
    let target_has_color = (target_format & PNG_FORMAT_FLAG_COLOR) != 0;
    let target_pixel_size = direct_pixel_size(target_format);
    let source_has_color = (source_format & PNG_FORMAT_FLAG_COLOR) != 0;
    let source_has_alpha = (source_format & PNG_FORMAT_FLAG_ALPHA) != 0;
    let gamma_fixed = match compat_default_gamma_from_transfer(decoded.direct_transfer) {
        Some(gamma_fixed) => gamma_fixed,
        None => return false,
    };

    if decoded.bit_depth != PngBitDepth::Sixteen
        || !background.is_null()
        || target_linear
        || !is_default_gamma(decoded.direct_nonlinear_encode)
        || (target_format & PNG_FORMAT_FLAG_COLORMAP) != 0
        || target_has_color != source_has_color
        || (source_has_alpha && !target_has_alpha)
        || matches!(decoded.color_type, PngColorType::Indexed)
    {
        return false;
    }

    let compat_table = compat_16_to_8_table(gamma_fixed);
    let source_channels = match decoded.color_type {
        PngColorType::Grayscale => 1,
        PngColorType::GrayscaleAlpha => 2,
        PngColorType::Rgb => 3,
        PngColorType::Rgba => 4,
        PngColorType::Indexed => 0,
    };

    for y in 0..decoded.height {
        let row = target_row(image, buffer, row_stride, y);
        let source_row = &decoded.data[y * decoded.line_size..(y + 1) * decoded.line_size];
        for x in 0..decoded.width {
            let source_start = x * source_channels * 2;
            let source_end = source_start + source_channels * 2;
            let start = x * target_pixel_size;
            let end = start + target_pixel_size;
            write_direct_raw_16bit_default_gamma_pixel(
                target_format,
                source_has_alpha,
                target_has_alpha,
                &source_row[source_start..source_end],
                decoded.color_type,
                &compat_table,
                &mut row[start..end],
            );
        }
    }

    true
}

fn write_direct_pixel_default_gamma_from_linear(
    format: png_uint_32,
    pixel: CanonicalPixel,
    out: &mut [u8],
) {
    let has_alpha = (format & PNG_FORMAT_FLAG_ALPHA) != 0;
    let has_color = (format & PNG_FORMAT_FLAG_COLOR) != 0;
    let bgr = (format & PNG_FORMAT_FLAG_BGR) != 0;
    let alpha_first = has_alpha && (format & PNG_FORMAT_FLAG_AFIRST) != 0;
    let mut offset = 0usize;

    if alpha_first {
        out[offset] = sample_to_u8(encode_u16(pixel.a), 2);
        offset += 1;
    }

    if has_color {
        let ordered = if bgr {
            [pixel.b, pixel.g, pixel.r]
        } else {
            [pixel.r, pixel.g, pixel.b]
        };
        for component in ordered {
            out[offset] = encode_default_gamma_8bit_from_linear(component);
            offset += 1;
        }
    } else {
        out[offset] = encode_default_gamma_8bit_from_linear(luminance(pixel));
        offset += 1;
    }

    if has_alpha && !alpha_first {
        out[offset] = sample_to_u8(encode_u16(pixel.a), 2);
    }
}

fn sample_to_u8(sample: u16, sample_bytes: usize) -> u8 {
    if sample_bytes == 1 {
        sample as u8
    } else {
        ((((u32::from(sample) * 255) + 32_895) >> 16) & 0xff) as u8
    }
}

fn decoded_direct_gray_linear_pixel(decoded: &DecodedImage, x: usize, y: usize) -> Option<CanonicalPixel> {
    if !matches!(decoded.color_type, PngColorType::Grayscale | PngColorType::GrayscaleAlpha) {
        return None;
    }

    let source = decoded_direct_pixel(decoded, x, y);
    let gray = f64::from(encode_nonlinear_byte(decoded.direct_nonlinear_encode, source.g)) / 255.0;
    let linear = clamp01(gray).powf(1.0 / PNG_DEFAULT_SRGB_GAMMA);
    Some(CanonicalPixel {
        r: linear,
        g: linear,
        b: linear,
        a: 1.0,
    })
}

fn div51(value: u8) -> u8 {
    (((u16::from(value) * 5) + 130) >> 8) as u8
}

fn rgb_cube_index(r: u8, g: u8, b: u8) -> u8 {
    6 * (6 * div51(r) + div51(g)) + div51(b)
}

fn rgb_mid_index(value: u8) -> u8 {
    if value < 64 {
        0
    } else if value < 192 {
        1
    } else {
        2
    }
}

fn copy_colormap_entries(
    image: &mut png_image,
    colormap: png_voidp,
    entry_size: usize,
    max_entries: usize,
    entries: &[Vec<u8>],
) {
    image.colormap_entries = entries.len() as png_uint_32;

    unsafe {
        let out = slice::from_raw_parts_mut(colormap.cast::<u8>(), entry_size * max_entries);
        for (index, entry) in entries.iter().take(max_entries).enumerate() {
            let start = index * entry_size;
            out[start..start + entry_size].copy_from_slice(entry);
        }
        if entries.len() < max_entries {
            let start = entries.len() * entry_size;
            out[start..].fill(0);
        }
    }
}

fn build_rgb_cube_entries(entry_format: png_uint_32) -> Vec<Vec<u8>> {
    let mut entries = Vec::with_capacity(216);
    for r in RGB_CUBE_VALUES {
        for g in RGB_CUBE_VALUES {
            for b in RGB_CUBE_VALUES {
                entries.push(encode_direct_entry(
                    entry_format,
                    srgb8_pixel(r, g, b, 255),
                    Transfer::Srgb,
                ));
            }
        }
    }
    entries
}

fn finish_gray_alpha_colormapped_read(
    image: &mut png_image,
    decoded: &DecodedImage,
    background: png_const_colorp,
    buffer: png_voidp,
    row_stride: png_int_32,
    colormap: png_voidp,
    entry_format: png_uint_32,
    entry_size: usize,
    max_entries: usize,
) -> Result<(), String> {
    if max_entries < 256 {
        return Err("png_image_finish_read[color-map]: too many colors".into());
    }

    let target_has_alpha = (entry_format & PNG_FORMAT_FLAG_ALPHA) != 0;
    let effective_background = if !target_has_alpha && (entry_format & PNG_FORMAT_FLAG_LINEAR) != 0 {
        ptr::null()
    } else {
        background
    };
    let target_is_gray = (entry_format & PNG_FORMAT_FLAG_COLOR) == 0;
    let background_is_gray = if effective_background.is_null() {
        true
    } else {
        let background = unsafe { &*effective_background };
        background.red == background.green && background.green == background.blue
    };
    let background_linear = if effective_background.is_null() {
        LinearBackground { r: 0.0, g: 0.0, b: 0.0 }
    } else if target_is_gray || background_is_gray {
        supplied_background(entry_format, effective_background)
    } else {
        supplied_background_rgb(effective_background)
    };
    let background_entry = if effective_background.is_null() {
        encode_direct_entry(
            entry_format,
            CanonicalPixel {
                r: background_linear.r,
                g: background_linear.g,
                b: background_linear.b,
                a: 1.0,
            },
            Transfer::Srgb,
        )
    } else if target_is_gray || background_is_gray {
        encode_background_entry(entry_format, effective_background)
    } else {
        encode_direct_entry(
            entry_format,
            CanonicalPixel {
                r: background_linear.r,
                g: background_linear.g,
                b: background_linear.b,
                a: 1.0,
            },
            Transfer::Srgb,
        )
    };

    let entries = if target_has_alpha {
        let mut ga_entries = Vec::with_capacity(256);
        for i in 0u16..231 {
            let gray = ((i * 256 + 115) / 231) as u8;
            ga_entries.push(encode_direct_entry(
                entry_format,
                srgb8_pixel(gray, gray, gray, 255),
                Transfer::Srgb,
            ));
        }
        ga_entries.push(encode_direct_entry(
            entry_format,
            srgb8_pixel(255, 255, 255, 0),
            Transfer::Srgb,
        ));
        for alpha in [51u8, 102, 153, 204] {
            for gray in RGB_CUBE_VALUES {
                ga_entries.push(encode_direct_entry(
                    entry_format,
                    srgb8_pixel(gray, gray, gray, alpha),
                    Transfer::Srgb,
                ));
            }
        }
        ga_entries
    } else if target_is_gray || background_is_gray {
        let mut gray_entries = Vec::with_capacity(256);
        for gray in 0u8..=255 {
            gray_entries.push(encode_direct_entry(
                entry_format,
                srgb8_pixel(gray, gray, gray, 255),
                Transfer::Srgb,
            ));
        }
        gray_entries
    } else {
        let mut ga_entries = Vec::with_capacity(256);
        for i in 0u16..231 {
            let gray = ((i * 256 + 115) / 231) as u8;
            ga_entries.push(encode_direct_entry(
                entry_format,
                srgb8_pixel(gray, gray, gray, 255),
                Transfer::Srgb,
            ));
        }
        ga_entries.push(background_entry);
        for alpha in [51u8, 102, 153, 204] {
            for gray in RGB_CUBE_VALUES {
                ga_entries.push(encode_direct_entry(
                    entry_format,
                    composite(srgb8_pixel(gray, gray, gray, alpha), background_linear),
                    Transfer::Srgb,
                ));
            }
        }
        ga_entries
    };

    for y in 0..decoded.height {
        let row = target_row(image, buffer, row_stride, y);
        for x in 0..decoded.width {
            let source = decoded_pixel(decoded, x, y);
            let (gray, alpha) = if target_has_alpha {
                (
                    encode_nonlinear_byte(decoded.nonlinear_encode, source.g),
                    encode_u8(source.a),
                )
            } else {
                let (gray, _, _, alpha) = nonlinear8_components(source, Transfer::Srgb);
                (gray, alpha)
            };
            row[x] = if target_has_alpha {
                if alpha > 229 {
                    (((231u16 * u16::from(gray)) + 128) >> 8) as u8
                } else if alpha < 26 {
                    231
                } else {
                    226 + 6 * div51(alpha) + div51(gray)
                }
            } else if target_is_gray || background_is_gray {
                let rendered = composite(source, background_linear);
                encode_nonlinear_byte(Transfer::Srgb, luminance(rendered))
            } else if alpha > 229 {
                (((231u16 * u16::from(gray)) + 128) >> 8) as u8
            } else if alpha < 26 {
                231
            } else {
                226 + 6 * div51(alpha) + div51(gray)
            };
        }
    }

    copy_colormap_entries(image, colormap, entry_size, max_entries, &entries);
    Ok(())
}

fn finish_rgb_alpha_color_colormapped_read(
    image: &mut png_image,
    decoded: &DecodedImage,
    background: png_const_colorp,
    buffer: png_voidp,
    row_stride: png_int_32,
    colormap: png_voidp,
    entry_format: png_uint_32,
    entry_size: usize,
    max_entries: usize,
) -> Result<(), String> {
    let effective_background = if (entry_format & PNG_FORMAT_FLAG_LINEAR) != 0 {
        ptr::null()
    } else {
        background
    };
    let background_linear = if effective_background.is_null() {
        LinearBackground {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    } else {
        supplied_background(entry_format, effective_background)
    };
    let background_entry = if effective_background.is_null() {
        encode_direct_entry(
            entry_format,
            CanonicalPixel {
                r: background_linear.r,
                g: background_linear.g,
                b: background_linear.b,
                a: 1.0,
            },
            Transfer::Srgb,
        )
    } else {
        encode_background_entry(entry_format, effective_background)
    };

    let mut entries = build_rgb_cube_entries(entry_format);
    let background_pixel = CanonicalPixel {
        r: background_linear.r,
        g: background_linear.g,
        b: background_linear.b,
        a: 1.0,
    };
    let (background_r, background_g, background_b, _) =
        nonlinear8_components(background_pixel, Transfer::Srgb);
    let background_index = rgb_cube_index(background_r, background_g, background_b) as usize;
    let background_in_cube = background_entry == entries[background_index];

    if !background_in_cube {
        if max_entries < 244 {
            return Err("png_image_finish_read[color-map]: too many colors".into());
        }
        entries.push(background_entry);
        for r in RGB_MID_VALUES {
            for g in RGB_MID_VALUES {
                for b in RGB_MID_VALUES {
                    entries.push(encode_direct_entry(
                        entry_format,
                        composite(srgb8_pixel(r, g, b, 128), background_linear),
                        Transfer::Srgb,
                    ));
                }
            }
        }
    } else if max_entries < 216 {
        return Err("png_image_finish_read[color-map]: too many colors".into());
    }

    for y in 0..decoded.height {
        let row = target_row(image, buffer, row_stride, y);
        for x in 0..decoded.width {
            let source = decoded_pixel(decoded, x, y);
            row[x] = if background_in_cube {
                let rendered = if (entry_format & PNG_FORMAT_FLAG_LINEAR) != 0 {
                    let (r, g, b, alpha) = nonlinear8_components(source, decoded.nonlinear_encode);
                    composite(srgb8_pixel(r, g, b, alpha), background_linear)
                } else {
                    composite(source, background_linear)
                };
                let (r, g, b, _) = nonlinear8_components(rendered, Transfer::Srgb);
                rgb_cube_index(r, g, b)
            } else {
                let (r, g, b, alpha) = nonlinear8_components(source, decoded.nonlinear_encode);
                if alpha >= 196 {
                    rgb_cube_index(r, g, b)
                } else if alpha < 64 {
                    216
                } else {
                    217 + rgb_mid_index(r) * 9 + rgb_mid_index(g) * 3 + rgb_mid_index(b)
                }
            };
        }
    }

    copy_colormap_entries(image, colormap, entry_size, max_entries, &entries);
    Ok(())
}

fn srgb8_pixel(r: u8, g: u8, b: u8, a: u8) -> CanonicalPixel {
    CanonicalPixel {
        r: srgb_to_linear(f64::from(r) / 255.0),
        g: srgb_to_linear(f64::from(g) / 255.0),
        b: srgb_to_linear(f64::from(b) / 255.0),
        a: f64::from(a) / 255.0,
    }
}

fn bucket_key(entry_format: png_uint_32, pixel: CanonicalPixel) -> usize {
    let linear = (entry_format & PNG_FORMAT_FLAG_LINEAR) != 0;
    let color = (entry_format & PNG_FORMAT_FLAG_COLOR) != 0;
    let alpha = (entry_format & PNG_FORMAT_FLAG_ALPHA) != 0;

    if linear {
        let gray16 = encode_u16(luminance(pixel));
        let r16 = encode_u16(pixel.r);
        let g16 = encode_u16(pixel.g);
        let b16 = encode_u16(pixel.b);
        let a8 = encode_u8(pixel.a);
        if alpha {
            if color {
                usize::from(((r16 >> 14) << 6) | ((g16 >> 14) << 4) | ((b16 >> 14) << 2) | u16::from(a8 >> 6))
            } else {
                usize::from((((gray16 >> 8) & 0x00f0) | u16::from(a8 >> 4)) as u8)
            }
        } else if color {
            usize::from(((r16 >> 13) << 5) | ((g16 >> 13) << 2) | (b16 >> 14))
        } else {
            usize::from((gray16 >> 8) as u8)
        }
    } else {
        let gray = encode_u8(linear_to_srgb(luminance(pixel)));
        let r = encode_u8(linear_to_srgb(pixel.r));
        let g = encode_u8(linear_to_srgb(pixel.g));
        let b = encode_u8(linear_to_srgb(pixel.b));
        let a = encode_u8(pixel.a);
        if alpha {
            if color {
                usize::from(((r >> 6) << 6) | ((g >> 6) << 4) | ((b >> 6) << 2) | (a >> 6))
            } else {
                usize::from(((gray >> 4) << 4) | (a >> 4))
            }
        } else if color {
            usize::from(((r >> 5) << 5) | ((g >> 5) << 2) | (b >> 6))
        } else {
            usize::from(gray)
        }
    }
}

fn bucket_pixel(entry_format: png_uint_32, index: usize) -> CanonicalPixel {
    let linear = (entry_format & PNG_FORMAT_FLAG_LINEAR) != 0;
    let color = (entry_format & PNG_FORMAT_FLAG_COLOR) != 0;
    let alpha = (entry_format & PNG_FORMAT_FLAG_ALPHA) != 0;

    let decode_srgb = |value: u8| srgb_to_linear(f64::from(value) / 255.0);
    if linear {
        if alpha {
            if color {
                let r = ((index >> 6) & 0x3) as f64 / 3.0;
                let g = ((index >> 4) & 0x3) as f64 / 3.0;
                let b = ((index >> 2) & 0x3) as f64 / 3.0;
                let a = (index & 0x3) as f64 / 3.0;
                CanonicalPixel { r, g, b, a }
            } else {
                let gray = ((index >> 4) & 0xf) as f64 / 15.0;
                let a = (index & 0xf) as f64 / 15.0;
                CanonicalPixel {
                    r: gray,
                    g: gray,
                    b: gray,
                    a,
                }
            }
        } else if color {
            let r = ((index >> 5) & 0x7) as f64 / 7.0;
            let g = ((index >> 2) & 0x7) as f64 / 7.0;
            let b = (index & 0x3) as f64 / 3.0;
            CanonicalPixel { r, g, b, a: 1.0 }
        } else {
            let gray = index as f64 / 255.0;
            CanonicalPixel {
                r: gray,
                g: gray,
                b: gray,
                a: 1.0,
            }
        }
    } else if alpha {
        if color {
            let r = ((index >> 6) & 0x3) as u8 * 85;
            let g = ((index >> 4) & 0x3) as u8 * 85;
            let b = ((index >> 2) & 0x3) as u8 * 85;
            let a = (index & 0x3) as f64 / 3.0;
            CanonicalPixel {
                r: decode_srgb(r),
                g: decode_srgb(g),
                b: decode_srgb(b),
                a,
            }
        } else {
            let gray = ((index >> 4) & 0xf) as u8 * 17;
            let a = (index & 0xf) as f64 / 15.0;
            let gray = decode_srgb(gray);
            CanonicalPixel {
                r: gray,
                g: gray,
                b: gray,
                a,
            }
        }
    } else if color {
        let r = (((index >> 5) & 0x7) as u8 * 255) / 7;
        let g = (((index >> 2) & 0x7) as u8 * 255) / 7;
        let b = ((index & 0x3) as u8 * 255) / 3;
        CanonicalPixel {
            r: decode_srgb(r),
            g: decode_srgb(g),
            b: decode_srgb(b),
            a: 1.0,
        }
    } else {
        let gray = decode_srgb(index as u8);
        CanonicalPixel {
            r: gray,
            g: gray,
            b: gray,
            a: 1.0,
        }
    }
}

fn target_row_stride_bytes(image: &png_image, row_stride: png_int_32) -> usize {
    let stride_components = if row_stride == 0 {
        image.width as usize * pixel_channels(image.format)
    } else {
        row_stride.unsigned_abs() as usize
    };
    stride_components.saturating_mul(pixel_component_size(image.format))
}

fn target_row<'a>(
    image: &png_image,
    buffer: png_voidp,
    row_stride: png_int_32,
    y: usize,
) -> &'a mut [u8] {
    let stride = target_row_stride_bytes(image, row_stride);
    let row_index = if row_stride < 0 {
        image.height as usize - 1 - y
    } else {
        y
    };
    unsafe { slice::from_raw_parts_mut(buffer.cast::<u8>().add(row_index * stride), stride) }
}

fn finish_direct_read(
    image: &mut png_image,
    source_format: png_uint_32,
    decoded: &DecodedImage,
    background: png_const_colorp,
    buffer: png_voidp,
    row_stride: png_int_32,
) -> Result<(), String> {
    let target_format = image.format;
    let target_has_alpha = (target_format & PNG_FORMAT_FLAG_ALPHA) != 0;
    let target_linear = (target_format & PNG_FORMAT_FLAG_LINEAR) != 0;
    let target_pixel_size = direct_pixel_size(target_format);
    let source_has_alpha = (source_format & PNG_FORMAT_FLAG_ALPHA) != 0;

    if finish_direct_read_scaled_linear_8bit(image, source_format, decoded, background, buffer, row_stride) {
        return Ok(());
    }

    if finish_direct_read_nonlinear_8bit(image, source_format, decoded, background, buffer, row_stride) {
        return Ok(());
    }

    if finish_direct_read_raw_16bit_default_gamma(
        image,
        source_format,
        decoded,
        background,
        buffer,
        row_stride,
    ) {
        return Ok(());
    }

    let can_passthrough_direct = background.is_null()
        && !target_has_alpha
        && !source_has_alpha
        && !target_linear
        && (target_format & PNG_FORMAT_FLAG_COLOR) == 0
        && (target_format & (PNG_FORMAT_FLAG_BGR | PNG_FORMAT_FLAG_AFIRST | PNG_FORMAT_FLAG_COLORMAP)) == 0
        && decoded.bit_depth == PngBitDepth::Eight
        && match (decoded.color_type, target_format & (PNG_FORMAT_FLAG_COLOR | PNG_FORMAT_FLAG_ALPHA)) {
            (PngColorType::Grayscale, 0) => true,
            (PngColorType::GrayscaleAlpha, 0) => true,
            _ => false,
        };

    if can_passthrough_direct {
        for y in 0..decoded.height {
            let row = target_row(image, buffer, row_stride, y);
            let source_row = &decoded.data[y * decoded.line_size..(y + 1) * decoded.line_size];
            let width = image.width as usize;

            match (decoded.color_type, target_format & (PNG_FORMAT_FLAG_COLOR | PNG_FORMAT_FLAG_ALPHA)) {
                (PngColorType::Grayscale, 0) => {
                    row[..width].copy_from_slice(&source_row[..width]);
                }
                (PngColorType::GrayscaleAlpha, 0) => {
                    for (index, chunk) in source_row.chunks_exact(2).enumerate() {
                        row[index] = chunk[0];
                    }
                }
                _ => {}
            }
        }

        return Ok(());
    }

    for y in 0..decoded.height {
        let row = target_row(image, buffer, row_stride, y);
        for x in 0..decoded.width {
            let use_direct_gray_linear = target_linear
                && (source_format
                    & (PNG_FORMAT_FLAG_COLOR
                        | PNG_FORMAT_FLAG_ALPHA
                        | PNG_FORMAT_FLAG_LINEAR
                        | PNG_FORMAT_FLAG_COLORMAP))
                    == 0;
            let source = if use_direct_gray_linear {
                decoded_direct_gray_linear_pixel(decoded, x, y)
                    .unwrap_or_else(|| decoded_direct_pixel(decoded, x, y))
            } else {
                decoded_direct_pixel(decoded, x, y)
            };
            let start = x * target_pixel_size;
            let end = start + target_pixel_size;
            let pixel_out = &mut row[start..end];
            if !target_has_alpha && !target_linear && source.a <= 0.0 {
                if !background.is_null() {
                    pixel_out.copy_from_slice(&encode_background_entry(target_format, background));
                }
                continue;
            }

            let rendered = if target_has_alpha {
                source
            } else if target_linear {
                composite(
                    source,
                    LinearBackground {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                    },
                )
            } else if !background.is_null() {
                composite(source, supplied_background(target_format, background))
            } else if (target_format & PNG_FORMAT_FLAG_COLORMAP) != 0 && source_has_alpha {
                return Err("png_image_finish_read[color-map]: no color-map".into());
            } else {
                composite(source, buffer_background(target_format, pixel_out))
            };

            if decoded.bit_depth == PngBitDepth::Sixteen
                && !target_linear
                && is_default_gamma(decoded.direct_nonlinear_encode)
            {
                write_direct_pixel_default_gamma_from_linear(target_format, rendered, pixel_out);
            } else {
                write_direct_pixel_with_transfer(
                    target_format,
                    rendered,
                    decoded.direct_nonlinear_encode,
                    pixel_out,
                );
            }
        }
    }

    Ok(())
}

fn finish_colormapped_read(
    image: &mut png_image,
    source_format: png_uint_32,
    decoded: &DecodedImage,
    background: png_const_colorp,
    buffer: png_voidp,
    row_stride: png_int_32,
    colormap: png_voidp,
) -> Result<(), String> {
    let entry_format = direct_entry_format(image.format);
    let entry_size = direct_pixel_size(entry_format);
    let max_entries = image.colormap_entries as usize;
    let source_has_alpha = (source_format & PNG_FORMAT_FLAG_ALPHA) != 0;
    let target_has_alpha = (entry_format & PNG_FORMAT_FLAG_ALPHA) != 0;
    let target_linear = (entry_format & PNG_FORMAT_FLAG_LINEAR) != 0;
    let target_colormap_requires_background =
        !target_has_alpha && !target_linear && source_has_alpha && background.is_null();

    if target_colormap_requires_background {
        return Err("png_image_finish_read[color-map]: no color-map".into());
    }

    if target_linear
        && (source_format
            & (PNG_FORMAT_FLAG_COLOR
                | PNG_FORMAT_FLAG_ALPHA
                | PNG_FORMAT_FLAG_LINEAR
                | PNG_FORMAT_FLAG_COLORMAP))
            == 0
    {
        let mut entries = Vec::<Vec<u8>>::new();
        let mut lookup = HashMap::<u8, u8>::new();

        for y in 0..decoded.height {
            let row = target_row(image, buffer, row_stride, y);
            for x in 0..decoded.width {
                let source = decoded_pixel(decoded, x, y);
                let gray = encode_nonlinear_byte(decoded.nonlinear_encode, source.g);
                let index = if let Some(index) = lookup.get(&gray).copied() {
                    index
                } else {
                    if entries.len() >= max_entries || entries.len() >= 256 {
                        return Err("png_image_finish_read[color-map]: too many colors".into());
                    }
                    let linear = clamp01(f64::from(gray) / 255.0).powf(1.0 / (45_455.0 / 100_000.0));
                    let index = entries.len() as u8;
                    lookup.insert(gray, index);
                    entries.push(encode_direct_entry(
                        entry_format,
                        CanonicalPixel {
                            r: linear,
                            g: linear,
                            b: linear,
                            a: 1.0,
                        },
                        Transfer::Srgb,
                    ));
                    index
                };
                row[x] = index;
            }
        }

        image.colormap_entries = entries.len() as png_uint_32;
        copy_colormap_entries(image, colormap, entry_size, max_entries, &entries);
        return Ok(());
    }

    let composite_background = if target_has_alpha {
        None
    } else if target_linear {
        Some(LinearBackground {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        })
    } else if !background.is_null() {
        Some(supplied_background(entry_format, background))
    } else {
        None
    };

    if source_has_alpha && (source_format & PNG_FORMAT_FLAG_COLOR) == 0 {
        return finish_gray_alpha_colormapped_read(
            image,
            decoded,
            background,
            buffer,
            row_stride,
            colormap,
            entry_format,
            entry_size,
            max_entries,
        );
    }

    if (source_format & PNG_FORMAT_FLAG_COLORMAP) == 0
        && !target_has_alpha
        && (entry_format & PNG_FORMAT_FLAG_COLOR) != 0
        && source_has_alpha
    {
        if matches!(decoded.color_type, PngColorType::Rgba | PngColorType::Rgb) {
            return finish_rgb_alpha_color_colormapped_read(
                image,
                decoded,
                background,
                buffer,
                row_stride,
                colormap,
                entry_format,
                entry_size,
                max_entries,
            );
        }
    }

    let mut entries = Vec::<Vec<u8>>::new();
    let mut lookup = HashMap::<Vec<u8>, u8>::new();
    let mut overflow = false;

    'exact: for y in 0..decoded.height {
        let row = target_row(image, buffer, row_stride, y);
        for x in 0..decoded.width {
            let source = decoded_pixel(decoded, x, y);
            let exact_background =
                source.a <= 0.0
                    && !target_has_alpha
                    && !target_linear
                    && !background.is_null();
            let rendered = if source.a <= 0.0 {
                if let Some(background) = composite_background {
                    CanonicalPixel {
                        r: background.r,
                        g: background.g,
                        b: background.b,
                        a: 1.0,
                    }
                } else {
                    source
                }
            } else if let Some(background) = composite_background {
                composite(source, background)
            } else {
                source
            };
            let entry = if exact_background {
                encode_background_entry(entry_format, background)
            } else {
                encode_direct_entry(entry_format, rendered, decoded.nonlinear_encode)
            };

            let index = if let Some(index) = lookup.get(&entry).copied() {
                index
            } else {
                if entries.len() >= max_entries || entries.len() >= 256 {
                    overflow = true;
                    break 'exact;
                }
                let index = entries.len() as u8;
                lookup.insert(entry.clone(), index);
                entries.push(entry);
                index
            };
            row[x] = index;
        }
    }

    if overflow {
        let palette_size = max_entries.min(256);
        if palette_size == 0 {
            return Err("png_image_finish_read[color-map]: too many colors".into());
        }

        if (entry_format & PNG_FORMAT_FLAG_COLOR) != 0 {
            let has_extra_background =
                source_has_alpha && !target_has_alpha && !target_linear && !background.is_null();
            let mut rgb_entries = build_rgb_cube_entries(entry_format);

            if target_has_alpha {
                rgb_entries.push(encode_direct_entry(
                    entry_format,
                    srgb8_pixel(255, 255, 255, 0),
                    Transfer::Srgb,
                ));
                for r in RGB_MID_VALUES {
                    for g in RGB_MID_VALUES {
                        for b in RGB_MID_VALUES {
                            rgb_entries.push(encode_direct_entry(
                                entry_format,
                                srgb8_pixel(r, g, b, 128),
                                Transfer::Srgb,
                            ));
                        }
                    }
                }
            } else if has_extra_background {
                rgb_entries.push(encode_background_entry(entry_format, background));
                let background_pixel = background_pixel(entry_format, background);
                let background_linear = LinearBackground {
                    r: background_pixel.r,
                    g: background_pixel.g,
                    b: background_pixel.b,
                };
                for r in RGB_MID_VALUES {
                    for g in RGB_MID_VALUES {
                        for b in RGB_MID_VALUES {
                            let rendered = composite(srgb8_pixel(r, g, b, 128), background_linear);
                            rgb_entries.push(encode_direct_entry(
                                entry_format,
                                rendered,
                                Transfer::Srgb,
                            ));
                        }
                    }
                }
            }

            let palette_len = rgb_entries.len().min(palette_size);
            for y in 0..decoded.height {
                let row = target_row(image, buffer, row_stride, y);
                for x in 0..decoded.width {
                    let source = decoded_pixel(decoded, x, y);
                    let (r, g, b, a) = nonlinear8_components(source, decoded.nonlinear_encode);
                    let index = if target_has_alpha {
                        if a >= 196 {
                            rgb_cube_index(r, g, b)
                        } else if a < 64 {
                            216
                        } else {
                            217 + rgb_mid_index(r) * 9 + rgb_mid_index(g) * 3 + rgb_mid_index(b)
                        }
                    } else if has_extra_background {
                        if a >= 196 {
                            rgb_cube_index(r, g, b)
                        } else if a < 64 {
                            216
                        } else {
                            217 + rgb_mid_index(r) * 9 + rgb_mid_index(g) * 3 + rgb_mid_index(b)
                        }
                    } else if source_has_alpha {
                        if a < 64 {
                            0
                        } else if a >= 196 {
                            rgb_cube_index(r, g, b)
                        } else {
                            let rendered = if let Some(background) = composite_background {
                                composite(source, background)
                            } else {
                                source
                            };
                            let (rr, gg, bb, _) =
                                nonlinear8_components(rendered, decoded.nonlinear_encode);
                            rgb_cube_index(rr, gg, bb)
                        }
                    } else {
                        rgb_cube_index(r, g, b)
                    };
                    row[x] = index.min((palette_len.saturating_sub(1)) as u8);
                }
            }

            entries = rgb_entries.into_iter().take(palette_len).collect();
            image.colormap_entries = entries.len() as png_uint_32;
        } else if (entry_format & PNG_FORMAT_FLAG_COLOR) == 0
            && (!source_has_alpha || target_has_alpha)
        {
            let gray_of = |pixel: CanonicalPixel| {
                if (source_format & PNG_FORMAT_FLAG_COLOR) != 0 {
                    encode_nonlinear_byte_trunc(decoded.nonlinear_encode, luminance(pixel))
                } else {
                    encode_nonlinear_byte(decoded.nonlinear_encode, luminance(pixel))
                }
            };

            if target_has_alpha && source_has_alpha {
                let mut ga_entries = Vec::with_capacity(256);
                for i in 0u16..231 {
                    let gray = ((i * 256 + 115) / 231) as u8;
                    ga_entries.push(encode_direct_entry(
                        entry_format,
                        srgb8_pixel(gray, gray, gray, 255),
                        Transfer::Srgb,
                    ));
                }
                ga_entries.push(encode_direct_entry(
                    entry_format,
                    srgb8_pixel(255, 255, 255, 0),
                    Transfer::Srgb,
                ));
                for alpha in [51u8, 102, 153, 204] {
                    for gray in [0u8, 51, 102, 153, 204, 255] {
                        ga_entries.push(encode_direct_entry(
                            entry_format,
                            srgb8_pixel(gray, gray, gray, alpha),
                            Transfer::Srgb,
                        ));
                    }
                }

                let palette_len = ga_entries.len().min(palette_size);
                for y in 0..decoded.height {
                    let row = target_row(image, buffer, row_stride, y);
                    for x in 0..decoded.width {
                        let source = decoded_pixel(decoded, x, y);
                        let gray = if (source_format & PNG_FORMAT_FLAG_COLOR) != 0 {
                            gray_of(source).saturating_sub(1)
                        } else {
                            gray_of(source)
                        };
                        let alpha = encode_u8(source.a);
                        let index = if alpha > 229 {
                            (((231u16 * u16::from(gray)) + 128) >> 8) as u8
                        } else if alpha < 26 {
                            231
                        } else {
                            226 + 6 * div51(alpha) + div51(gray)
                        };
                        row[x] = index.min((palette_len.saturating_sub(1)) as u8);
                    }
                }

                entries = ga_entries.into_iter().take(palette_len).collect();
                image.colormap_entries = entries.len() as png_uint_32;
            } else {
                let mut gray_entries = Vec::with_capacity(256);
                for gray in 0u16..=255 {
                    let gray = gray as u8;
                    gray_entries.push(encode_direct_entry(
                        entry_format,
                        srgb8_pixel(gray, gray, gray, 255),
                        Transfer::Srgb,
                    ));
                }

                let palette_len = gray_entries.len().min(palette_size);
                for y in 0..decoded.height {
                    let row = target_row(image, buffer, row_stride, y);
                    for x in 0..decoded.width {
                        let source = decoded_pixel(decoded, x, y);
                        row[x] = gray_of(source).min((palette_len.saturating_sub(1)) as u8);
                    }
                }

                entries = gray_entries.into_iter().take(palette_len).collect();
                image.colormap_entries = entries.len() as png_uint_32;
            }
        } else {
            let mut bucket_entries = vec![vec![0u8; entry_size]; palette_size];

            for y in 0..decoded.height {
                let row = target_row(image, buffer, row_stride, y);
                for x in 0..decoded.width {
                    let source = decoded_pixel(decoded, x, y);
                    let rendered = if source.a <= 0.0 {
                        if let Some(background) = composite_background {
                            CanonicalPixel {
                                r: background.r,
                                g: background.g,
                                b: background.b,
                                a: 1.0,
                            }
                        } else {
                            source
                        }
                    } else if let Some(background) = composite_background {
                        composite(source, background)
                    } else {
                        source
                    };
                    let index = bucket_key(entry_format, rendered).min(palette_size - 1);
                    row[x] = index as u8;
                }
            }

            for (index, entry) in bucket_entries.iter_mut().enumerate() {
                let pixel = bucket_pixel(entry_format, index);
                write_direct_pixel_with_transfer(
                    entry_format,
                    pixel,
                    decoded.nonlinear_encode,
                    entry,
                );
            }
            entries = bucket_entries;
            image.colormap_entries = palette_size as png_uint_32;
        }
    } else {
        image.colormap_entries = entries.len() as png_uint_32;
    }

    copy_colormap_entries(image, colormap, entry_size, max_entries, &entries);

    Ok(())
}

fn extract_direct_input(
    image: &png_image,
    buffer: png_const_voidp,
    row_stride: png_int_32,
) -> Vec<Vec<u8>> {
    let pixel_size = direct_pixel_size(image.format);
    let stride = target_row_stride_bytes(image, row_stride);
    let mut rows = Vec::with_capacity(image.height as usize);
    for y in 0..image.height as usize {
        let row_index = if row_stride < 0 {
            image.height as usize - 1 - y
        } else {
            y
        };
        let row_ptr = unsafe { buffer.cast::<u8>().add(row_index * stride) };
        let row = unsafe { slice::from_raw_parts(row_ptr, image.width as usize * pixel_size) };
        rows.push(row.to_vec());
    }
    rows
}

fn extract_index_rows(
    image: &png_image,
    buffer: png_const_voidp,
    row_stride: png_int_32,
) -> Vec<Vec<u8>> {
    let stride = target_row_stride_bytes(image, row_stride);
    let mut rows = Vec::with_capacity(image.height as usize);
    for y in 0..image.height as usize {
        let row_index = if row_stride < 0 {
            image.height as usize - 1 - y
        } else {
            y
        };
        let row_ptr = unsafe { buffer.cast::<u8>().add(row_index * stride) };
        let row = unsafe { slice::from_raw_parts(row_ptr, image.width as usize) };
        rows.push(row.to_vec());
    }
    rows
}

fn canonical_to_png_direct(
    color: PngColorType,
    depth: PngBitDepth,
    pixel: CanonicalPixel,
    nonlinear_encode: Transfer,
) -> Vec<u8> {
    let mut out = Vec::new();
    let gray = luminance(pixel);

    let push_linear16 = |out: &mut Vec<u8>, value: f64| {
        out.extend_from_slice(&encode_u16(value).to_be_bytes());
    };
    let push_nonlinear8 = |out: &mut Vec<u8>, value: f64| {
        out.push(encode_nonlinear_byte(nonlinear_encode, value));
    };
    let push_alpha16 = |out: &mut Vec<u8>, value: f64| {
        out.extend_from_slice(&encode_u16(value).to_be_bytes());
    };
    let push_alpha8 = |out: &mut Vec<u8>, value: f64| {
        out.push(encode_u8(value));
    };

    match (color, depth) {
        (PngColorType::Grayscale, PngBitDepth::Eight) => push_nonlinear8(&mut out, gray),
        (PngColorType::GrayscaleAlpha, PngBitDepth::Eight) => {
            push_nonlinear8(&mut out, gray);
            push_alpha8(&mut out, pixel.a);
        }
        (PngColorType::Rgb, PngBitDepth::Eight) => {
            push_nonlinear8(&mut out, pixel.r);
            push_nonlinear8(&mut out, pixel.g);
            push_nonlinear8(&mut out, pixel.b);
        }
        (PngColorType::Rgba, PngBitDepth::Eight) => {
            push_nonlinear8(&mut out, pixel.r);
            push_nonlinear8(&mut out, pixel.g);
            push_nonlinear8(&mut out, pixel.b);
            push_alpha8(&mut out, pixel.a);
        }
        (PngColorType::Grayscale, PngBitDepth::Sixteen) => push_linear16(&mut out, gray),
        (PngColorType::GrayscaleAlpha, PngBitDepth::Sixteen) => {
            push_linear16(&mut out, gray);
            push_alpha16(&mut out, pixel.a);
        }
        (PngColorType::Rgb, PngBitDepth::Sixteen) => {
            push_linear16(&mut out, pixel.r);
            push_linear16(&mut out, pixel.g);
            push_linear16(&mut out, pixel.b);
        }
        (PngColorType::Rgba, PngBitDepth::Sixteen) => {
            push_linear16(&mut out, pixel.r);
            push_linear16(&mut out, pixel.g);
            push_linear16(&mut out, pixel.b);
            push_alpha16(&mut out, pixel.a);
        }
        _ => {}
    }

    out
}

fn encode_png_bytes(
    image: &png_image,
    convert_to_8bit: c_int,
    buffer: png_const_voidp,
    row_stride: png_int_32,
    colormap: png_const_voidp,
) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let fast = (image.flags & PNG_IMAGE_FLAG_FAST) != 0;
    let is_colormap = (image.format & PNG_FORMAT_FLAG_COLORMAP) != 0;

    let mut encoder = Encoder::new(&mut bytes, image.width, image.height);
    encoder.set_compression(if fast { Compression::Fast } else { Compression::Balanced });

    let png_color = if is_colormap {
        PngColorType::Indexed
    } else if (image.format & PNG_FORMAT_FLAG_COLOR) != 0 {
        if (image.format & PNG_FORMAT_FLAG_ALPHA) != 0 {
            PngColorType::Rgba
        } else {
            PngColorType::Rgb
        }
    } else if (image.format & PNG_FORMAT_FLAG_ALPHA) != 0 {
        PngColorType::GrayscaleAlpha
    } else {
        PngColorType::Grayscale
    };

    let png_depth = if is_colormap || convert_to_8bit != 0 || (image.format & PNG_FORMAT_FLAG_LINEAR) == 0 {
        PngBitDepth::Eight
    } else {
        PngBitDepth::Sixteen
    };
    encoder.set_color(png_color);
    encoder.set_depth(png_depth);
    if png_depth == PngBitDepth::Eight {
        encoder.set_source_srgb(SrgbRenderingIntent::Perceptual);
    } else {
        encoder.set_source_gamma(ScaledFloat::from_scaled(100_000));
    }

    let encoded_rows = if is_colormap {
        let entry_format = direct_entry_format(image.format);
        let entry_size = direct_pixel_size(entry_format);
        let map_size = entry_size
            .checked_mul(image.colormap_entries as usize)
            .ok_or_else(|| "colormap too large".to_string())?;
        let entries = if colormap.is_null() {
            return Err("invalid argument".into());
        } else {
            unsafe { slice::from_raw_parts(colormap.cast::<u8>(), map_size) }
        };

        let mut palette = Vec::with_capacity(image.colormap_entries as usize * 3);
        let mut trns = Vec::with_capacity(image.colormap_entries as usize);
        let has_alpha = (entry_format & PNG_FORMAT_FLAG_ALPHA) != 0;
        for index in 0..image.colormap_entries as usize {
            let start = index * entry_size;
            let pixel = read_direct_pixel(entry_format, &entries[start..start + entry_size]);
            let rgb = canonical_to_png_direct(
                PngColorType::Rgb,
                PngBitDepth::Eight,
                pixel,
                Transfer::Srgb,
            );
            palette.extend_from_slice(&rgb);
            if has_alpha {
                trns.push(encode_u8(pixel.a));
            }
        }
        encoder.set_palette(palette);
        if has_alpha {
            while trns.last() == Some(&255) {
                trns.pop();
            }
            if !trns.is_empty() {
                encoder.set_trns(trns);
            }
        }
        extract_index_rows(image, buffer, row_stride)
    } else {
        let rows = extract_direct_input(image, buffer, row_stride);
        let can_passthrough_direct = png_depth == PngBitDepth::Eight
            && (image.format & PNG_FORMAT_FLAG_LINEAR) == 0
            && (image.format & (PNG_FORMAT_FLAG_BGR | PNG_FORMAT_FLAG_AFIRST)) == 0;

        if can_passthrough_direct {
            return {
                let mut writer = encoder.write_header().map_err(|err| err.to_string())?;
                let total_size = rows.iter().map(Vec::len).sum();
                let mut image_bytes = Vec::with_capacity(total_size);
                for row in rows {
                    image_bytes.extend_from_slice(&row);
                }
                writer
                    .write_image_data(&image_bytes)
                    .and_then(|_| writer.finish())
                    .map_err(|err| err.to_string())?;
                Ok(bytes)
            };
        }

        let mut encoded = Vec::with_capacity(image.height as usize);
        let direct_output_encode = if png_depth == PngBitDepth::Eight {
            Transfer::Srgb
        } else {
            Transfer::Srgb
        };
        for row in rows {
            let mut out_row = Vec::new();
            let pixel_size = direct_pixel_size(image.format);
            for pixel_bytes in row.chunks_exact(pixel_size) {
                let pixel = read_write_direct_pixel(image.format, pixel_bytes);
                let png_pixel =
                    canonical_to_png_direct(png_color, png_depth, pixel, direct_output_encode);
                out_row.extend_from_slice(&png_pixel);
            }
            encoded.push(out_row);
        }
        encoded
    };

    let mut writer = encoder.write_header().map_err(|err| err.to_string())?;
    let total_size = encoded_rows.iter().map(Vec::len).sum();
    let mut image_bytes = Vec::with_capacity(total_size);
    for row in encoded_rows {
        image_bytes.extend_from_slice(&row);
    }
    writer
        .write_image_data(&image_bytes)
        .and_then(|_| writer.finish())
        .map_err(|err| err.to_string())?;

    Ok(bytes)
}

fn write_memory_output(bytes: &[u8], memory: png_voidp, memory_bytes: *mut png_alloc_size_t) -> c_int {
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

pub(crate) unsafe fn begin_read_from_file(image: png_imagep, file_name: png_const_charp) -> c_int {
    clear_image_status(image);
    let bytes = match read_file_bytes(file_name) {
        Ok(bytes) => bytes,
        Err(err) => return set_image_error(image, format!("png_image_begin_read_from_file failed: {err}")),
    };
    let header = match parse_header(&bytes) {
        Ok(header) => header,
        Err(err) => return set_image_error(image, format!("png_image_begin_read_from_file failed: {err}")),
    };
    install_state(image, header, bytes)
}

pub(crate) unsafe fn begin_read_from_stdio(image: png_imagep, file: *mut FILE) -> c_int {
    clear_image_status(image);
    let bytes = match read_stdio_bytes(file) {
        Ok(bytes) => bytes,
        Err(err) => return set_image_error(image, format!("png_image_begin_read_from_stdio failed: {err}")),
    };
    let header = match parse_header(&bytes) {
        Ok(header) => header,
        Err(err) => return set_image_error(image, format!("png_image_begin_read_from_stdio failed: {err}")),
    };
    install_state(image, header, bytes)
}

pub(crate) unsafe fn begin_read_from_memory(
    image: png_imagep,
    memory: png_const_voidp,
    size: usize,
) -> c_int {
    clear_image_status(image);
    if memory.is_null() {
        return set_image_error(image, "png_image_begin_read_from_memory failed: invalid argument");
    }
    let bytes = unsafe { slice::from_raw_parts(memory.cast::<u8>(), size) }.to_vec();
    let header = match parse_header(&bytes) {
        Ok(header) => header,
        Err(err) => return set_image_error(image, format!("png_image_begin_read_from_memory failed: {err}")),
    };
    install_state(image, header, bytes)
}

pub(crate) unsafe fn finish_read(
    image: png_imagep,
    background: png_const_colorp,
    buffer: png_voidp,
    row_stride: png_int_32,
    colormap: png_voidp,
) -> c_int {
    if image.is_null() || buffer.is_null() {
        return set_image_error(image, "png_image_finish_read failed");
    }

    let image_ref = unsafe { &mut *image };
    if image_ref.opaque.is_null() {
        return set_image_error(image, "png_image_finish_read failed");
    }

    let state = unsafe { &*image_ref.opaque.cast::<SimplifiedImageState>() };
    let decoded = match decode_png(&state.bytes, image_ref.flags) {
        Ok(decoded) => decoded,
        Err(err) => return set_image_error(image, format!("png_image_finish_read failed: {err}")),
    };

    let result = if (image_ref.format & PNG_FORMAT_FLAG_COLORMAP) != 0 {
        finish_colormapped_read(
            image_ref,
            state.source_format,
            &decoded,
            background,
            buffer,
            row_stride,
            colormap,
        )
    } else {
        finish_direct_read(
            image_ref,
            state.source_format,
            &decoded,
            background,
            buffer,
            row_stride,
        )
    };

    match result {
        Ok(()) => {
            unsafe { free_simplified_image_state(image) };
            clear_image_status(image);
            1
        }
        Err(err) => set_image_error(image, err),
    }
}

pub(crate) unsafe fn write_to_file(
    image: png_imagep,
    file_name: png_const_charp,
    convert_to_8bit: c_int,
    buffer: png_const_voidp,
    row_stride: png_int_32,
    colormap: png_const_voidp,
) -> c_int {
    if image.is_null() || file_name.is_null() {
        return set_image_error(image, "png_image_write_to_file failed");
    }

    let bytes = match encode_png_bytes(unsafe { &*image }, convert_to_8bit, buffer, row_stride, colormap) {
        Ok(bytes) => bytes,
        Err(err) => return set_image_error(image, format!("png_image_write_to_file failed: {err}")),
    };
    let path = match unsafe { CStr::from_ptr(file_name) }.to_str() {
        Ok(path) => path,
        Err(_) => return set_image_error(image, "png_image_write_to_file failed: invalid path"),
    };
    if let Err(err) = fs::write(path, &bytes) {
        return set_image_error(image, format!("png_image_write_to_file failed: {err}"));
    }

    clear_image_status(image);
    1
}

pub(crate) unsafe fn write_to_stdio(
    image: png_imagep,
    file: *mut FILE,
    convert_to_8bit: c_int,
    buffer: png_const_voidp,
    row_stride: png_int_32,
    colormap: png_const_voidp,
) -> c_int {
    if image.is_null() || file.is_null() {
        return set_image_error(image, "png_image_write_to_stdio failed");
    }

    let bytes = match encode_png_bytes(unsafe { &*image }, convert_to_8bit, buffer, row_stride, colormap) {
        Ok(bytes) => bytes,
        Err(err) => return set_image_error(image, format!("png_image_write_to_stdio failed: {err}")),
    };
    let written = unsafe { libc::fwrite(bytes.as_ptr().cast(), 1, bytes.len(), file) };
    if written != bytes.len() {
        return set_image_error(image, "png_image_write_to_stdio failed");
    }

    clear_image_status(image);
    1
}

pub(crate) unsafe fn write_to_memory(
    image: png_imagep,
    memory: png_voidp,
    memory_bytes: *mut png_alloc_size_t,
    convert_to_8bit: c_int,
    buffer: png_const_voidp,
    row_stride: png_int_32,
    colormap: png_const_voidp,
) -> c_int {
    if image.is_null() || memory_bytes.is_null() {
        return set_image_error(image, "png_image_write_to_memory failed");
    }

    let bytes = match encode_png_bytes(unsafe { &*image }, convert_to_8bit, buffer, row_stride, colormap) {
        Ok(bytes) => bytes,
        Err(err) => return set_image_error(image, format!("png_image_write_to_memory failed: {err}")),
    };
    let status = write_memory_output(&bytes, memory, memory_bytes);
    if status == 0 && !memory.is_null() {
        return set_image_error(image, "png_image_write_to_memory failed");
    }

    clear_image_status(image);
    status
}

pub(crate) unsafe fn image_free(image: png_imagep) {
    unsafe { free_simplified_image_state(image) };
}
