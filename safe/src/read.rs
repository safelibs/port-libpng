use crate::chunks::{
    call_app_error, call_benign_error, call_error, call_warning, read_core, read_info_core,
    set_read_phase, sync_read_phase_from_core, sync_unknown_chunk_policy_to_upstream,
    validate_parser_chunk, write_core, write_info_core,
};
use crate::common::PNG_FLAG_ROW_INIT;
use crate::interlace;
use crate::read_util::ReadPhase;
use crate::state;
use crate::types::*;
use core::ptr;

const PNG_INTERLACE_TRANSFORM: png_uint_32 = 0x0002;
const PNG_HAVE_IHDR: png_uint_32 = 0x01;
const PNG_HAVE_PLTE: png_uint_32 = 0x02;
const PNG_HAVE_IDAT: png_uint_32 = 0x04;
const PNG_AFTER_IDAT: png_uint_32 = 0x08;
const PNG_HAVE_CHUNK_AFTER_IDAT: png_uint_32 = 0x2000;
const PNG_FLAG_ZSTREAM_ENDED: png_uint_32 = 0x0008;
const PNG_IDAT: png_uint_32 = u32::from_be_bytes(*b"IDAT");
const PNG_IEND: png_uint_32 = u32::from_be_bytes(*b"IEND");
const PNG_IHDR: png_uint_32 = u32::from_be_bytes(*b"IHDR");
const PNG_PLTE: png_uint_32 = u32::from_be_bytes(*b"PLTE");
const PNG_BKGD: png_uint_32 = u32::from_be_bytes(*b"bKGD");
const PNG_CHRM: png_uint_32 = u32::from_be_bytes(*b"cHRM");
const PNG_EXIF: png_uint_32 = u32::from_be_bytes(*b"eXIf");
const PNG_GAMA: png_uint_32 = u32::from_be_bytes(*b"gAMA");
const PNG_HIST: png_uint_32 = u32::from_be_bytes(*b"hIST");
const PNG_OFFS: png_uint_32 = u32::from_be_bytes(*b"oFFs");
const PNG_PCAL: png_uint_32 = u32::from_be_bytes(*b"pCAL");
const PNG_SCAL: png_uint_32 = u32::from_be_bytes(*b"sCAL");
const PNG_PHYS: png_uint_32 = u32::from_be_bytes(*b"pHYs");
const PNG_SBIT: png_uint_32 = u32::from_be_bytes(*b"sBIT");
const PNG_SRGB: png_uint_32 = u32::from_be_bytes(*b"sRGB");
const PNG_ICCP: png_uint_32 = u32::from_be_bytes(*b"iCCP");
const PNG_SPLT: png_uint_32 = u32::from_be_bytes(*b"sPLT");
const PNG_TEXT: png_uint_32 = u32::from_be_bytes(*b"tEXt");
const PNG_TIME: png_uint_32 = u32::from_be_bytes(*b"tIME");
const PNG_TRNS: png_uint_32 = u32::from_be_bytes(*b"tRNS");
const PNG_ZTXT: png_uint_32 = u32::from_be_bytes(*b"zTXt");
const PNG_ITXT: png_uint_32 = u32::from_be_bytes(*b"iTXt");

const DISPATCH_IHDR: i32 = 1;
const DISPATCH_IEND: i32 = 2;
const DISPATCH_PLTE: i32 = 3;
const DISPATCH_BKGD: i32 = 4;
const DISPATCH_CHRM: i32 = 5;
const DISPATCH_EXIF: i32 = 6;
const DISPATCH_GAMA: i32 = 7;
const DISPATCH_HIST: i32 = 8;
const DISPATCH_OFFS: i32 = 9;
const DISPATCH_PCAL: i32 = 10;
const DISPATCH_SCAL: i32 = 11;
const DISPATCH_PHYS: i32 = 12;
const DISPATCH_SBIT: i32 = 13;
const DISPATCH_SRGB: i32 = 14;
const DISPATCH_ICCP: i32 = 15;
const DISPATCH_SPLT: i32 = 16;
const DISPATCH_TEXT: i32 = 17;
const DISPATCH_TIME: i32 = 18;
const DISPATCH_TRNS: i32 = 19;
const DISPATCH_ZTXT: i32 = 20;
const DISPATCH_ITXT: i32 = 21;
const DISPATCH_UNKNOWN: i32 = 22;

