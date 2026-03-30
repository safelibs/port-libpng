use crate::chunks::{call_app_error, read_core, write_core};
use crate::interlace::mask_packed_row_padding;
use crate::types::*;
use core::ffi::c_int;
use core::ptr;
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
}

#[derive(Debug)]
struct BufferedReadState {
    rows: Vec<u8>,
    rowbytes: usize,
    height: usize,
    passes: usize,
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

    let key = png_ptr as usize;

    let need_init = buffered_reads()
        .lock()
        .map(|states| !states.contains_key(&key))
        .unwrap_or(false);

    if need_init {
        let core = unsafe { read_core(png_ptr) };
        let height = match usize::try_from(core.height) {
            Ok(value) if value > 0 => value,
            _ => return,
        };
        let explicit_interlace = core.interlaced != 0 && (core.transformations & PNG_INTERLACE) != 0;
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

        let passes = if explicit_interlace {
            7usize
        } else {
            1usize
        };

        let state = BufferedReadState {
            rows,
            rowbytes,
            height,
            passes,
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

        let Some(total_slots) = state.height.checked_mul(state.passes) else {
            return;
        };
        if state.next_slot >= total_slots {
            return;
        }

        let row_index = state.next_slot % state.height;
        let start = row_index * state.rowbytes;
        let end = start + state.rowbytes;
        let src = &state.rows[start..end];

        if !row.is_null() {
            unsafe {
                ptr::copy_nonoverlapping(src.as_ptr(), row, src.len());
            }
        }
        if !display_row.is_null() {
            unsafe {
                ptr::copy_nonoverlapping(src.as_ptr(), display_row, src.len());
            }
        }

        state.next_slot += 1;
    }

    unsafe {
        mask_packed_row_padding(png_ptr, row);
        mask_packed_row_padding(png_ptr, display_row);
    }
}
