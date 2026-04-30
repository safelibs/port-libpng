use crate::chunks::{read_core, read_info_core};
use crate::common::{
    PNG_INFO_bKGD, PNG_INFO_hIST, PNG_INFO_iCCP, PNG_INFO_oFFs, PNG_INFO_sBIT, PNG_INFO_sCAL,
    PNG_INFO_tIME,
};
use crate::io;
use crate::read_util::checked_rowbytes_for_width;
use crate::state::{self, OwnedTextChunk, PngInfoState, PngStructState, WriteSessionState};
use crate::types::*;
use core::ffi::c_int;
use core::ptr;
use core::slice;
use flate2::Compression as ZlibCompression;
use flate2::write::ZlibEncoder;
use png::chunk::{self, ChunkType};
use png::text_metadata::{ITXtChunk, TEXtChunk, ZTXtChunk};
use png::{
    BitDepth as PngBitDepth, ColorType as PngColorType, Compression as PngCompression, Encoder,
    Filter as PngFilter, PixelDimensions, ScaledFloat, SourceChromaticities, SrgbRenderingIntent,
    Unit,
};
use std::borrow::Cow;
use std::io::Write;

const PNG_FLAG_ROW_INIT: png_uint_32 = 0x0040;
const PNG_INTERLACE_TRANSFORM: png_uint_32 = 0x0002;
const PNG_BGR: png_uint_32 = 0x0001;
const PNG_PACK: png_uint_32 = 0x0004;
const PNG_SHIFT: png_uint_32 = 0x0008;
const PNG_SWAP_BYTES: png_uint_32 = 0x0010;
const PNG_INVERT_MONO: png_uint_32 = 0x0020;
const PNG_FILLER: png_uint_32 = 0x8000;
const PNG_PACKSWAP: png_uint_32 = 0x10000;
const PNG_SWAP_ALPHA: png_uint_32 = 0x20000;
const PNG_INVERT_ALPHA: png_uint_32 = 0x80000;
const PNG_FLAG_FILLER_AFTER: png_uint_32 = 0x0080;
const PNG_TEXT_COMPRESSION_NONE_WR: c_int = -3;
const PNG_TEXT_COMPRESSION_ZTXT_WR: c_int = -2;
const PNG_TEXT_COMPRESSION_NONE: c_int = -1;
const PNG_TEXT_COMPRESSION_ZTXT: c_int = 0;
const PNG_ITXT_COMPRESSION_NONE: c_int = 1;
const PNG_ITXT_COMPRESSION_ZTXT: c_int = 2;
const MAX_IDAT_CHUNK_LEN: usize = 0x7fff_ffff;
const ADAM7_PASSES: [(usize, usize, usize, usize); 7] = [
    (0, 8, 0, 8),
    (4, 8, 0, 8),
    (0, 4, 4, 8),
    (2, 4, 0, 4),
    (0, 2, 2, 4),
    (1, 2, 0, 2),
    (0, 1, 1, 2),
];

#[derive(Clone, Copy)]
struct WriteRowLayout {
    width: usize,
    color_type: png_byte,
    channels: png_byte,
    bit_depth: png_byte,
    rowbytes: usize,
}

#[derive(Clone, Copy)]
struct WriteTransformSpec {
    input: WriteRowLayout,
    output: WriteRowLayout,
    transformations: png_uint_32,
    filler_after: bool,
    shift: png_color_8,
}

fn uses_manual_adam7_rows(core: &png_safe_read_core) -> bool {
    core.interlaced != 0 && (core.transformations & PNG_INTERLACE_TRANSFORM) == 0
}

fn adam7_pass_width(width: usize, pass: usize) -> usize {
    let (x_offset, x_step, _, _) = ADAM7_PASSES[pass];
    width.saturating_sub(x_offset).div_ceil(x_step)
}

fn adam7_pass_lines(height: usize, pass: usize) -> usize {
    let (_, _, y_offset, y_step) = ADAM7_PASSES[pass];
    height.saturating_sub(y_offset).div_ceil(y_step)
}

fn next_nonempty_adam7_pass(width: usize, height: usize, pass: usize) -> Option<usize> {
    (pass..ADAM7_PASSES.len()).find(|&candidate| {
        adam7_pass_width(width, candidate) != 0 && adam7_pass_lines(height, candidate) != 0
    })
}

fn initialize_write_row_state(core: &mut png_safe_read_core) {
    if !uses_manual_adam7_rows(core) {
        core.num_rows = core.height;
        return;
    }

    let width = usize::try_from(core.width).unwrap_or(0);
    let height = usize::try_from(core.height).unwrap_or(0);
    let Some(pass) = next_nonempty_adam7_pass(width, height, 0) else {
        core.pass = 7;
        core.row_number = 0;
        core.num_rows = 0;
        return;
    };

    core.pass = pass as c_int;
    core.row_number = 0;
    core.num_rows = adam7_pass_lines(height, pass) as png_uint_32;
}

fn advance_write_row_state(core: &mut png_safe_read_core) {
    core.row_number = core.row_number.saturating_add(1);

    if core.num_rows == 0 || core.row_number < core.num_rows {
        return;
    }

    core.row_number = 0;

    if !uses_manual_adam7_rows(core) {
        if core.interlaced != 0 {
            core.pass = core.pass.saturating_add(1);
        }
        return;
    }

    let width = usize::try_from(core.width).unwrap_or(0);
    let height = usize::try_from(core.height).unwrap_or(0);
    let next = usize::try_from(core.pass)
        .ok()
        .and_then(|pass| next_nonempty_adam7_pass(width, height, pass.saturating_add(1)));
    if let Some(pass) = next {
        core.pass = pass as c_int;
        core.num_rows = adam7_pass_lines(height, pass) as png_uint_32;
    } else {
        core.pass = 7;
        core.num_rows = 0;
    }
}

fn png_error_message(png_ptr: png_structrp, message: &'static [u8]) -> ! {
    unsafe { crate::error::png_error(png_ptr, message.as_ptr().cast()) }
}

fn latin1_bytes_to_string(bytes: &[u8]) -> String {
    bytes.iter().map(|&byte| char::from(byte)).collect()
}

fn trim_trailing_nul(bytes: &[u8]) -> &[u8] {
    if let Some((&0, body)) = bytes.split_last() {
        body
    } else {
        bytes
    }
}

fn current_width(info_core: &png_safe_info_core, png_core: &png_safe_read_core) -> Option<usize> {
    if info_core.width != 0 {
        usize::try_from(info_core.width).ok()
    } else if png_core.width != 0 {
        usize::try_from(png_core.width).ok()
    } else {
        None
    }
}

fn current_height(info_core: &png_safe_info_core, png_core: &png_safe_read_core) -> Option<usize> {
    if info_core.height != 0 {
        usize::try_from(info_core.height).ok()
    } else if png_core.height != 0 {
        usize::try_from(png_core.height).ok()
    } else {
        None
    }
}

