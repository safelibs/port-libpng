#![allow(dead_code)]

use crate::chunks::{read_core, read_info_core, write_core, write_info_core};
use crate::common::{
    PNG_INFO_PLTE, PNG_INFO_bKGD, PNG_INFO_cHRM, PNG_INFO_eXIf, PNG_INFO_gAMA, PNG_INFO_hIST,
    PNG_INFO_iCCP, PNG_INFO_oFFs, PNG_INFO_pHYs, PNG_INFO_sBIT, PNG_INFO_sCAL, PNG_INFO_sRGB,
    PNG_INFO_tIME, PNG_INFO_tRNS, PNG_IO_CHUNK_CRC, PNG_IO_CHUNK_DATA, PNG_IO_CHUNK_HDR,
    PNG_IO_SIGNATURE, PNG_IO_WRITING, PNG_OPTION_INVALID, PNG_OPTION_OFF, PNG_OPTION_ON,
};
use crate::io;
use crate::read_util::{ReadPhase, checked_rowbytes_for_width};
use crate::state;
use crate::types::*;
use core::ffi::{c_char, c_int};
use core::ptr;
use libc::FILE;

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

fn double_from_fixed(value: png_fixed_point) -> f64 {
    f64::from(value) / 100_000.0
}

fn latin1_bytes_to_string(bytes: &[u8]) -> String {
    bytes.iter().map(|&byte| char::from(byte)).collect()
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
    if state::with_png(png_ptr, |png_state| png_state.has_pending_chunk_header).unwrap_or(false) {
        return true;
    }

    loop {
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_parse_snapshot_free(_snapshot: *mut core::ffi::c_void) {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_sync_png_info_aliases(
    _png_ptr: png_structrp,
    _info_ptr: png_const_inforp,
) {
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
    let core = read_info_core(info_ptr);
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
    let core = read_info_core(info_ptr);
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
    _info_ptr: png_inforp,
    _text_ptr: *mut png_textp,
    num_text: *mut c_int,
) -> c_int {
    if !num_text.is_null() {
        unsafe { *num_text = 0 };
    }
    0
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
        if !trans_alpha.is_null() && !info_state.trans_alpha.is_empty() {
            unsafe { *trans_alpha = info_state.trans_alpha.as_mut_ptr() };
        }
        if !num_trans.is_null() {
            unsafe { *num_trans = info_state.trans_alpha.len() as c_int };
        }
        if !trans_color.is_null() {
            unsafe { *trans_color = &mut info_state.core.trans_color };
        }
        PNG_INFO_tRNS
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_sCAL(
    _png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
    unit: *mut c_int,
    width: png_doublep,
    height: png_doublep,
) -> png_uint_32 {
    state::with_info(info_ptr.cast_mut(), |info_state| {
        if info_state.scal_width.is_empty() || info_state.scal_height.is_empty() {
            return 0;
        }
        if !unit.is_null() {
            unsafe { *unit = info_state.scal_unit };
        }
        if !width.is_null() {
            unsafe {
                *width = core::str::from_utf8(&info_state.scal_width)
                    .ok()
                    .and_then(|text| text.trim_end_matches('\0').parse::<f64>().ok())
                    .unwrap_or(0.0);
            }
        }
        if !height.is_null() {
            unsafe {
                *height = core::str::from_utf8(&info_state.scal_height)
                    .ok()
                    .and_then(|text| text.trim_end_matches('\0').parse::<f64>().ok())
                    .unwrap_or(0.0);
            }
        }
        PNG_INFO_sCAL
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_sCAL_fixed(
    _png_ptr: png_const_structrp,
    _info_ptr: png_const_inforp,
    _unit: *mut c_int,
    _width: png_fixed_point_p,
    _height: png_fixed_point_p,
) -> png_uint_32 {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_sCAL_s(
    _png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
    unit: *mut c_int,
    swidth: *mut png_charp,
    sheight: *mut png_charp,
) -> png_uint_32 {
    state::with_info_mut(info_ptr.cast_mut(), |info_state| {
        if info_state.scal_width.is_empty() || info_state.scal_height.is_empty() {
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
    state::update_png(png_ptr, |png_state| {
        png_state.sig_bytes = num_bytes.clamp(0, 8);
    });
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
    if !(0..31).contains(&option) {
        return PNG_OPTION_INVALID;
    }
    state::update_png(png_ptr, |png_state| {
        let mask = 3u32 << option;
        let setting = if onoff != 0 { PNG_OPTION_ON } else { PNG_OPTION_OFF } as u32;
        png_state.options = (png_state.options & !mask) | (setting << option);
    });
    if onoff != 0 { PNG_OPTION_ON } else { PNG_OPTION_OFF }
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
        info_state.core.colorspace.end_points_xy.whitex = white_x;
        info_state.core.colorspace.end_points_xy.whitey = white_y;
        info_state.core.colorspace.end_points_xy.redx = red_x;
        info_state.core.colorspace.end_points_xy.redy = red_y;
        info_state.core.colorspace.end_points_xy.greenx = green_x;
        info_state.core.colorspace.end_points_xy.greeny = green_y;
        info_state.core.colorspace.end_points_xy.bluex = blue_x;
        info_state.core.colorspace.end_points_xy.bluey = blue_y;
        info_state.core.valid |= PNG_INFO_cHRM;
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
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    sig_bit: png_const_color_8p,
) {
    if sig_bit.is_null() {
        return;
    }
    state::update_info(info_ptr, |info_state| {
        info_state.core.sig_bit = unsafe { *sig_bit };
        info_state.core.valid |= PNG_INFO_sBIT;
    });
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
    });
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
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    unit: c_int,
    width: f64,
    height: f64,
) {
    let width = format!("{width}\0").into_bytes();
    let height = format!("{height}\0").into_bytes();
    state::update_info(info_ptr, |info_state| {
        info_state.scal_unit = unit;
        info_state.scal_width = width.clone();
        info_state.scal_height = height.clone();
        info_state.core.valid |= PNG_INFO_sCAL;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_sCAL_fixed(
    _png_ptr: png_const_structrp,
    _info_ptr: png_inforp,
    _unit: c_int,
    _width: png_fixed_point,
    _height: png_fixed_point,
) {
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_sCAL_s(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    unit: c_int,
    swidth: png_const_charp,
    sheight: png_const_charp,
) {
    if swidth.is_null() || sheight.is_null() {
        return;
    }
    let swidth_len = unsafe { libc::strlen(swidth) };
    let sheight_len = unsafe { libc::strlen(sheight) };
    state::update_info(info_ptr, |info_state| {
        info_state.scal_unit = unit;
        info_state.scal_width =
            unsafe { core::slice::from_raw_parts(swidth.cast::<u8>(), swidth_len + 1) }.to_vec();
        info_state.scal_height =
            unsafe { core::slice::from_raw_parts(sheight.cast::<u8>(), sheight_len + 1) }.to_vec();
        info_state.core.valid |= PNG_INFO_sCAL;
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
        if png_state.core.num_rows == 0 {
            png_state.core.num_rows = png_state.core.height;
        }
    });
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_call_read_transform_info(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
) -> c_int {
    let core = read_core(png_ptr);
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
        });
    }
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_safe_call_read_row(
    png_ptr: png_structrp,
    row: png_bytep,
    _display_row: png_bytep,
) -> c_int {
    if !drain_idat_stream(png_ptr) {
        return 0;
    }

    let core_before = read_core(png_ptr);
    let rowbytes = if core_before.rowbytes != 0 {
        core_before.rowbytes
    } else {
        read_info_core(ptr::null()).rowbytes
    };
    if !row.is_null() && rowbytes != 0 {
        unsafe {
            ptr::write_bytes(row, 0, rowbytes);
        }
    }
    state::update_png(png_ptr, |png_state| {
        png_state.core.row_number = png_state.core.row_number.saturating_add(1);
        if png_state.core.num_rows != 0 && png_state.core.row_number >= png_state.core.num_rows {
            png_state.core.pass += 1;
            png_state.core.row_number = 0;
        }
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
    state::with_png(png_ptr, |png_state| png_state.progressive_state.last_pause_bytes as png_uint_32)
        .unwrap_or(0)
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
    _transforms: png_uint_32,
    _params: png_voidp,
) {
    unsafe {
        bridge_png_write_info(png_ptr, info_ptr);
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
    _png_ptr: png_structrp,
    _filler: png_uint_32,
    _flags: c_int,
) {
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_filler(
    _png_ptr: png_structrp,
    _filler: png_uint_32,
    _flags: c_int,
) {
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_packing(_png_ptr: png_structrp) {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_packswap(_png_ptr: png_structrp) {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_swap(_png_ptr: png_structrp) {}

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