unsafe extern "C" {
    fn png_safe_call_read_sig(png_ptr: png_structrp, info_ptr: png_inforp) -> core::ffi::c_int;
    fn png_safe_call_read_chunk_header(
        png_ptr: png_structrp,
        length_out: *mut png_uint_32,
    ) -> core::ffi::c_int;
    fn png_safe_call_dispatch_chunk(
        png_ptr: png_structrp,
        info_ptr: png_inforp,
        length: png_uint_32,
        dispatch: core::ffi::c_int,
        keep: core::ffi::c_int,
    ) -> core::ffi::c_int;
    fn png_safe_call_crc_finish(
        png_ptr: png_structrp,
        skip: png_uint_32,
        crc_result: *mut core::ffi::c_int,
    ) -> core::ffi::c_int;
    fn png_safe_call_read_row(
        png_ptr: png_structrp,
        row: png_bytep,
        display_row: png_bytep,
    ) -> core::ffi::c_int;
    fn png_safe_call_read_finish_idat(png_ptr: png_structrp) -> core::ffi::c_int;
    fn png_safe_call_read_start_row(png_ptr: png_structrp) -> core::ffi::c_int;
    fn png_safe_call_read_transform_info(
        png_ptr: png_structrp,
        info_ptr: png_inforp,
    ) -> core::ffi::c_int;
    fn png_chunk_unknown_handling(
        png_ptr: png_const_structrp,
        chunk_name: png_uint_32,
    ) -> core::ffi::c_int;
}

#[derive(Clone)]
struct ParseSnapshot {
    core: png_safe_read_core,
    png_state: Option<state::PngStructState>,
    info_core: Option<png_safe_info_core>,
    info_state: Option<state::PngInfoState>,
}

fn snapshot_parse_state(png_ptr: png_structrp, info_ptr: png_inforp) -> ParseSnapshot {
    ParseSnapshot {
        core: read_core(png_ptr),
        png_state: state::get_png(png_ptr),
        info_core: (!info_ptr.is_null()).then(|| read_info_core(info_ptr)),
        info_state: state::get_info(info_ptr),
    }
}

fn rollback_parse_state(png_ptr: png_structrp, info_ptr: png_inforp, snapshot: &ParseSnapshot) {
    write_core(png_ptr, &snapshot.core);
    if let Some(info_core) = snapshot.info_core.as_ref() {
        write_info_core(info_ptr, info_core);
    }
    if let Some(png_state) = snapshot.png_state.clone() {
        state::register_png(png_ptr, png_state);
    }
    if let Some(info_state) = snapshot.info_state {
        state::register_info(info_ptr, info_state);
    }
}

unsafe fn rollback_and_rethrow(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
) -> ! {
    rollback_parse_state(png_ptr, info_ptr, snapshot);
    crate::error::png_longjmp(png_ptr, 1)
}

unsafe fn error_and_rethrow(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    message: &'static [u8],
) -> ! {
    let _ = call_error(png_ptr, message);
    rollback_and_rethrow(png_ptr, info_ptr, snapshot)
}

fn keep_for_chunk(png_ptr: png_structrp, chunk_name: png_uint_32) -> core::ffi::c_int {
    unsafe { png_chunk_unknown_handling(png_ptr, chunk_name) }
}

fn dispatch_for_chunk(chunk_name: png_uint_32) -> Option<core::ffi::c_int> {
    Some(match chunk_name {
        PNG_IHDR => DISPATCH_IHDR,
        PNG_IEND => DISPATCH_IEND,
        PNG_PLTE => DISPATCH_PLTE,
        PNG_BKGD => DISPATCH_BKGD,
        PNG_CHRM => DISPATCH_CHRM,
        PNG_EXIF => DISPATCH_EXIF,
        PNG_GAMA => DISPATCH_GAMA,
        PNG_HIST => DISPATCH_HIST,
        PNG_OFFS => DISPATCH_OFFS,
        PNG_PCAL => DISPATCH_PCAL,
        PNG_SCAL => DISPATCH_SCAL,
        PNG_PHYS => DISPATCH_PHYS,
        PNG_SBIT => DISPATCH_SBIT,
        PNG_SRGB => DISPATCH_SRGB,
        PNG_ICCP => DISPATCH_ICCP,
        PNG_SPLT => DISPATCH_SPLT,
        PNG_TEXT => DISPATCH_TEXT,
        PNG_TIME => DISPATCH_TIME,
        PNG_TRNS => DISPATCH_TRNS,
        PNG_ZTXT => DISPATCH_ZTXT,
        PNG_ITXT => DISPATCH_ITXT,
        _ => return None,
    })
}

