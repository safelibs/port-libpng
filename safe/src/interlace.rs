use crate::chunks::read_core;
use crate::types::*;
use core::ffi::c_int;

const PNG_INTERLACE_TRANSFORM: png_uint_32 = 0x0002;
const PNG_PACKSWAP: png_uint_32 = 0x10000;
const ADAM7_PASSES: [(u32, u32, u32, u32); 7] = [
    (8, 0, 8, 0),
    (8, 4, 8, 0),
    (4, 0, 8, 4),
    (4, 2, 4, 0),
    (2, 0, 4, 2),
    (2, 1, 2, 0),
    (1, 0, 2, 1),
];

fn uses_adam7_pass_rows(core: &png_safe_read_core) -> bool {
    core.interlaced != 0 && (core.transformations & PNG_INTERLACE_TRANSFORM) == 0
}

fn adam7_pass_samples(width: u32, pass: usize) -> u32 {
    let (x_sampling, x_offset, _, _) = ADAM7_PASSES[pass];
    width.saturating_sub(x_offset).div_ceil(x_sampling)
}

fn sanitize_buffer_padding(buffer: png_bytep, full_bytes: usize, used_mask: u8) {
    if buffer.is_null() {
        return;
    }

    unsafe {
        *buffer.add(full_bytes) &= used_mask;
    }
}

pub(crate) fn sanitize_row_padding_for_core(
    core: &png_safe_read_core,
    row: png_bytep,
    display_row: png_bytep,
) {
    let pixel_depth = usize::from(core.pixel_depth);
    if pixel_depth >= 8 || pixel_depth == 0 {
        return;
    }

    let width = if uses_adam7_pass_rows(core) {
        let Ok(pass) = usize::try_from(core.pass) else {
            return;
        };
        if pass >= ADAM7_PASSES.len() {
            return;
        }
        usize::try_from(adam7_pass_samples(core.width, pass)).unwrap_or(0)
    } else {
        usize::try_from(core.width).unwrap_or(0)
    };
    if width == 0 {
        return;
    }

    let Some(used_bits) = width.checked_mul(pixel_depth) else {
        return;
    };
    let full_bytes = used_bits / 8;
    let tail_bits = used_bits % 8;
    if tail_bits == 0 {
        return;
    }

    let used_mask = if (core.transformations & PNG_PACKSWAP) != 0 {
        (1u8 << tail_bits) - 1
    } else {
        !((1u8 << (8 - tail_bits)) - 1)
    };

    sanitize_buffer_padding(row, full_bytes, used_mask);
    if display_row != row {
        sanitize_buffer_padding(display_row, full_bytes, used_mask);
    }
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