fn current_rowbytes(
    info_core: &png_safe_info_core,
    png_core: &png_safe_read_core,
) -> Option<usize> {
    if info_core.rowbytes != 0 {
        return Some(info_core.rowbytes);
    }
    if png_core.info_rowbytes != 0 {
        return Some(png_core.info_rowbytes);
    }

    let pixel_depth = current_pixel_depth(info_core, png_core)?;
    let width = current_width(info_core, png_core)?;

    checked_rowbytes_for_width(width, pixel_depth)
}

fn current_pixel_depth(
    info_core: &png_safe_info_core,
    png_core: &png_safe_read_core,
) -> Option<usize> {
    if info_core.pixel_depth != 0 {
        return Some(usize::from(info_core.pixel_depth));
    }
    if png_core.pixel_depth != 0 {
        return Some(usize::from(png_core.pixel_depth));
    }

    let channels = if info_core.channels != 0 {
        usize::from(info_core.channels)
    } else {
        usize::from(png_core.channels)
    };
    let bit_depth = if info_core.bit_depth != 0 {
        usize::from(info_core.bit_depth)
    } else {
        usize::from(png_core.bit_depth)
    };

    channels.checked_mul(bit_depth).filter(|depth| *depth != 0)
}

fn write_compression(settings: crate::common::WriteZlibSettings) -> PngCompression {
    match settings.level {
        i32::MIN..=-1 => PngCompression::Balanced,
        0 => PngCompression::NoCompression,
        1 => PngCompression::Fastest,
        2..=3 => PngCompression::Fast,
        4..=6 => PngCompression::Balanced,
        _ => PngCompression::High,
    }
}

fn write_filter(_png_state: &PngStructState) -> PngFilter {
    PngFilter::NoFilter
}

fn png_color_type(color_type: png_byte) -> Option<PngColorType> {
    match color_type {
        0 => Some(PngColorType::Grayscale),
        2 => Some(PngColorType::Rgb),
        3 => Some(PngColorType::Indexed),
        4 => Some(PngColorType::GrayscaleAlpha),
        6 => Some(PngColorType::Rgba),
        _ => None,
    }
}

fn png_bit_depth(bit_depth: png_byte) -> Option<PngBitDepth> {
    match bit_depth {
        1 => Some(PngBitDepth::One),
        2 => Some(PngBitDepth::Two),
        4 => Some(PngBitDepth::Four),
        8 => Some(PngBitDepth::Eight),
        16 => Some(PngBitDepth::Sixteen),
        _ => None,
    }
}

fn write_layout_from_info(info_state: &PngInfoState) -> Option<WriteRowLayout> {
    let width = usize::try_from(info_state.core.width).ok()?;
    let channels = info_state.core.channels;
    let bit_depth = info_state.core.bit_depth;
    let rowbytes = checked_rowbytes_for_width(
        width,
        usize::from(channels).checked_mul(usize::from(bit_depth))?,
    )?;

    Some(WriteRowLayout {
        width,
        color_type: info_state.core.color_type,
        channels,
        bit_depth,
        rowbytes,
    })
}

fn write_transform_spec(
    png_state: &PngStructState,
    info_state: &PngInfoState,
) -> Option<WriteTransformSpec> {
    let output = write_layout_from_info(info_state)?;
    let mut input_channels = output.channels;
    let mut input_bit_depth = output.bit_depth;

    if (png_state.core.transformations & PNG_FILLER) != 0 {
        input_channels = input_channels.checked_add(1)?;
    }

    if (png_state.core.transformations & PNG_PACK) != 0 && output.bit_depth < 8 {
        input_bit_depth = 8;
    }

    let input_rowbytes = checked_rowbytes_for_width(
        output.width,
        usize::from(input_channels).checked_mul(usize::from(input_bit_depth))?,
    )?;

    Some(WriteTransformSpec {
        input: WriteRowLayout {
            width: output.width,
            color_type: output.color_type,
            channels: input_channels,
            bit_depth: input_bit_depth,
            rowbytes: input_rowbytes,
        },
        output,
        transformations: png_state.core.transformations,
        filler_after: (png_state.core.flags & PNG_FLAG_FILLER_AFTER) != 0,
        shift: png_state.core.shift,
    })
}

fn packswap_byte(value: u8, bit_depth: png_byte) -> u8 {
    match bit_depth {
        1 => value.reverse_bits(),
        2 => {
            ((value & 0b1100_0000) >> 6)
                | ((value & 0b0011_0000) >> 2)
                | ((value & 0b0000_1100) << 2)
                | ((value & 0b0000_0011) << 6)
        }
        4 => value.rotate_right(4),
        _ => value,
    }
}

fn apply_write_packswap(row: &mut [u8], bit_depth: png_byte) {
    if !matches!(bit_depth, 1 | 2 | 4) {
        return;
    }

    for byte in row {
        *byte = packswap_byte(*byte, bit_depth);
    }
}

fn apply_write_swap_bytes(row: &mut [u8], bit_depth: png_byte) {
    if bit_depth != 16 {
        return;
    }

    for sample in row.chunks_exact_mut(2) {
        sample.swap(0, 1);
    }
}

fn strip_filler_channel(
    row: &mut Vec<u8>,
    layout: &mut WriteRowLayout,
    at_start: bool,
) -> Option<()> {
    let mut out = Vec::with_capacity(layout.rowbytes);

    match (layout.channels, layout.bit_depth) {
        (2, 8) => {
            for pixel in row.chunks_exact(2) {
                out.push(if at_start { pixel[1] } else { pixel[0] });
            }
        }
        (2, 16) => {
            for pixel in row.chunks_exact(4) {
                if at_start {
                    out.extend_from_slice(&pixel[2..4]);
                } else {
                    out.extend_from_slice(&pixel[0..2]);
                }
            }
        }
        (4, 8) => {
            for pixel in row.chunks_exact(4) {
                if at_start {
                    out.extend_from_slice(&pixel[1..4]);
                } else {
                    out.extend_from_slice(&pixel[0..3]);
                }
            }
        }
        (4, 16) => {
            for pixel in row.chunks_exact(8) {
                if at_start {
                    out.extend_from_slice(&pixel[2..8]);
                } else {
                    out.extend_from_slice(&pixel[0..6]);
                }
            }
        }
        _ => return None,
    }

    layout.channels = layout.channels.saturating_sub(1);
    layout.rowbytes = checked_rowbytes_for_width(
        layout.width,
        usize::from(layout.channels).checked_mul(usize::from(layout.bit_depth))?,
    )?;
    *row = out;
    Some(())
}