unsafe fn call_dispatch_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    length: png_uint_32,
    dispatch: core::ffi::c_int,
    keep: core::ffi::c_int,
    snapshot: &ParseSnapshot,
) {
    if png_safe_call_dispatch_chunk(png_ptr, info_ptr, length, dispatch, keep) == 0 {
        rollback_and_rethrow(png_ptr, info_ptr, snapshot);
    }
}

unsafe fn read_chunk_header_or_rethrow(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
) -> png_uint_32 {
    let mut length = 0;
    if png_safe_call_read_chunk_header(png_ptr, &mut length) == 0 {
        rollback_and_rethrow(png_ptr, info_ptr, snapshot);
    }
    length
}

unsafe fn read_info_loop(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
) {
    loop {
        let length = read_chunk_header_or_rethrow(png_ptr, info_ptr, snapshot);
        let mut core = read_core(png_ptr);
        let chunk_name = core.chunk_name;

        if let Err(message) = validate_parser_chunk(png_ptr, chunk_name, length) {
            error_and_rethrow(png_ptr, info_ptr, snapshot, message);
        }

        if chunk_name == PNG_IDAT {
            if (core.mode & PNG_HAVE_IHDR) == 0 {
                error_and_rethrow(png_ptr, info_ptr, snapshot, b"Missing IHDR before IDAT\0");
            } else if core.color_type == 3 && (core.mode & PNG_HAVE_PLTE) == 0 {
                error_and_rethrow(png_ptr, info_ptr, snapshot, b"Missing PLTE before IDAT\0");
            } else if (core.mode & PNG_AFTER_IDAT) != 0 {
                let _ = call_benign_error(png_ptr, b"Too many IDATs found\0");
            }

            core.mode |= PNG_HAVE_IDAT;
            write_core(png_ptr, &core);
        } else if (core.mode & PNG_HAVE_IDAT) != 0 {
            core.mode |= PNG_HAVE_CHUNK_AFTER_IDAT | PNG_AFTER_IDAT;
            write_core(png_ptr, &core);
        }

        let keep = keep_for_chunk(png_ptr, chunk_name);
        if keep != 0 {
            call_dispatch_chunk(png_ptr, info_ptr, length, DISPATCH_UNKNOWN, keep, snapshot);
            if chunk_name == PNG_PLTE {
                let mut updated = read_core(png_ptr);
                updated.mode |= PNG_HAVE_PLTE;
                write_core(png_ptr, &updated);
            } else if chunk_name == PNG_IDAT {
                let mut updated = read_core(png_ptr);
                updated.idat_size = 0;
                write_core(png_ptr, &updated);
                break;
            }
            continue;
        }

        if let Some(dispatch) = dispatch_for_chunk(chunk_name) {
            call_dispatch_chunk(png_ptr, info_ptr, length, dispatch, 0, snapshot);
        } else if chunk_name == PNG_IDAT {
            let mut updated = read_core(png_ptr);
            updated.idat_size = length;
            write_core(png_ptr, &updated);
            break;
        } else {
            call_dispatch_chunk(png_ptr, info_ptr, length, DISPATCH_UNKNOWN, 0, snapshot);
        }
    }
}

