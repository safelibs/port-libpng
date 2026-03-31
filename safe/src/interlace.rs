use crate::chunks::read_core;
use crate::read_util::infer_pixel_depth;
use crate::types::*;
use core::ffi::c_int;

const PNG_INTERLACE_TRANSFORM: png_uint_32 = 0x0002;

fn mask_packed_row_padding(row: png_bytep, rowbytes: usize, width: usize, pixel_depth: usize) {
    if row.is_null() || rowbytes == 0 || width == 0 || pixel_depth >= 8 {
        return;
    }

    let Some(used_bits) = width.checked_mul(pixel_depth) else {
        return;
    };
    let padding_bits = (8usize.wrapping_sub(used_bits % 8)) % 8;
    if padding_bits == 0 {
        return;
    }

    let mask = !((1u8 << padding_bits) - 1);
    unsafe {
        *row.add(rowbytes - 1) &= mask;
    }
}

pub(crate) fn sanitize_row_padding(png_ptr: png_structrp, row: png_bytep, display_row: png_bytep) {
    let core = read_core(png_ptr);
    let handled_interlace =
        core.interlaced != 0 && (core.transformations & PNG_INTERLACE_TRANSFORM) != 0;

    if !handled_interlace && core.interlaced != 0 {
        return;
    }

    let rowbytes = if core.rowbytes != 0 {
        core.rowbytes
    } else {
        core.info_rowbytes
    };
    let Some(width) = usize::try_from(core.width).ok() else {
        return;
    };
    let Some(pixel_depth) = infer_pixel_depth(&core, rowbytes) else {
        return;
    };

    mask_packed_row_padding(row, rowbytes, width, pixel_depth);
    mask_packed_row_padding(display_row, rowbytes, width, pixel_depth);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_interlace_handling(png_ptr: png_structrp) -> c_int {
    crate::abi_guard!(png_ptr, {
        if png_ptr.is_null() {
            return 1;
        }

        let mut core = read_core(png_ptr);
        if core.interlaced != 0 {
            core.transformations |= PNG_INTERLACE_TRANSFORM;
            core.num_rows = core.height;
            crate::chunks::write_core(png_ptr, &core);
            7
        } else {
            1
        }
    })
}
