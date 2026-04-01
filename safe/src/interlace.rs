use crate::chunks::read_core;
use crate::types::*;
use core::ffi::c_int;

const PNG_INTERLACE_TRANSFORM: png_uint_32 = 0x0002;

pub(crate) fn sanitize_row_padding(_png_ptr: png_structrp, _row: png_bytep, _display_row: png_bytep) {
    // libpng preserves the caller's padding bits in packed rows.
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