unsafe fn read_end_loop(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
) {
    if keep_for_chunk(png_ptr, PNG_IDAT) == 0 && png_safe_call_read_finish_idat(png_ptr) == 0 {
        rollback_and_rethrow(png_ptr, info_ptr, snapshot);
    }

    loop {
        let length = read_chunk_header_or_rethrow(png_ptr, info_ptr, snapshot);
        let mut core = read_core(png_ptr);
        let chunk_name = core.chunk_name;

        if let Err(message) = validate_parser_chunk(png_ptr, chunk_name, length) {
            error_and_rethrow(png_ptr, info_ptr, snapshot, message);
        }

        if chunk_name != PNG_IDAT {
            core.mode |= PNG_HAVE_CHUNK_AFTER_IDAT;
            write_core(png_ptr, &core);
        }

        let keep = keep_for_chunk(png_ptr, chunk_name);
        if chunk_name == PNG_IEND {
            call_dispatch_chunk(png_ptr, info_ptr, length, DISPATCH_IEND, 0, snapshot);
        } else if chunk_name == PNG_IHDR {
            call_dispatch_chunk(png_ptr, info_ptr, length, DISPATCH_IHDR, 0, snapshot);
        } else if info_ptr.is_null() {
            let mut crc_result = 0;
            if png_safe_call_crc_finish(png_ptr, length, &mut crc_result) == 0 {
                rollback_and_rethrow(png_ptr, info_ptr, snapshot);
            }
        } else if keep != 0 {
            if chunk_name == PNG_IDAT {
                let core = read_core(png_ptr);
                if (length > 0 && (core.flags & PNG_FLAG_ZSTREAM_ENDED) == 0)
                    || (core.mode & PNG_HAVE_CHUNK_AFTER_IDAT) != 0
                {
                    let _ = call_benign_error(png_ptr, b".Too many IDATs found\0");
                }
            }

            call_dispatch_chunk(png_ptr, info_ptr, length, DISPATCH_UNKNOWN, keep, snapshot);
            if chunk_name == PNG_PLTE {
                let mut updated = read_core(png_ptr);
                updated.mode |= PNG_HAVE_PLTE;
                write_core(png_ptr, &updated);
            }
        } else if chunk_name == PNG_IDAT {
            let core = read_core(png_ptr);
            if (length > 0 && (core.flags & PNG_FLAG_ZSTREAM_ENDED) == 0)
                || (core.mode & PNG_HAVE_CHUNK_AFTER_IDAT) != 0
            {
                let _ = call_benign_error(png_ptr, b"..Too many IDATs found\0");
            }

            let mut crc_result = 0;
            if png_safe_call_crc_finish(png_ptr, length, &mut crc_result) == 0 {
                rollback_and_rethrow(png_ptr, info_ptr, snapshot);
            }
        } else if let Some(dispatch) = dispatch_for_chunk(chunk_name) {
            call_dispatch_chunk(png_ptr, info_ptr, length, dispatch, 0, snapshot);
        } else {
            call_dispatch_chunk(png_ptr, info_ptr, length, DISPATCH_UNKNOWN, 0, snapshot);
        }

        sync_read_phase_from_core(png_ptr);
        if matches!(state::get_png(png_ptr).map(|state| state.read_phase), Some(ReadPhase::Terminal))
        {
            break;
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_info(png_ptr: png_structrp, info_ptr: png_inforp) {
    crate::abi_guard!(png_ptr, unsafe {
        let snapshot = snapshot_parse_state(png_ptr, info_ptr);
        sync_unknown_chunk_policy_to_upstream(png_ptr);
        if png_safe_call_read_sig(png_ptr, info_ptr) == 0 {
            rollback_and_rethrow(png_ptr, info_ptr, &snapshot);
        }
        read_info_loop(png_ptr, info_ptr, &snapshot);
        sync_read_phase_from_core(png_ptr);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_update_info(png_ptr: png_structrp, info_ptr: png_inforp) {
    crate::abi_guard!(png_ptr, unsafe {
        let snapshot = snapshot_parse_state(png_ptr, info_ptr);
        if (read_core(png_ptr).flags & PNG_FLAG_ROW_INIT) == 0 {
            if png_safe_call_read_start_row(png_ptr) == 0 {
                rollback_and_rethrow(png_ptr, info_ptr, &snapshot);
            }
            if png_safe_call_read_transform_info(png_ptr, info_ptr) == 0 {
                rollback_and_rethrow(png_ptr, info_ptr, &snapshot);
            }
        } else {
            let _ = call_app_error(
                png_ptr,
                b"png_read_update_info/png_start_read_image: duplicate call\0",
            );
        }
        set_read_phase(png_ptr, ReadPhase::ImageRows);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_start_read_image(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, unsafe {
        let snapshot = snapshot_parse_state(png_ptr, ptr::null_mut());
        if (read_core(png_ptr).flags & PNG_FLAG_ROW_INIT) == 0 {
            if png_safe_call_read_start_row(png_ptr) == 0 {
                rollback_and_rethrow(png_ptr, ptr::null_mut(), &snapshot);
            }
        } else {
            let _ = call_app_error(
                png_ptr,
                b"png_start_read_image/png_read_update_info: duplicate call\0",
            );
        }
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
        let snapshot = snapshot_parse_state(png_ptr, ptr::null_mut());
        if png_safe_call_read_row(png_ptr, row, display_row) == 0 {
            rollback_and_rethrow(png_ptr, ptr::null_mut(), &snapshot);
        }
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
        let snapshot = snapshot_parse_state(png_ptr, info_ptr);
        sync_unknown_chunk_policy_to_upstream(png_ptr);
        read_end_loop(png_ptr, info_ptr, &snapshot);
        set_read_phase(png_ptr, ReadPhase::Terminal);
    });
}
