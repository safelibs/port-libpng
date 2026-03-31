use crate::chunks::{read_core, write_core};
use crate::types::*;
use core::ffi::c_int;

const PNG_INTERLACE: png_uint_32 = 0x0002;

pub(crate) fn mask_packed_row_padding_for_width(
    row: &mut [u8],
    width: usize,
    pixel_depth: usize,
) {
    if width == 0 || pixel_depth >= 8 || row.is_empty() {
        return;
    }

    let used_bits = match width.checked_mul(pixel_depth) {
        Some(bits) => bits,
        None => return,
    };
    let padding_bits = (8 - (used_bits % 8)) % 8;
    if padding_bits == 0 {
        return;
    }

    let mask = !(((1u16 << padding_bits) - 1) as u8);
    if let Some(last) = row.last_mut() {
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
