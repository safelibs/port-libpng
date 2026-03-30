use crate::chunks::{OutputInfo, output_info_for, refresh_output_info, update_read_state};
use crate::types::*;
use core::ffi::c_int;
use core::slice;

unsafe extern "C" {
    fn png_get_rows(png_ptr: png_const_structrp, info_ptr: png_const_inforp) -> png_bytepp;
    fn upstream_png_set_interlace_handling(png_ptr: png_structrp) -> c_int;
}

fn packed_padding_mask(output: OutputInfo) -> Option<u8> {
    if output.width == 0 || output.channels == 0 || output.bit_depth >= 8 {
        return None;
    }

    let pixel_bits = usize::from(output.channels).checked_mul(usize::from(output.bit_depth))?;
    let used_bits = (output.width as usize).checked_mul(pixel_bits)?;
    let padding_bits = (8 - (used_bits % 8)) % 8;
    if padding_bits == 0 {
        None
    } else {
        Some(!(((1u16 << padding_bits) - 1) as u8))
    }
}

pub(crate) unsafe fn mask_packed_row_padding(png_ptr: png_structrp, row: png_bytep) {
    if row.is_null() {
        return;
    }

    let Some(output) = output_info_for(png_ptr) else {
        return;
    };
    let Some(mask) = packed_padding_mask(output) else {
        return;
    };
    if output.rowbytes == 0 {
        return;
    }

    let row_slice = unsafe { slice::from_raw_parts_mut(row, output.rowbytes) };
    if let Some(last) = row_slice.last_mut() {
        *last &= mask;
    }
}

pub(crate) unsafe fn mask_info_rows(png_ptr: png_structrp, info_ptr: png_inforp) {
    let Some(output) = (unsafe { refresh_output_info(png_ptr, info_ptr) }) else {
        return;
    };
    let Some(mask) = packed_padding_mask(output) else {
        return;
    };
    if output.rowbytes == 0 || output.height == 0 {
        return;
    }

    let rows = unsafe { png_get_rows(png_ptr, info_ptr) };
    if rows.is_null() {
        return;
    }

    for row_index in 0..output.height as usize {
        let row_ptr = unsafe { *rows.add(row_index) };
        if row_ptr.is_null() {
            continue;
        }

        let row_slice = unsafe { slice::from_raw_parts_mut(row_ptr, output.rowbytes) };
        if let Some(last) = row_slice.last_mut() {
            *last &= mask;
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_interlace_handling(png_ptr: png_structrp) -> c_int {
    let passes = unsafe { upstream_png_set_interlace_handling(png_ptr) };
    update_read_state(png_ptr, |state| state.interlace_passes = passes);
    passes
}