fn pack_row(
    row: &mut Vec<u8>,
    layout: &mut WriteRowLayout,
    output_bit_depth: png_byte,
) -> Option<()> {
    if layout.bit_depth != 8 || layout.channels != 1 || !matches!(output_bit_depth, 1 | 2 | 4) {
        return None;
    }

    let mut out =
        vec![0u8; checked_rowbytes_for_width(layout.width, usize::from(output_bit_depth))?];

    match output_bit_depth {
        1 => {
            for (i, sample) in row.iter().take(layout.width).enumerate() {
                if *sample != 0 {
                    out[i / 8] |= 0x80 >> (i % 8);
                }
            }
        }
        2 => {
            for (i, sample) in row.iter().take(layout.width).enumerate() {
                out[i / 4] |= (sample & 0x03) << (6 - ((i % 4) * 2));
            }
        }
        4 => {
            for (i, sample) in row.iter().take(layout.width).enumerate() {
                out[i / 2] |= (sample & 0x0f) << if i % 2 == 0 { 4 } else { 0 };
            }
        }
        _ => return None,
    }

    layout.bit_depth = output_bit_depth;
    layout.rowbytes = out.len();
    *row = out;
    Some(())
}

fn channel_shift_value(value: u32, bit_depth: png_byte, true_bits: png_byte) -> u32 {
    let shift_start = i32::from(bit_depth) - i32::from(true_bits);
    let shift_dec = i32::from(true_bits);
    let mut out = 0u32;
    let mut shift = shift_start;
    while shift > -shift_dec {
        if shift > 0 {
            out |= value << shift;
        } else {
            out |= value >> (-shift);
        }
        shift -= shift_dec;
    }
    out
}

fn shift_channel_depths(layout: WriteRowLayout, shift: &png_color_8) -> Option<Vec<png_byte>> {
    if layout.color_type == 3 {
        return None;
    }

    let mut depths = Vec::with_capacity(usize::from(layout.channels));
    match layout.color_type {
        0 => depths.push(shift.gray),
        2 => {
            depths.push(shift.red);
            depths.push(shift.green);
            depths.push(shift.blue);
        }
        4 => {
            depths.push(shift.gray);
            depths.push(shift.alpha);
        }
        6 => {
            depths.push(shift.red);
            depths.push(shift.green);
            depths.push(shift.blue);
            depths.push(shift.alpha);
        }
        _ => return None,
    }

    Some(depths)
}

fn apply_write_shift(row: &mut [u8], layout: WriteRowLayout, shift: &png_color_8) {
    let Some(channel_depths) = shift_channel_depths(layout, shift) else {
        return;
    };

    if channel_depths
        .iter()
        .any(|&depth| depth == 0 || depth > layout.bit_depth)
    {
        return;
    }

    match layout.bit_depth {
        1 | 2 | 4 => {
            if layout.channels != 1 {
                return;
            }
            let rowbytes = row.len();
            let mask = if shift.gray == 1 && layout.bit_depth == 2 {
                0x55
            } else if layout.bit_depth == 4 && shift.gray == 3 {
                0x11
            } else {
                0xff
            };
            let shift_start = i32::from(layout.bit_depth) - i32::from(shift.gray);
            let shift_dec = i32::from(shift.gray);

            for byte in row.iter_mut().take(rowbytes) {
                let value = u32::from(*byte);
                let mut out = 0u32;
                let mut shift_value = shift_start;
                while shift_value > -shift_dec {
                    if shift_value > 0 {
                        out |= value << shift_value;
                    } else {
                        out |= (value >> (-shift_value)) & mask;
                    }
                    shift_value -= shift_dec;
                }
                *byte = (out & 0xff) as u8;
            }
        }
        8 => {
            for (i, sample) in row.iter_mut().enumerate() {
                let channel = i % channel_depths.len();
                *sample = (channel_shift_value(
                    u32::from(*sample),
                    layout.bit_depth,
                    channel_depths[channel],
                ) & 0xff) as u8;
            }
        }
        16 => {
            for (i, sample) in row.chunks_exact_mut(2).enumerate() {
                let channel = i % channel_depths.len();
                let value = u16::from_be_bytes([sample[0], sample[1]]);
                let shifted = channel_shift_value(
                    u32::from(value),
                    layout.bit_depth,
                    channel_depths[channel],
                ) as u16;
                sample.copy_from_slice(&shifted.to_be_bytes());
            }
        }
        _ => {}
    }
}

fn apply_write_swap_alpha(row: &mut [u8], layout: WriteRowLayout) {
    match (layout.color_type, layout.bit_depth) {
        (6, 8) => {
            for pixel in row.chunks_exact_mut(4) {
                pixel.rotate_left(1);
            }
        }
        (6, 16) => {
            for pixel in row.chunks_exact_mut(8) {
                pixel.rotate_left(2);
            }
        }
        (4, 8) => {
            for pixel in row.chunks_exact_mut(2) {
                pixel.swap(0, 1);
            }
        }
        (4, 16) => {
            for pixel in row.chunks_exact_mut(4) {
                pixel.rotate_left(2);
            }
        }
        _ => {}
    }
}

fn apply_write_invert_alpha(row: &mut [u8], layout: WriteRowLayout) {
    match (layout.color_type, layout.bit_depth) {
        (6, 8) => {
            for pixel in row.chunks_exact_mut(4) {
                pixel[3] = 255 - pixel[3];
            }
        }
        (6, 16) => {
            for pixel in row.chunks_exact_mut(8) {
                pixel[6] = 255 - pixel[6];
                pixel[7] = 255 - pixel[7];
            }
        }
        (4, 8) => {
            for pixel in row.chunks_exact_mut(2) {
                pixel[1] = 255 - pixel[1];
            }
        }
        (4, 16) => {
            for pixel in row.chunks_exact_mut(4) {
                pixel[2] = 255 - pixel[2];
                pixel[3] = 255 - pixel[3];
            }
        }
        _ => {}
    }
}

fn apply_write_bgr(row: &mut [u8], layout: WriteRowLayout) {
    match (layout.color_type, layout.bit_depth) {
        (2, 8) => {
            for pixel in row.chunks_exact_mut(3) {
                pixel.swap(0, 2);
            }
        }
        (6, 8) => {
            for pixel in row.chunks_exact_mut(4) {
                pixel.swap(0, 2);
            }
        }
        (2, 16) => {
            for pixel in row.chunks_exact_mut(6) {
                pixel.swap(0, 4);
                pixel.swap(1, 5);
            }
        }
        (6, 16) => {
            for pixel in row.chunks_exact_mut(8) {
                pixel.swap(0, 4);
                pixel.swap(1, 5);
            }
        }
        _ => {}
    }
}

fn apply_write_invert_mono(row: &mut [u8], layout: WriteRowLayout) {
    match (layout.color_type, layout.bit_depth) {
        (0, _) => {
            for byte in row {
                *byte = !*byte;
            }
        }
        (4, 8) => {
            for pixel in row.chunks_exact_mut(2) {
                pixel[0] = !pixel[0];
            }
        }
        (4, 16) => {
            for pixel in row.chunks_exact_mut(4) {
                pixel[0] = !pixel[0];
                pixel[1] = !pixel[1];
            }
        }
        _ => {}
    }
}

