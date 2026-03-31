use crate::chunks::{call_warning, set_read_phase, sync_read_phase_from_core};
use crate::interlace;
use crate::read_util::ReadPhase;
use crate::types::*;
use core::ptr;

const PNG_INTERLACE_TRANSFORM: png_uint_32 = 0x0002;

unsafe extern "C" {
    fn upstream_png_read_info(png_ptr: png_structrp, info_ptr: png_inforp);
    fn upstream_png_read_update_info(png_ptr: png_structrp, info_ptr: png_inforp);
    fn upstream_png_read_row(png_ptr: png_structrp, row: png_bytep, display_row: png_bytep);
    fn upstream_png_read_end(png_ptr: png_structrp, info_ptr: png_inforp);
    fn upstream_png_start_read_image(png_ptr: png_structrp);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_info(png_ptr: png_structrp, info_ptr: png_inforp) {
    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_read_info(png_ptr, info_ptr);
        sync_read_phase_from_core(png_ptr);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_update_info(png_ptr: png_structrp, info_ptr: png_inforp) {
    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_read_update_info(png_ptr, info_ptr);
        set_read_phase(png_ptr, ReadPhase::ImageRows);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_start_read_image(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_start_read_image(png_ptr);
        set_read_phase(png_ptr, ReadPhase::ImageRows);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_row(
    png_ptr: png_structrp,
    row: png_bytep,
    display_row: png_bytep,
) {
    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_read_row(png_ptr, row, display_row);
        if !(row.is_null() && display_row.is_null()) {
            interlace::sanitize_row_padding(png_ptr, row, display_row);
        }
        set_read_phase(png_ptr, ReadPhase::ImageRows);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_rows(
    png_ptr: png_structrp,
    row: png_bytepp,
    display_row: png_bytepp,
    num_rows: png_uint_32,
) {
    crate::abi_guard!(png_ptr, unsafe {
        if png_ptr.is_null() {
            return;
        }

        let mut rp = row;
        let mut dp = display_row;
        for _ in 0..num_rows {
            let rptr = if rp.is_null() {
                ptr::null_mut()
            } else {
                let value = *rp;
                rp = rp.add(1);
                value
            };
            let dptr = if dp.is_null() {
                ptr::null_mut()
            } else {
                let value = *dp;
                dp = dp.add(1);
                value
            };
            png_read_row(png_ptr, rptr, dptr);
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_image(png_ptr: png_structrp, image: png_bytepp) {
    crate::abi_guard!(png_ptr, unsafe {
        if png_ptr.is_null() || image.is_null() {
            return;
        }

        let mut core = crate::chunks::read_core(png_ptr);
        let passes = if (core.flags & crate::common::PNG_FLAG_ROW_INIT) == 0 {
            let passes = crate::interlace::png_set_interlace_handling(png_ptr);
            png_start_read_image(png_ptr);
            passes
        } else {
            if core.interlaced != 0 && (core.transformations & PNG_INTERLACE_TRANSFORM) == 0 {
                let _ = call_warning(
                    png_ptr,
                    b"Interlace handling should be turned on when using png_read_image\0",
                );
                core.num_rows = core.height;
                crate::chunks::write_core(png_ptr, &core);
            }

            crate::interlace::png_set_interlace_handling(png_ptr)
        };

        let image_height = core.height;
        for _pass in 0..passes {
            let mut rows = image;
            for _ in 0..image_height {
                png_read_row(png_ptr, *rows, ptr::null_mut());
                rows = rows.add(1);
            }
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_end(png_ptr: png_structrp, info_ptr: png_inforp) {
    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_read_end(png_ptr, info_ptr);
        set_read_phase(png_ptr, ReadPhase::Terminal);
    });
}
