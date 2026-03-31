use crate::chunks::{call_app_error, read_core, write_core};
use crate::interlace::mask_packed_row_padding_for_width;
use crate::types::*;
use core::ffi::c_int;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

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
const PNG_EXPAND_tRNS: png_uint_32 = 0x2000000;
const PNG_SCALE_16_TO_8: png_uint_32 = 0x4000000;
const PNG_BGR: png_uint_32 = 0x0001;

const PNG_FLAG_ROW_INIT: png_uint_32 = 0x0040;
const PNG_FLAG_DETECT_UNINITIALIZED: png_uint_32 = 0x4000;

const PASS_START_COL: [usize; 7] = [0, 4, 0, 2, 0, 1, 0];
const PASS_COL_OFFSET: [usize; 7] = [8, 8, 4, 4, 2, 2, 1];
const PASS_START_ROW: [usize; 7] = [0, 0, 4, 0, 2, 0, 1];
const PASS_ROW_OFFSET: [usize; 7] = [8, 8, 8, 4, 4, 2, 2];

unsafe extern "C" {
    fn png_safe_call_read_image(png_ptr: png_structrp, image: png_bytepp) -> c_int;
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

#[derive(Clone, Copy, Debug)]
struct RowSlot {
    row_index: usize,
    pass_width: usize,
    start_col: usize,
    col_step: usize,
    present: bool,
}

#[derive(Debug)]
struct BufferedReadState {
    rows: Vec<u8>,
    rowbytes: usize,
    width: usize,
    pixel_depth: usize,
    source_interlaced: bool,
    handled_interlace: bool,
    slots: Vec<RowSlot>,
    next_slot: usize,
}

fn buffered_reads() -> &'static Mutex<HashMap<usize, BufferedReadState>> {
    static BUFFERED_READS: OnceLock<Mutex<HashMap<usize, BufferedReadState>>> = OnceLock::new();
    BUFFERED_READS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn clear_read_state(png_ptr: png_const_structrp) {
    if png_ptr.is_null() {
        return;
    }

    if let Ok(mut states) = buffered_reads().lock() {
        states.remove(&(png_ptr as usize));
    }
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

fn pass_cols(width: usize, pass: usize) -> usize {
    let start = PASS_START_COL[pass];
    if width <= start {
        0
    } else {
        (width - start).div_ceil(PASS_COL_OFFSET[pass])
    }
}

fn row_in_pass(row: usize, pass: usize) -> bool {
    row >= PASS_START_ROW[pass] && (row - PASS_START_ROW[pass]).is_multiple_of(PASS_ROW_OFFSET[pass])
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

fn read_packed_pixel(row: &[u8], x: usize, pixel_depth: usize) -> u8 {
    let bit_offset = x * pixel_depth;
    let byte_index = bit_offset / 8;
    let shift = 8 - pixel_depth - (bit_offset % 8);
    let mask = ((1u16 << pixel_depth) - 1) as u8;
    (row[byte_index] >> shift) & mask
}

fn write_packed_pixel(row: &mut [u8], x: usize, pixel_depth: usize, value: u8) {
    let bit_offset = x * pixel_depth;
    let byte_index = bit_offset / 8;
    let shift = 8 - pixel_depth - (bit_offset % 8);
    let mask = (((1u16 << pixel_depth) - 1) as u8) << shift;
    row[byte_index] = (row[byte_index] & !mask) | ((value << shift) & mask);
}

fn copy_dense_pass_row(
    state: &BufferedReadState,
    slot: RowSlot,
    dst: png_bytep,
) {
    if dst.is_null() || !slot.present || state.pixel_depth == 0 || slot.pass_width == 0 {
        return;
    }

    let src_start = slot.row_index * state.rowbytes;
    let src_end = src_start + state.rowbytes;
    let src_row = &state.rows[src_start..src_end];
    let dst_rowbytes = rowbytes_for_width(slot.pass_width, state.pixel_depth);
    if dst_rowbytes == 0 {
        return;
    }
    let dst_row = unsafe { std::slice::from_raw_parts_mut(dst, dst_rowbytes) };

    if state.pixel_depth >= 8 {
        let pixel_bytes = state.pixel_depth / 8;
        for x in 0..slot.pass_width {
            let src_x = slot.start_col + x * slot.col_step;
            let src_offset = src_x * pixel_bytes;
            let dst_offset = x * pixel_bytes;
            dst_row[dst_offset..dst_offset + pixel_bytes]
                .copy_from_slice(&src_row[src_offset..src_offset + pixel_bytes]);
        }
    } else {
        dst_row.fill(0);
        for x in 0..slot.pass_width {
            let src_x = slot.start_col + x * slot.col_step;
            let value = read_packed_pixel(src_row, src_x, state.pixel_depth);
            write_packed_pixel(dst_row, x, state.pixel_depth, value);
        }
        mask_packed_row_padding_for_width(dst_row, slot.pass_width, state.pixel_depth);
    }
}

fn combine_interlaced_row(
    state: &BufferedReadState,
    slot: RowSlot,
    dst: png_bytep,
    display: bool,
) {
    if dst.is_null() || !slot.present || state.pixel_depth == 0 || state.width == 0 {
        return;
    }

    let src_start = slot.row_index * state.rowbytes;
    let src_end = src_start + state.rowbytes;
    let src_row = &state.rows[src_start..src_end];
    let dst_row = unsafe { std::slice::from_raw_parts_mut(dst, state.rowbytes) };

    if state.pixel_depth >= 8 {
        let pixel_bytes = state.pixel_depth / 8;
        for x in 0..slot.pass_width {
            let src_x = slot.start_col + x * slot.col_step;
            let src_offset = src_x * pixel_bytes;
            let fill_end = if display {
                usize::min(src_x + slot.col_step, state.width)
            } else {
                src_x + 1
            };
            for out_x in src_x..fill_end {
                let dst_offset = out_x * pixel_bytes;
                dst_row[dst_offset..dst_offset + pixel_bytes]
                    .copy_from_slice(&src_row[src_offset..src_offset + pixel_bytes]);
            }
        }
    } else {
        for x in 0..slot.pass_width {
            let src_x = slot.start_col + x * slot.col_step;
            let value = read_packed_pixel(src_row, src_x, state.pixel_depth);
            let fill_end = if display {
                usize::min(src_x + slot.col_step, state.width)
            } else {
                src_x + 1
            };
            for out_x in src_x..fill_end {
                write_packed_pixel(dst_row, out_x, state.pixel_depth, value);
            }
        }
        mask_packed_row_padding_for_width(dst_row, state.width, state.pixel_depth);
    }
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
    palette: png_colorp,
    num_palette: c_int,
    maximum_colors: c_int,
    histogram: png_const_uint_16p,
    full_quantize: c_int,
) {
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

    let mut core = unsafe { read_core(png_ptr) };
    core.transformations |= PNG_QUANTIZE;
    unsafe {
        write_core(png_ptr, &core);
    }
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
    core.transformations |= PNG_SHIFT;
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
    if png_ptr.is_null() {
        return;
    }

    let core = unsafe { read_core(png_ptr) };
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

    let key = png_ptr as usize;

    let need_init = buffered_reads()
        .lock()
        .map(|states| !states.contains_key(&key))
        .unwrap_or(false);

    if need_init {
        let core = unsafe { read_core(png_ptr) };
        let width = match usize::try_from(core.width) {
            Ok(value) if value > 0 => value,
            _ => return,
        };
        let height = match usize::try_from(core.height) {
            Ok(value) if value > 0 => value,
            _ => return,
        };
        let source_interlaced = core.interlaced != 0;
        let handled_interlace =
            source_interlaced && (core.transformations & PNG_INTERLACE) != 0;
        let rowbytes = if core.info_rowbytes != 0 {
            core.info_rowbytes
        } else {
            core.rowbytes
        };
        let total_bytes = match rowbytes.checked_mul(height) {
            Some(value) if value > 0 => value,
            _ => return,
        };

        let mut rows = vec![0u8; total_bytes];
        let mut row_ptrs = Vec::<png_bytep>::with_capacity(height);
        for index in 0..height {
            row_ptrs.push(unsafe { rows.as_mut_ptr().add(index * rowbytes) });
        }

        if unsafe { png_safe_call_read_image(png_ptr, row_ptrs.as_mut_ptr()) } == 0 {
            return;
        }

        let updated_core = unsafe { read_core(png_ptr) };
        let pixel_depth = infer_pixel_depth(updated_core, width, rowbytes);
        if pixel_depth == 0 {
            return;
        }

        let passes = if source_interlaced { 7usize } else { 1usize };
        let mut full_slots = Vec::new();
        let mut present_slots = Vec::new();

        for pass in 0..passes {
            let pass_width = if source_interlaced {
                pass_cols(width, pass)
            } else {
                width
            };
            let start_col = if source_interlaced {
                PASS_START_COL[pass]
            } else {
                0
            };
            let col_step = if source_interlaced {
                PASS_COL_OFFSET[pass]
            } else {
                1
            };

            for row_index in 0..height {
                let present = if source_interlaced {
                    pass_width > 0 && row_in_pass(row_index, pass)
                } else {
                    true
                };
                let slot = RowSlot {
                    row_index,
                    pass_width,
                    start_col,
                    col_step,
                    present,
                };
                full_slots.push(slot);
                if present {
                    present_slots.push(slot);
                }
            }
        }

        let state = BufferedReadState {
            rows,
            rowbytes,
            width,
            pixel_depth,
            source_interlaced,
            handled_interlace,
            slots: if source_interlaced && !handled_interlace {
                present_slots
            } else {
                full_slots
            },
            next_slot: 0,
        };

        if let Ok(mut states) = buffered_reads().lock() {
            states.entry(key).or_insert(state);
        } else {
            return;
        }
    }

    if let Ok(mut states) = buffered_reads().lock() {
        let Some(state) = states.get_mut(&key) else {
            return;
        };

        if state.next_slot >= state.slots.len() {
            return;
        }

        let slot = state.slots[state.next_slot];
        if slot.present {
            if state.source_interlaced && state.handled_interlace {
                combine_interlaced_row(state, slot, row, false);
                combine_interlaced_row(state, slot, display_row, true);
            } else {
                copy_dense_pass_row(state, slot, row);
                copy_dense_pass_row(state, slot, display_row);
            }
        }

        state.next_slot += 1;
    }
}