fn transform_info_row(row: &[u8], spec: WriteTransformSpec) -> Option<Vec<u8>> {
    if row.len() < spec.input.rowbytes {
        return None;
    }

    let mut layout = spec.input;
    let mut out = row[..spec.input.rowbytes].to_vec();

    if (spec.transformations & PNG_FILLER) != 0 {
        strip_filler_channel(&mut out, &mut layout, spec.filler_after)?;
    }
    if (spec.transformations & PNG_PACKSWAP) != 0 {
        apply_write_packswap(&mut out, layout.bit_depth);
    }
    if (spec.transformations & PNG_PACK) != 0 {
        pack_row(&mut out, &mut layout, spec.output.bit_depth)?;
    }
    if (spec.transformations & PNG_SWAP_BYTES) != 0 {
        apply_write_swap_bytes(&mut out, layout.bit_depth);
    }
    if (spec.transformations & PNG_SHIFT) != 0 {
        apply_write_shift(&mut out, layout, &spec.shift);
    }
    if (spec.transformations & PNG_SWAP_ALPHA) != 0 {
        apply_write_swap_alpha(&mut out, layout);
    }
    if (spec.transformations & PNG_INVERT_ALPHA) != 0 {
        apply_write_invert_alpha(&mut out, layout);
    }
    if (spec.transformations & PNG_BGR) != 0 {
        apply_write_bgr(&mut out, layout);
    }
    if (spec.transformations & PNG_INVERT_MONO) != 0 {
        apply_write_invert_mono(&mut out, layout);
    }

    if layout.channels != spec.output.channels
        || layout.bit_depth != spec.output.bit_depth
        || out.len() != spec.output.rowbytes
    {
        return None;
    }

    Some(out)
}

fn png_srgb_intent(intent: png_uint_16) -> Option<SrgbRenderingIntent> {
    match intent as u8 {
        0 => Some(SrgbRenderingIntent::Perceptual),
        1 => Some(SrgbRenderingIntent::RelativeColorimetric),
        2 => Some(SrgbRenderingIntent::Saturation),
        3 => Some(SrgbRenderingIntent::AbsoluteColorimetric),
        _ => None,
    }
}

fn chunk_data_bkgd(info_state: &PngInfoState) -> Vec<u8> {
    match info_state.core.color_type {
        3 => vec![info_state.core.background.index],
        0 | 4 => info_state.core.background.gray.to_be_bytes().to_vec(),
        2 | 6 => [
            info_state.core.background.red.to_be_bytes().as_slice(),
            info_state.core.background.green.to_be_bytes().as_slice(),
            info_state.core.background.blue.to_be_bytes().as_slice(),
        ]
        .concat(),
        _ => Vec::new(),
    }
}

fn chunk_data_sbit(info_state: &PngInfoState) -> Vec<u8> {
    let sig = info_state.core.sig_bit;
    match info_state.core.color_type {
        0 => vec![sig.gray],
        2 | 3 => vec![sig.red, sig.green, sig.blue],
        4 => vec![sig.gray, sig.alpha],
        6 => vec![sig.red, sig.green, sig.blue, sig.alpha],
        _ => Vec::new(),
    }
}

fn validated_sbit_data(
    png_ptr: png_structrp,
    info_state: &PngInfoState,
    warn: bool,
) -> Option<Vec<u8>> {
    let sig = info_state.core.sig_bit;
    let color_type = info_state.core.color_type;
    let max_bits = if color_type == 3 {
        8
    } else {
        info_state.core.bit_depth
    };
    let invalid = match color_type {
        0 => sig.gray == 0 || sig.gray > max_bits,
        2 | 3 => {
            sig.red == 0
                || sig.red > max_bits
                || sig.green == 0
                || sig.green > max_bits
                || sig.blue == 0
                || sig.blue > max_bits
        }
        4 => sig.gray == 0 || sig.gray > max_bits || sig.alpha == 0 || sig.alpha > max_bits,
        6 => {
            sig.red == 0
                || sig.red > max_bits
                || sig.green == 0
                || sig.green > max_bits
                || sig.blue == 0
                || sig.blue > max_bits
                || sig.alpha == 0
                || sig.alpha > max_bits
        }
        _ => true,
    };

    if invalid {
        if warn {
            unsafe {
                crate::error::png_warning(
                    png_ptr,
                    b"Invalid sBIT depth specified\0".as_ptr().cast(),
                );
            }
        }
        None
    } else {
        Some(chunk_data_sbit(info_state))
    }
}

fn chunk_data_hist(info_state: &PngInfoState) -> Vec<u8> {
    let mut out = Vec::with_capacity(info_state.hist.len() * 2);
    for value in &info_state.hist {
        out.extend_from_slice(&value.to_be_bytes());
    }
    out
}

fn chunk_data_trns(info_state: &PngInfoState) -> Option<Vec<u8>> {
    if (info_state.core.valid & crate::common::PNG_INFO_tRNS) == 0 {
        return None;
    }

    match info_state.core.color_type {
        0 => Some(info_state.core.trans_color.gray.to_be_bytes().to_vec()),
        2 => Some(
            [
                info_state.core.trans_color.red.to_be_bytes().as_slice(),
                info_state.core.trans_color.green.to_be_bytes().as_slice(),
                info_state.core.trans_color.blue.to_be_bytes().as_slice(),
            ]
            .concat(),
        ),
        _ => None,
    }
}

fn chunk_data_offs(info_state: &PngInfoState) -> Option<Vec<u8>> {
    let (x, y, unit) = info_state.offs?;
    let mut out = Vec::with_capacity(9);
    out.extend_from_slice(&x.to_be_bytes());
    out.extend_from_slice(&y.to_be_bytes());
    out.push(unit as u8);
    Some(out)
}

fn chunk_data_time(info_state: &PngInfoState) -> Option<Vec<u8>> {
    let time = info_state.time?;
    let mut out = Vec::with_capacity(7);
    out.extend_from_slice(&time.year.to_be_bytes());
    out.push(time.month);
    out.push(time.day);
    out.push(time.hour);
    out.push(time.minute);
    out.push(time.second);
    Some(out)
}

fn chunk_data_scal(info_state: &PngInfoState) -> Option<Vec<u8>> {
    if info_state.scal_width.is_empty() || info_state.scal_height.is_empty() {
        return None;
    }

    let width = trim_trailing_nul(&info_state.scal_width);
    let height = trim_trailing_nul(&info_state.scal_height);
    let mut out = Vec::with_capacity(2 + width.len() + height.len());
    out.push(info_state.scal_unit as u8);
    out.extend_from_slice(width);
    out.push(0);
    out.extend_from_slice(height);
    Some(out)
}

fn write_iccp_chunk(
    writer: &mut png::Writer<&mut Vec<u8>>,
    info_state: &PngInfoState,
) -> Result<(), png::EncodingError> {
    if info_state.iccp_profile.is_empty() {
        return Ok(());
    }

    let name = if info_state.iccp_name.is_empty() {
        b"_".as_slice()
    } else {
        trim_trailing_nul(&info_state.iccp_name)
    };

    let mut data = Vec::with_capacity(name.len() + 2 + info_state.iccp_profile.len());
    data.extend_from_slice(name);
    data.push(0);
    data.push(0);

    let mut encoder = ZlibEncoder::new(Vec::new(), ZlibCompression::default());
    encoder.write_all(&info_state.iccp_profile)?;
    data.extend_from_slice(&encoder.finish()?);

    writer.write_chunk(chunk::iCCP, &data)
}

