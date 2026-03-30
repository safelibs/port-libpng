use crate::chunks::{read_core, write_core};
use crate::types::*;
use core::ffi::c_int;
use core::slice;

const PNG_INTERLACE: png_uint_32 = 0x0002;

fn padding_mask(width: png_uint_32, pixel_depth: png_byte) -> Option<u8> {
    if width == 0 || pixel_depth >= 8 {
        return None;
    }

    let used_bits = (width as usize).checked_mul(usize::from(pixel_depth))?;
    let padding_bits = (8 - (used_bits % 8)) % 8;
    if padding_bits == 0 {
        None
    } else {
        Some(!(((1u16 << padding_bits) - 1) as u8))
    }
}

pub(crate) unsafe fn mask_packed_row_padding(png_ptr: png_structrp, row: png_bytep) {
    if png_ptr.is_null() || row.is_null() {
        return;
    }

    let core = unsafe { read_core(png_ptr) };
    let Some(mask) = padding_mask(core.width, core.transformed_pixel_depth) else {
        return;
    };
    if core.rowbytes == 0 {
        return;
    }

    let row_slice = unsafe { slice::from_raw_parts_mut(row, core.rowbytes) };
    if let Some(last) = row_slice.last_mut() {
        *last &= mask;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_interlace_handling(png_ptr: png_structrp) -> c_int {
    if png_ptr.is_null() {
        return 1;
    }

    let mut core = unsafe { read_core(png_ptr) };
    if core.interlaced != 0 {
        core.transformations |= PNG_INTERLACE;
        unsafe {
            write_core(png_ptr, &core);
        }
        7
    } else {
        1
    }
}