fn write_exif_chunk(
    writer: &mut png::Writer<&mut Vec<u8>>,
    info_state: &PngInfoState,
) -> Result<(), png::EncodingError> {
    if (info_state.core.valid & crate::common::PNG_INFO_eXIf) != 0 && !info_state.exif.is_empty() {
        writer.write_chunk(ChunkType(*b"eXIf"), &info_state.exif)?;
    }

    Ok(())
}

fn write_pre_idat_chunks(
    png_ptr: png_structrp,
    writer: &mut png::Writer<&mut Vec<u8>>,
    info_state: &PngInfoState,
) -> Result<(), png::EncodingError> {
    if (info_state.core.valid & PNG_INFO_sBIT) != 0 {
        if let Some(data) =
            validated_sbit_data(png_ptr, info_state, false).filter(|data| !data.is_empty())
        {
            writer.write_chunk(ChunkType(*b"sBIT"), &data)?;
        }
    }

    if (info_state.core.valid & PNG_INFO_iCCP) != 0 {
        write_iccp_chunk(writer, info_state)?;
    }

    if (info_state.core.valid & PNG_INFO_bKGD) != 0 {
        let data = chunk_data_bkgd(info_state);
        if !data.is_empty() {
            writer.write_chunk(ChunkType(*b"bKGD"), &data)?;
        }
    }

    write_exif_chunk(writer, info_state)?;

    if (info_state.core.valid & PNG_INFO_hIST) != 0 && !info_state.hist.is_empty() {
        writer.write_chunk(ChunkType(*b"hIST"), &chunk_data_hist(info_state))?;
    }

    if (info_state.core.valid & PNG_INFO_oFFs) != 0 {
        if let Some(data) = chunk_data_offs(info_state) {
            writer.write_chunk(ChunkType(*b"oFFs"), &data)?;
        }
    }

    if (info_state.core.valid & PNG_INFO_sCAL) != 0 {
        if let Some(data) = chunk_data_scal(info_state) {
            writer.write_chunk(ChunkType(*b"sCAL"), &data)?;
        }
    }

    if (info_state.core.valid & PNG_INFO_tIME) != 0 {
        if let Some(data) = chunk_data_time(info_state) {
            writer.write_chunk(ChunkType(*b"tIME"), &data)?;
        }
    }

    for chunk in &info_state.unknown_chunks {
        if chunk.location == 0x08 {
            continue;
        }
        writer.write_chunk(
            ChunkType(chunk.name[..4].try_into().unwrap_or(*b"uNkN")),
            &chunk.data,
        )?;
    }

    Ok(())
}

fn write_post_idat_chunks(
    writer: &mut png::Writer<&mut Vec<u8>>,
    info_state: &PngInfoState,
    start_text_index: usize,
    write_time: bool,
    write_exif: bool,
) -> Result<(), png::EncodingError> {
    for text in info_state.text_chunks.iter().skip(start_text_index) {
        write_text_chunk(writer, text)?;
    }

    if write_time && (info_state.core.valid & PNG_INFO_tIME) != 0 {
        if let Some(data) = chunk_data_time(info_state) {
            writer.write_chunk(ChunkType(*b"tIME"), &data)?;
        }
    }

    if write_exif {
        write_exif_chunk(writer, info_state)?;
    }

    for chunk in &info_state.unknown_chunks {
        if chunk.location == 0x08 {
            writer.write_chunk(
                ChunkType(chunk.name[..4].try_into().unwrap_or(*b"uNkN")),
                &chunk.data,
            )?;
        }
    }

    Ok(())
}

fn write_text_chunk(
    writer: &mut png::Writer<&mut Vec<u8>>,
    text: &OwnedTextChunk,
) -> Result<(), png::EncodingError> {
    match text.compression {
        PNG_TEXT_COMPRESSION_NONE_WR | PNG_TEXT_COMPRESSION_NONE => {
            writer.write_text_chunk(&TEXtChunk::new(text.keyword.clone(), text.text.clone()))
        }
        PNG_TEXT_COMPRESSION_ZTXT_WR | PNG_TEXT_COMPRESSION_ZTXT => {
            writer.write_text_chunk(&ZTXtChunk::new(text.keyword.clone(), text.text.clone()))
        }
        PNG_ITXT_COMPRESSION_NONE | PNG_ITXT_COMPRESSION_ZTXT => {
            let mut chunk = ITXtChunk::new(text.keyword.clone(), text.text.clone());
            chunk.compressed = text.compression == PNG_ITXT_COMPRESSION_ZTXT;
            chunk.language_tag = text.language_tag.clone();
            chunk.translated_keyword = text.translated_keyword.clone();
            writer.write_text_chunk(&chunk)
        }
        _ => writer.write_text_chunk(&TEXtChunk::new(text.keyword.clone(), text.text.clone())),
    }
}

fn build_png_info(
    info_state: &PngInfoState,
    session: &WriteSessionState,
) -> Result<png::Info<'static>, &'static [u8]> {
    let Some(color_type) = png_color_type(info_state.core.color_type) else {
        return Err(b"unsupported color type\0".as_slice());
    };
    let Some(bit_depth) = png_bit_depth(info_state.core.bit_depth) else {
        return Err(b"unsupported bit depth\0".as_slice());
    };

    let mut info = png::Info::with_size(info_state.core.width, info_state.core.height);
    info.color_type = color_type;
    info.bit_depth = bit_depth;
    info.interlaced = info_state.core.interlace_type != 0;

    if !info_state.palette.is_empty() {
        let mut palette = Vec::with_capacity(info_state.palette.len() * 3);
        for entry in &info_state.palette {
            palette.push(entry.red);
            palette.push(entry.green);
            palette.push(entry.blue);
        }
        info.palette = Some(Cow::Owned(palette));
    }

    if let Some(trns) = if !info_state.trans_alpha.is_empty() {
        Some(info_state.trans_alpha.clone())
    } else {
        chunk_data_trns(info_state)
    } {
        info.trns = Some(Cow::Owned(trns));
    }

    if let Some((res_x, res_y, unit_type)) = info_state.phys {
        let unit = if unit_type == 1 {
            Unit::Meter
        } else {
            Unit::Unspecified
        };
        info.pixel_dims = Some(PixelDimensions {
            xppu: res_x,
            yppu: res_y,
            unit,
        });
    }

    if (info_state.core.valid & crate::common::PNG_INFO_sRGB) != 0 {
        if let Some(intent) = png_srgb_intent(info_state.core.colorspace.rendering_intent) {
            info.srgb = Some(intent);
        }
    } else {
        if (info_state.core.valid & crate::common::PNG_INFO_gAMA) != 0
            && info_state.core.colorspace.gamma > 0
        {
            info.source_gamma = Some(ScaledFloat::from_scaled(
                info_state.core.colorspace.gamma as u32,
            ));
        }
        if (info_state.core.valid & crate::common::PNG_INFO_cHRM) != 0 {
            let xy = info_state.core.colorspace.end_points_xy;
            info.source_chromaticities = Some(SourceChromaticities {
                white: (
                    ScaledFloat::from_scaled(xy.whitex as u32),
                    ScaledFloat::from_scaled(xy.whitey as u32),
                ),
                red: (
                    ScaledFloat::from_scaled(xy.redx as u32),
                    ScaledFloat::from_scaled(xy.redy as u32),
                ),
                green: (
                    ScaledFloat::from_scaled(xy.greenx as u32),
                    ScaledFloat::from_scaled(xy.greeny as u32),
                ),
                blue: (
                    ScaledFloat::from_scaled(xy.bluex as u32),
                    ScaledFloat::from_scaled(xy.bluey as u32),
                ),
            });
        }
    }

    for text in info_state
        .text_chunks
        .iter()
        .take(session.header_text_count)
    {
        match text.compression {
            PNG_TEXT_COMPRESSION_NONE_WR | PNG_TEXT_COMPRESSION_NONE => {
                info.uncompressed_latin1_text
                    .push(TEXtChunk::new(text.keyword.clone(), text.text.clone()));
            }
            PNG_TEXT_COMPRESSION_ZTXT_WR | PNG_TEXT_COMPRESSION_ZTXT => {
                info.compressed_latin1_text
                    .push(ZTXtChunk::new(text.keyword.clone(), text.text.clone()));
            }
            PNG_ITXT_COMPRESSION_NONE | PNG_ITXT_COMPRESSION_ZTXT => {
                let mut chunk = ITXtChunk::new(text.keyword.clone(), text.text.clone());
                chunk.compressed = text.compression == PNG_ITXT_COMPRESSION_ZTXT;
                chunk.language_tag = text.language_tag.clone();
                chunk.translated_keyword = text.translated_keyword.clone();
                info.utf8_text.push(chunk);
            }
            _ => {}
        }
    }

    Ok(info)
}

fn read_packed_pixel(row: &[u8], pixel_depth: usize, x: usize) -> u64 {
    let mut value = 0u64;
    let start_bit = x * pixel_depth;
    for bit in 0..pixel_depth {
        let offset = start_bit + bit;
        let byte = row[offset / 8];
        let mask = 1u8 << (7 - (offset % 8));
        value = (value << 1) | u64::from((byte & mask) != 0);
    }
    value
}

fn write_packed_pixel(row: &mut [u8], pixel_depth: usize, x: usize, value: u64) {
    let start_bit = x * pixel_depth;
    for bit in 0..pixel_depth {
        let offset = start_bit + bit;
        let byte = &mut row[offset / 8];
        let mask = 1u8 << (7 - (offset % 8));
        let set = ((value >> (pixel_depth - bit - 1)) & 1) != 0;
        if set {
            *byte |= mask;
        } else {
            *byte &= !mask;
        }
    }
}

fn merge_adam7_pass_row(
    dest_row: &mut [u8],
    pass_row: &[u8],
    pixel_depth: usize,
    width: usize,
    pass: usize,
) {
    let pass_width = adam7_pass_width(width, pass);
    if pass_width == 0 {
        return;
    }

    let (x_offset, x_step, _, _) = ADAM7_PASSES[pass];
    for index in 0..pass_width {
        let x = x_offset + (index * x_step);
        let pixel = read_packed_pixel(pass_row, pixel_depth, index);
        write_packed_pixel(dest_row, pixel_depth, x, pixel);
    }
}

fn filtered_scanlines(
    info_state: &PngInfoState,
    session: &WriteSessionState,
) -> Result<Vec<u8>, &'static [u8]> {
    let width = usize::try_from(info_state.core.width).map_err(|_| b"write error\0".as_slice())?;
    let height =
        usize::try_from(info_state.core.height).map_err(|_| b"write error\0".as_slice())?;
    let rowbytes = session.rowbytes;
    let pixel_depth = current_pixel_depth(&info_state.core, &png_safe_read_core::default())
        .ok_or(b"write error\0".as_slice())?;
    let interlaced = info_state.core.interlace_type != 0;

    if !interlaced {
        let mut out = Vec::with_capacity(height.saturating_mul(rowbytes.saturating_add(1)));
        for row in session.image_data.chunks_exact(rowbytes) {
            out.push(0);
            out.extend_from_slice(row);
        }
        return Ok(out);
    }

    let mut out = Vec::new();
    for (x_offset, x_step, y_offset, y_step) in ADAM7_PASSES {
        if width <= x_offset || height <= y_offset {
            continue;
        }

        let pass_width = (width - x_offset).div_ceil(x_step);
        let Some(pass_rowbytes) = checked_rowbytes_for_width(pass_width, pixel_depth) else {
            return Err(b"write error\0".as_slice());
        };

        let mut pass_row = vec![0u8; pass_rowbytes];
        let mut y = y_offset;
        while y < height {
            pass_row.fill(0);
            let src = &session.image_data[y * rowbytes..(y + 1) * rowbytes];
            for index in 0..pass_width {
                let x = x_offset + (index * x_step);
                let pixel = read_packed_pixel(src, pixel_depth, x);
                write_packed_pixel(&mut pass_row, pixel_depth, index, pixel);
            }
            out.push(0);
            out.extend_from_slice(&pass_row);
            y += y_step;
        }
    }

    Ok(out)
}

fn compressed_idat_data(
    png_state: &PngStructState,
    info_state: &PngInfoState,
    session: &WriteSessionState,
) -> Result<Vec<u8>, &'static [u8]> {
    let filtered = filtered_scanlines(info_state, session)?;
    let level = match write_compression(png_state.write_zlib) {
        PngCompression::NoCompression => ZlibCompression::none(),
        PngCompression::Fastest => ZlibCompression::fast(),
        PngCompression::Fast => ZlibCompression::new(3),
        PngCompression::Balanced => ZlibCompression::default(),
        PngCompression::High => ZlibCompression::best(),
        _ => ZlibCompression::default(),
    };

    let mut encoder = ZlibEncoder::new(Vec::new(), level);
    encoder
        .write_all(&filtered)
        .map_err(|_| b"write error\0".as_slice())?;
    encoder.finish().map_err(|_| b"write error\0".as_slice())
}

fn copy_missing_rows_from_info(
    info_ptr: png_inforp,
    session: &mut WriteSessionState,
    transform_spec: Option<WriteTransformSpec>,
) {
    let row_pointers = read_info_core(info_ptr).row_pointers;
    if row_pointers.is_null() || session.seen_rows.is_empty() {
        return;
    }

    for (index, seen) in session.seen_rows.iter_mut().enumerate() {
        if *seen {
            continue;
        }
        let row = unsafe { *row_pointers.add(index) };
        if row.is_null() {
            continue;
        }
        let start = index * session.rowbytes;
        let end = start + session.rowbytes;
        if end > session.image_data.len() {
            break;
        }
        let dst = &mut session.image_data[start..end];
        if let Some(spec) = transform_spec {
            let src = unsafe { slice::from_raw_parts(row, spec.input.rowbytes) };
            let Some(transformed) = transform_info_row(src, spec) else {
                continue;
            };
            dst.copy_from_slice(&transformed);
        } else {
            unsafe {
                ptr::copy_nonoverlapping(row, dst.as_mut_ptr(), session.rowbytes);
            }
        }
        *seen = true;
    }
}

fn emit_png_bytes(png_ptr: png_structrp, bytes: &[u8]) -> Result<(), &'static [u8]> {
    let Some((_, write_data_fn, flush_fn, _)) = io::write_callback_registration(png_ptr) else {
        return Err(b"Write Error\0".as_slice());
    };
    let Some(callback) = write_data_fn else {
        return Err(b"Write Error\0".as_slice());
    };

    unsafe {
        callback(png_ptr, bytes.as_ptr().cast_mut(), bytes.len());
        if let Some(flush) = flush_fn {
            flush(png_ptr);
        }
    }

    Ok(())
}

pub(crate) unsafe fn begin_write_info(png_ptr: png_structrp, info_ptr: png_const_inforp) {
    if png_ptr.is_null() || info_ptr.is_null() {
        return;
    }

    let info_state = state::get_info(info_ptr.cast_mut()).unwrap_or_default();
    let info_core = info_state.core;
    let png_core = read_core(png_ptr);
    let Some(rowbytes) = current_rowbytes(&info_core, &png_core) else {
        return;
    };
    let height = current_height(&info_core, &png_core).unwrap_or(0);
    let total = match rowbytes.checked_mul(height) {
        Some(total) => total,
        None => png_error_message(png_ptr, b"write error\0"),
    };
    let header_text_count = info_state.text_chunks.len();
    let captures_header_info = info_core.width != 0 && info_core.height != 0;
    let wrote_time_in_header = (info_state.core.valid & PNG_INFO_tIME) != 0;
    let wrote_exif_in_header = (info_state.core.valid & crate::common::PNG_INFO_eXIf) != 0;

    if (info_state.core.valid & PNG_INFO_sBIT) != 0 {
        let _ = validated_sbit_data(png_ptr, &info_state, true);
    }

    state::update_png(png_ptr, |png_state| {
        let needs_reinit = png_state
            .write_session
            .as_ref()
            .map(|session| session.rowbytes != rowbytes || session.image_data.len() != total)
            .unwrap_or(true);

        if needs_reinit {
            png_state.write_session = Some(WriteSessionState {
                rowbytes,
                image_data: vec![0; total],
                seen_rows: vec![false; height],
                header_text_count: 0,
                total_row_writes: 0,
                header_info_ptr: ptr::null_mut(),
                header_info: None,
                wrote_time_in_header: false,
                wrote_exif_in_header: false,
            });
        }

        png_state.core.flags |= PNG_FLAG_ROW_INIT;
        if info_core.width != 0 {
            png_state.core.width = info_core.width;
        }
        if info_core.height != 0 {
            png_state.core.height = info_core.height;
        }
        png_state.core.rowbytes = rowbytes;
        png_state.core.info_rowbytes = rowbytes;
        if info_core.bit_depth != 0 {
            png_state.core.bit_depth = info_core.bit_depth;
        }
        if info_core.color_type != 0 || info_core.channels != 0 {
            png_state.core.color_type = info_core.color_type;
        }
        if info_core.channels != 0 {
            png_state.core.channels = info_core.channels;
        }
        if info_core.pixel_depth != 0 {
            png_state.core.pixel_depth = info_core.pixel_depth;
        }
        if captures_header_info {
            png_state.core.compression_type = info_core.compression_type;
            png_state.core.filter_type = info_core.filter_type;
            png_state.core.interlaced = info_core.interlace_type;
        }
        png_state.core.num_rows = current_height(&info_core, &png_core).unwrap_or(0) as png_uint_32;
        let wrote_rows = png_state
            .write_session
            .as_ref()
            .map(|session| session.total_row_writes != 0)
            .unwrap_or(false);
        if !wrote_rows {
            png_state.core.row_number = 0;
            png_state.core.pass = 0;
            initialize_write_row_state(&mut png_state.core);
        }

        if let Some(session) = png_state.write_session.as_mut() {
            if captures_header_info || session.header_info.is_none() {
                session.header_info_ptr = info_ptr.cast_mut();
                session.header_info = Some(info_state.clone());
                session.header_text_count = header_text_count;
                session.wrote_time_in_header = wrote_time_in_header;
                session.wrote_exif_in_header = wrote_exif_in_header;
            }
        }

        png_state.passthrough_written = false;
    });
}

fn maybe_transform_write_row(
    png_ptr: png_structrp,
    rowbytes: usize,
    row: png_const_bytep,
) -> Vec<u8> {
    let mut out = if row.is_null() || rowbytes == 0 {
        Vec::new()
    } else {
        unsafe { slice::from_raw_parts(row, rowbytes) }.to_vec()
    };

    let transform = io::write_user_transform_registration(png_ptr);
    if let Some((callback, _, _, _)) = transform {
        if let Some(callback) = callback {
            let core = read_core(png_ptr);
            let mut row_info = png_row_info {
                width: core.width,
                rowbytes,
                color_type: core.color_type,
                bit_depth: core.bit_depth,
                channels: core.channels,
                pixel_depth: core.pixel_depth,
            };
            unsafe {
                callback(png_ptr, &mut row_info, out.as_mut_ptr());
            }
        }
    }

    out
}

pub(crate) unsafe fn write_row(png_ptr: png_structrp, row: png_const_bytep) {
    if png_ptr.is_null() {
        return;
    }

    let rowbytes = state::with_png(png_ptr, |png_state| {
        png_state
            .write_session
            .as_ref()
            .map(|session| session.rowbytes)
            .unwrap_or(png_state.core.rowbytes)
    })
    .unwrap_or(0);
    let row_data = maybe_transform_write_row(png_ptr, rowbytes, row);

    let callbacks = state::with_png_mut(png_ptr, |png_state| {
        let Some(session) = png_state.write_session.as_mut() else {
            return None;
        };
        let height = usize::try_from(png_state.core.height).unwrap_or(0);
        if height == 0 || session.rowbytes == 0 {
            return None;
        }

        let manual_adam7 = uses_manual_adam7_rows(&png_state.core);
        let row_index = if manual_adam7 {
            let pass = usize::try_from(png_state.core.pass).unwrap_or(0);
            if pass >= ADAM7_PASSES.len() {
                return None;
            }
            let (_, _, y_offset, y_step) = ADAM7_PASSES[pass];
            y_offset + (usize::try_from(png_state.core.row_number).unwrap_or(0) * y_step)
        } else {
            usize::try_from(png_state.core.row_number).unwrap_or(0) % height
        };
        if row_index >= height {
            return None;
        }
        let start = row_index * session.rowbytes;
        let end = start + session.rowbytes;
        if end > session.image_data.len() {
            return None;
        }
        if manual_adam7 {
            let pass = usize::try_from(png_state.core.pass).unwrap_or(0);
            let pass_width =
                adam7_pass_width(usize::try_from(png_state.core.width).unwrap_or(0), pass);
            let pass_rowbytes =
                checked_rowbytes_for_width(pass_width, usize::from(png_state.core.pixel_depth))?;
            if row_data.len() < pass_rowbytes {
                return None;
            }
            merge_adam7_pass_row(
                &mut session.image_data[start..end],
                &row_data[..pass_rowbytes],
                usize::from(png_state.core.pixel_depth),
                usize::try_from(png_state.core.width).unwrap_or(0),
                pass,
            );
        } else {
            if row_data.len() < session.rowbytes {
                return None;
            }
            session.image_data[start..end].copy_from_slice(&row_data[..session.rowbytes]);
        }
        if let Some(seen) = session.seen_rows.get_mut(row_index) {
            *seen = true;
        }
        session.total_row_writes = session.total_row_writes.saturating_add(1);

        let callback_row = png_state.core.row_number;
        let callback_pass = png_state.core.pass;
        advance_write_row_state(&mut png_state.core);

        Some((
            png_state.write_row_fn,
            png_state.output_flush_fn,
            png_state.flush_rows,
            callback_row,
            callback_pass,
        ))
    })
    .flatten();

    if let Some((write_row_fn, flush_fn, flush_rows, row_number, pass)) = callbacks {
        if let Some(callback) = write_row_fn {
            unsafe { callback(png_ptr, row_number, pass) };
        }
        if flush_rows > 0 && row_number != 0 && row_number % (flush_rows as png_uint_32) == 0 {
            if let Some(flush) = flush_fn {
                unsafe { flush(png_ptr) };
            }
        }
    }
}

pub(crate) unsafe fn write_rows(png_ptr: png_structrp, rows: png_bytepp, num_rows: png_uint_32) {
    if png_ptr.is_null() || rows.is_null() {
        return;
    }

    for index in 0..num_rows {
        let row = unsafe { *rows.add(index as usize) };
        unsafe { write_row(png_ptr, row.cast_const()) };
    }
}

pub(crate) unsafe fn write_image(png_ptr: png_structrp, image: png_bytepp) {
    if png_ptr.is_null() || image.is_null() {
        return;
    }

    let height = state::with_png(png_ptr, |png_state| png_state.core.height).unwrap_or(0);
    unsafe { write_rows(png_ptr, image, height) };
}

fn encode_png(
    png_ptr: png_structrp,
    png_state: &PngStructState,
    header_info_state: &PngInfoState,
    trailer_info_state: Option<&PngInfoState>,
    trailer_text_index: usize,
    write_trailer_time: bool,
    write_trailer_exif: bool,
    session: &WriteSessionState,
) -> Result<Vec<u8>, &'static [u8]> {
    let mut bytes = Vec::new();
    let info = build_png_info(header_info_state, session)?;
    let mut encoder =
        Encoder::with_info(&mut bytes, info).map_err(|_| b"write error\0".as_slice())?;
    encoder.set_compression(write_compression(png_state.write_zlib));
    encoder.set_filter(write_filter(png_state));

    let mut writer = encoder
        .write_header()
        .map_err(|_| b"write error\0".as_slice())?;
    write_pre_idat_chunks(png_ptr, &mut writer, header_info_state)
        .map_err(|_| b"write error\0".as_slice())?;

    let idat = compressed_idat_data(png_state, header_info_state, session)?;
    for chunk_bytes in idat.chunks(MAX_IDAT_CHUNK_LEN) {
        writer
            .write_chunk(chunk::IDAT, chunk_bytes)
            .map_err(|_| b"write error\0".as_slice())?;
    }

    if let Some(trailer_info_state) = trailer_info_state {
        write_post_idat_chunks(
            &mut writer,
            trailer_info_state,
            trailer_text_index,
            write_trailer_time,
            write_trailer_exif,
        )
        .map_err(|_| b"write error\0".as_slice())?;
    }
    drop(writer);
    Ok(bytes)
}

pub(crate) unsafe fn write_end(png_ptr: png_structrp, info_ptr: png_inforp) -> bool {
    if png_ptr.is_null() {
        return false;
    }

    let Some(png_state) = state::get_png(png_ptr) else {
        return false;
    };

    if png_state.write_session.is_none() {
        let fallback_info = if info_ptr.is_null() {
            ptr::null()
        } else {
            info_ptr.cast_const()
        };
        if fallback_info.is_null() {
            return false;
        }
        unsafe { begin_write_info(png_ptr, fallback_info) };
    }

    let mut session =
        match state::with_png(png_ptr, |png_state| png_state.write_session.clone()).flatten() {
            Some(session) => session,
            None => return false,
        };
    let header_info_state = match session.header_info.clone().or_else(|| {
        (!info_ptr.is_null())
            .then(|| state::get_info(info_ptr))
            .flatten()
    }) {
        Some(info_state) => info_state,
        None => return false,
    };
    let transform_spec = write_transform_spec(&png_state, &header_info_state);
    if session.rowbytes
        != transform_spec
            .map(|spec| spec.output.rowbytes)
            .unwrap_or(session.rowbytes)
    {
        return false;
    }
    if !session.header_info_ptr.is_null() {
        copy_missing_rows_from_info(session.header_info_ptr, &mut session, transform_spec);
    }
    if !info_ptr.is_null() && info_ptr != session.header_info_ptr {
        copy_missing_rows_from_info(info_ptr, &mut session, transform_spec);
    }
    let trailer_info_state = if info_ptr.is_null() {
        None
    } else {
        state::get_info(info_ptr)
    };
    let same_info_ptr = !info_ptr.is_null() && info_ptr == session.header_info_ptr;
    let trailer_text_index = if same_info_ptr {
        session.header_text_count
    } else {
        0
    };
    let write_trailer_time = !same_info_ptr || !session.wrote_time_in_header;
    let write_trailer_exif = !same_info_ptr || !session.wrote_exif_in_header;

    if crate::bridge_ffi::passthrough_png_if_rows_match(
        png_ptr,
        &session.image_data,
        session.rowbytes,
    ) {
        state::update_png(png_ptr, |png_state| {
            png_state.passthrough_written = true;
            png_state.write_session = Some(session);
        });
        return true;
    }

    let bytes = match encode_png(
        png_ptr,
        &png_state,
        &header_info_state,
        trailer_info_state.as_ref(),
        trailer_text_index,
        write_trailer_time,
        write_trailer_exif,
        &session,
    ) {
        Ok(bytes) => bytes,
        Err(message) => png_error_message(png_ptr, message),
    };

    if let Err(message) = emit_png_bytes(png_ptr, &bytes) {
        png_error_message(png_ptr, message);
    }

    state::update_png(png_ptr, |png_state| {
        png_state.passthrough_written = true;
        png_state.write_session = Some(session);
    });
    true
}
