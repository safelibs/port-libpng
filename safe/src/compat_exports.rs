use crate::chunks::{call_app_error, call_warning, read_core, write_core};
use crate::common::{
    PNG_FREE_ROWS, PNG_FREE_UNKN, PNG_INFO_IDAT, PNG_INFO_pHYs, PNG_INFO_sBIT, PNG_UINT_31_MAX,
};
use crate::read_util::PNG_SIGNATURE;
use crate::state::{self, OwnedUnknownChunk};
use crate::types::*;
use core::ffi::c_int;
use core::mem::size_of;
use core::ptr;

const PNG_FLAG_ROW_INIT: png_uint_32 = 0x0040;
const PNG_FLAG_DETECT_UNINITIALIZED: png_uint_32 = 0x4000;
const PNG_FLAG_ZSTREAM_ENDED: png_uint_32 = 0x0008;
const PNG_FLAG_CRC_ANCILLARY_USE: png_uint_32 = 0x0100;
const PNG_FLAG_CRC_ANCILLARY_NOWARN: png_uint_32 = 0x0200;
const PNG_FLAG_CRC_CRITICAL_USE: png_uint_32 = 0x0400;
const PNG_FLAG_CRC_CRITICAL_IGNORE: png_uint_32 = 0x0800;
const PNG_FLAG_CRC_ANCILLARY_MASK: png_uint_32 =
    PNG_FLAG_CRC_ANCILLARY_USE | PNG_FLAG_CRC_ANCILLARY_NOWARN;
const PNG_FLAG_CRC_CRITICAL_MASK: png_uint_32 =
    PNG_FLAG_CRC_CRITICAL_USE | PNG_FLAG_CRC_CRITICAL_IGNORE;
const PNG_HAVE_IHDR: png_uint_32 = 0x01;
const PNG_HAVE_PLTE: png_uint_32 = 0x02;
const PNG_AFTER_IDAT: png_uint_32 = 0x08;
const PNG_RESOLUTION_METER: c_int = 1;
const PNG_OFFSET_PIXEL: c_int = 0;
const PNG_OFFSET_MICROMETER: c_int = 1;
const PNG_CRC_DEFAULT: c_int = 0;
const PNG_CRC_ERROR_QUIT: c_int = 1;
const PNG_CRC_WARN_DISCARD: c_int = 2;
const PNG_CRC_WARN_USE: c_int = 3;
const PNG_CRC_QUIET_USE: c_int = 4;
const PNG_CRC_NO_CHANGE: c_int = 5;
const PNG_FP_1: png_fixed_point = 100_000;
const PNG_COLORSPACE_HAVE_GAMMA: png_uint_16 = 0x0001;
const PNG_EXPAND: png_uint_32 = 0x1000;
const PNG_STRIP_ALPHA: png_uint_32 = 0x40000;
const PNG_TRANSFORM_STRIP_16: c_int = 0x0001;
const PNG_TRANSFORM_STRIP_ALPHA: c_int = 0x0002;
const PNG_TRANSFORM_PACKING: c_int = 0x0004;
const PNG_TRANSFORM_PACKSWAP: c_int = 0x0008;
const PNG_TRANSFORM_EXPAND: c_int = 0x0010;
const PNG_TRANSFORM_INVERT_MONO: c_int = 0x0020;
const PNG_TRANSFORM_SHIFT: c_int = 0x0040;
const PNG_TRANSFORM_BGR: c_int = 0x0080;
const PNG_TRANSFORM_SWAP_ALPHA: c_int = 0x0100;
const PNG_TRANSFORM_SWAP_ENDIAN: c_int = 0x0200;
const PNG_TRANSFORM_INVERT_ALPHA: c_int = 0x0400;
const PNG_TRANSFORM_GRAY_TO_RGB: c_int = 0x2000;
const PNG_TRANSFORM_EXPAND_16: c_int = 0x4000;
const PNG_TRANSFORM_SCALE_16: c_int = 0x8000;
const PNG_ALL_MNG_FEATURES: png_uint_32 = 0x05;
const Z_OK: c_int = 0;
const Z_STREAM_ERROR: c_int = -2;

fn read_transform_ok(png_ptr: png_structrp, need_ihdr: bool) -> bool {
    if png_ptr.is_null() {
        return false;
    }

    let mut core = read_core(png_ptr);
    if (core.flags & PNG_FLAG_ROW_INIT) != 0 {
        let _ = unsafe {
            call_app_error(
                png_ptr,
                b"invalid after png_start_read_image or png_read_update_info\0",
            )
        };
        return false;
    }

    if need_ihdr && (core.mode & PNG_HAVE_IHDR) == 0 {
        let _ =
            unsafe { call_app_error(png_ptr, b"invalid before the PNG header has been read\0") };
        return false;
    }

    core.flags |= PNG_FLAG_DETECT_UNINITIALIZED;
    write_core(png_ptr, &core);
    true
}

fn apply_transform_mask(png_ptr: png_structrp, mask: png_uint_32) {
    if !read_transform_ok(png_ptr, false) {
        return;
    }

    let mut core = read_core(png_ptr);
    core.transformations |= mask;
    write_core(png_ptr, &core);
}

fn gamma_fixed_from_double(png_ptr: png_structrp, value: f64) -> png_fixed_point {
    if !value.is_finite() || value <= 0.0 {
        unsafe {
            crate::error::png_error(png_ptr, b"invalid gamma value\0".as_ptr().cast());
        }
    }

    let scaled = value * f64::from(PNG_FP_1);
    if scaled < 1.0 || scaled > f64::from(i32::MAX) {
        unsafe {
            crate::error::png_error(png_ptr, b"invalid gamma value\0".as_ptr().cast());
        }
    }

    scaled.round() as png_fixed_point
}

fn normalize_unknown_chunk_location(png_ptr: png_const_structrp, mut location: c_int) -> png_byte {
    location &= (PNG_HAVE_IHDR | PNG_HAVE_PLTE | PNG_AFTER_IDAT) as c_int;

    if location == 0 {
        let mode = read_core(png_ptr.cast_mut()).mode;
        let is_read_struct =
            state::with_png(png_ptr.cast_mut(), |png_state| png_state.is_read_struct)
                .unwrap_or(false);
        if !is_read_struct {
            location = (mode & (PNG_HAVE_IHDR | PNG_HAVE_PLTE | PNG_AFTER_IDAT)) as c_int;
        }
        if location == 0 {
            location = PNG_HAVE_IHDR as c_int;
        }
    }

    while location != (location & -location) {
        location &= !(location & -location);
    }

    location as png_byte
}

fn rebuild_unknown_chunk_cache(info_state: &mut state::PngInfoState) {
    let mut cache = Vec::with_capacity(info_state.unknown_chunks.len());
    for chunk in &mut info_state.unknown_chunks {
        cache.push(png_unknown_chunk {
            name: chunk.name,
            data: if chunk.data.is_empty() {
                ptr::null_mut()
            } else {
                chunk.data.as_mut_ptr()
            },
            size: chunk.data.len(),
            location: chunk.location,
        });
    }
    info_state.unknown_chunk_cache = cache;
}

unsafe fn longjmp_on_zero(png_ptr: png_structrp, status: c_int) {
    let error_callback_called =
        state::with_png(png_ptr, |png_state| png_state.error_callback_called).unwrap_or(false);
    state::update_png(png_ptr, |png_state| {
        png_state.error_callback_called = false;
    });
    if status == 0 {
        if error_callback_called {
            unsafe { crate::error::png_longjmp(png_ptr, 1) };
        } else {
            unsafe { crate::error::png_error(png_ptr, c"Read Error".as_ptr()) };
        }
    }
}

fn read_phys(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> Option<(png_uint_32, png_uint_32, c_int)> {
    let mut res_x = 0;
    let mut res_y = 0;
    let mut unit_type = 0;
    let valid = unsafe {
        crate::get::png_get_pHYs(png_ptr, info_ptr, &mut res_x, &mut res_y, &mut unit_type)
    };
    if (valid & PNG_INFO_pHYs) != 0 {
        Some((res_x, res_y, unit_type))
    } else {
        None
    }
}

fn read_offsets(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> Option<(png_int_32, png_int_32, c_int)> {
    let mut x = 0;
    let mut y = 0;
    let mut unit_type = 0;
    let valid =
        unsafe { crate::get::png_get_oFFs(png_ptr, info_ptr, &mut x, &mut y, &mut unit_type) };
    if valid != 0 {
        Some((x, y, unit_type))
    } else {
        None
    }
}

fn ppi_from_ppm(ppm: png_uint_32) -> png_uint_32 {
    if ppm > PNG_UINT_31_MAX {
        return 0;
    }

    (((ppm as u64) * 127 + 2_500) / 5_000) as png_uint_32
}

fn checked_fixed_point(value: i128) -> Option<png_fixed_point> {
    if value < i128::from(i32::MIN) || value > i128::from(i32::MAX) {
        return None;
    }

    Some(value as png_fixed_point)
}

fn fixed_inches_from_microns(microns: png_int_32) -> png_fixed_point {
    let value = i128::from(microns) * 500;
    let rounded = if value < 0 {
        (value - 63) / 127
    } else {
        (value + 63) / 127
    };

    checked_fixed_point(rounded).unwrap_or(0)
}

unsafe fn apply_read_png_transforms(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    transforms: c_int,
) {
    if (transforms & PNG_TRANSFORM_SCALE_16) != 0 {
        unsafe { crate::read_transform::png_set_scale_16(png_ptr) };
    }
    if (transforms & PNG_TRANSFORM_STRIP_16) != 0 {
        unsafe { crate::read_transform::png_set_strip_16(png_ptr) };
    }
    if (transforms & PNG_TRANSFORM_STRIP_ALPHA) != 0 {
        unsafe { png_set_strip_alpha(png_ptr) };
    }
    if (transforms & PNG_TRANSFORM_PACKING) != 0 {
        unsafe { crate::write_transform::png_set_packing(png_ptr) };
    }
    if (transforms & PNG_TRANSFORM_PACKSWAP) != 0 {
        unsafe { crate::write_transform::png_set_packswap(png_ptr) };
    }
    if (transforms & PNG_TRANSFORM_EXPAND) != 0 {
        unsafe { crate::read_transform::png_set_expand(png_ptr) };
    }
    if (transforms & PNG_TRANSFORM_INVERT_MONO) != 0 {
        unsafe { crate::read_transform::png_set_invert_mono(png_ptr) };
    }
    if (transforms & PNG_TRANSFORM_SHIFT) != 0 {
        let mut sig_bit = ptr::null_mut();
        if (unsafe { crate::get::png_get_sBIT(png_ptr, info_ptr, &mut sig_bit) } & PNG_INFO_sBIT)
            != 0
            && !sig_bit.is_null()
        {
            unsafe { crate::read_transform::png_set_shift(png_ptr, sig_bit) };
        }
    }
    if (transforms & PNG_TRANSFORM_BGR) != 0 {
        unsafe { crate::read_transform::png_set_bgr(png_ptr) };
    }
    if (transforms & PNG_TRANSFORM_SWAP_ALPHA) != 0 {
        unsafe { crate::read_transform::png_set_swap_alpha(png_ptr) };
    }
    if (transforms & PNG_TRANSFORM_SWAP_ENDIAN) != 0 {
        unsafe { crate::write_transform::png_set_swap(png_ptr) };
    }
    if (transforms & PNG_TRANSFORM_INVERT_ALPHA) != 0 {
        unsafe { crate::read_transform::png_set_invert_alpha(png_ptr) };
    }
    if (transforms & PNG_TRANSFORM_GRAY_TO_RGB) != 0 {
        unsafe { crate::read_transform::png_set_gray_to_rgb(png_ptr) };
    }
    if (transforms & PNG_TRANSFORM_EXPAND_16) != 0 {
        unsafe { crate::read_transform::png_set_expand_16(png_ptr) };
    }
}

unsafe fn ensure_read_png_rows(png_ptr: png_structrp, info_ptr: png_inforp) {
    if !unsafe { crate::get::png_get_rows(png_ptr, info_ptr) }.is_null() {
        return;
    }

    let height = unsafe { crate::get::png_get_image_height(png_ptr, info_ptr) };
    let rowbytes = unsafe { crate::get::png_get_rowbytes(png_ptr, info_ptr) };
    if height == 0 || rowbytes == 0 {
        return;
    }

    let height_usize = usize::try_from(height).unwrap_or(usize::MAX);
    let Some(pointer_bytes) = height_usize.checked_mul(size_of::<png_bytep>()) else {
        unsafe {
            crate::error::png_error(
                png_ptr,
                b"Image is too high to process with png_read_png()\0"
                    .as_ptr()
                    .cast(),
            );
        }
    };

    let row_pointers =
        unsafe { crate::memory::png_malloc(png_ptr, pointer_bytes) }.cast::<png_bytep>();
    if row_pointers.is_null() {
        return;
    }

    for index in 0..height_usize {
        unsafe { *row_pointers.add(index) = ptr::null_mut() };
    }

    for index in 0..height_usize {
        let row = unsafe { crate::memory::png_malloc(png_ptr, rowbytes) }.cast::<png_byte>();
        if row.is_null() {
            return;
        }
        unsafe { *row_pointers.add(index) = row };
    }

    state::update_info(info_ptr, |info_state| {
        info_state.core.row_pointers = row_pointers;
        info_state.core.free_me |= PNG_FREE_ROWS;
    });
}

pub(crate) unsafe fn store_unknown_chunks_impl(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    unknowns: png_const_unknown_chunkp,
    num_unknowns: c_int,
) {
    if png_ptr.is_null() || info_ptr.is_null() || unknowns.is_null() || num_unknowns <= 0 {
        return;
    }

    let count = usize::try_from(num_unknowns).unwrap_or(0);
    if count == 0 {
        return;
    }

    let incoming = unsafe { core::slice::from_raw_parts(unknowns, count) };
    state::update_info(info_ptr, |info_state| {
        for chunk in incoming {
            let mut name = chunk.name;
            name[4] = 0;
            let data = if chunk.data.is_null() || chunk.size == 0 {
                Vec::new()
            } else {
                unsafe { core::slice::from_raw_parts(chunk.data, chunk.size) }.to_vec()
            };
            info_state.unknown_chunks.push(OwnedUnknownChunk {
                name,
                data,
                location: normalize_unknown_chunk_location(png_ptr, c_int::from(chunk.location)),
            });
        }
        rebuild_unknown_chunk_cache(info_state);
        info_state.core.free_me |= PNG_FREE_UNKN;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_reset_zstream(png_ptr: png_structrp) -> c_int {
    crate::abi_guard!(png_ptr, {
        if png_ptr.is_null() {
            return Z_STREAM_ERROR;
        }

        state::update_png(png_ptr, |png_state| {
            png_state.progressive_state = Default::default();
            png_state.core.idat_size = 0;
            png_state.core.zowner = 0;
            png_state.core.save_buffer_size = 0;
            png_state.core.current_buffer_size = 0;
            png_state.core.flags &= !PNG_FLAG_ZSTREAM_ENDED;
        });

        Z_OK
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_convert_to_rfc1123(
    png_ptr: png_structrp,
    ptime: png_const_timep,
) -> png_const_charp {
    crate::abi_guard!(png_ptr, {
        if png_ptr.is_null() || ptime.is_null() {
            return ptr::null();
        }

        let converted = state::with_png_mut(png_ptr, |png_state| {
            if unsafe {
                crate::common::png_convert_to_rfc1123_buffer(
                    png_state.time_buffer.as_mut_ptr(),
                    ptime,
                )
            } != 0
            {
                png_state.time_buffer.as_ptr()
            } else {
                ptr::null()
            }
        })
        .unwrap_or(ptr::null());

        if converted.is_null() {
            unsafe {
                crate::error::png_warning(
                    png_ptr,
                    b"Ignoring invalid time value\0".as_ptr().cast(),
                );
            }
        }

        converted
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_expand_gray_1_2_4_to_8(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, {
        apply_transform_mask(png_ptr, PNG_EXPAND);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_rgb_to_gray_status(png_ptr: png_const_structrp) -> png_byte {
    crate::abi_guard!(png_ptr.cast_mut(), {
        read_core(png_ptr).rgb_to_gray_status
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_strip_alpha(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, {
        apply_transform_mask(png_ptr, PNG_STRIP_ALPHA);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_gamma(
    png_ptr: png_structrp,
    screen_gamma: f64,
    override_file_gamma: f64,
) {
    crate::abi_guard!(png_ptr, unsafe {
        png_set_gamma_fixed(
            png_ptr,
            gamma_fixed_from_double(png_ptr, screen_gamma),
            gamma_fixed_from_double(png_ptr, override_file_gamma),
        );
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_gamma_fixed(
    png_ptr: png_structrp,
    screen_gamma: png_fixed_point,
    override_file_gamma: png_fixed_point,
) {
    crate::abi_guard!(png_ptr, {
        if !read_transform_ok(png_ptr, false) {
            return;
        }

        if screen_gamma <= 0 || override_file_gamma <= 0 {
            unsafe {
                crate::error::png_error(png_ptr, b"invalid gamma value\0".as_ptr().cast());
            }
        }

        let mut core = read_core(png_ptr);
        core.colorspace.gamma = override_file_gamma;
        core.colorspace.flags |= PNG_COLORSPACE_HAVE_GAMMA;
        core.screen_gamma = screen_gamma;
        write_core(png_ptr, &core);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_info(png_ptr: png_structrp, info_ptr: png_inforp) {
    crate::abi_guard!(png_ptr, unsafe {
        longjmp_on_zero(
            png_ptr,
            crate::read::png_safe_rust_read_info(png_ptr, info_ptr),
        );
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_start_read_image(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, unsafe {
        longjmp_on_zero(
            png_ptr,
            crate::read::png_safe_rust_start_read_image(png_ptr),
        );
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_update_info(png_ptr: png_structrp, info_ptr: png_inforp) {
    crate::abi_guard!(png_ptr, unsafe {
        longjmp_on_zero(
            png_ptr,
            crate::read::png_safe_rust_read_update_info(png_ptr, info_ptr),
        );
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
        longjmp_on_zero(
            png_ptr,
            crate::read::png_safe_rust_read_rows(png_ptr, row, display_row, num_rows),
        );
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_row(
    png_ptr: png_structrp,
    row: png_bytep,
    display_row: png_bytep,
) {
    crate::abi_guard!(png_ptr, unsafe {
        longjmp_on_zero(
            png_ptr,
            crate::read::png_safe_rust_read_row(png_ptr, row, display_row),
        );
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_image(png_ptr: png_structrp, image: png_bytepp) {
    crate::abi_guard!(png_ptr, unsafe {
        longjmp_on_zero(
            png_ptr,
            crate::read::png_safe_rust_read_image(png_ptr, image),
        );
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_end(png_ptr: png_structrp, info_ptr: png_inforp) {
    crate::abi_guard!(png_ptr, unsafe {
        longjmp_on_zero(
            png_ptr,
            crate::read::png_safe_rust_read_end(png_ptr, info_ptr),
        );
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_crc_action(
    png_ptr: png_structrp,
    crit_action: c_int,
    ancil_action: c_int,
) {
    crate::abi_guard!(png_ptr, {
        if png_ptr.is_null() {
            return;
        }

        let mut core = read_core(png_ptr);
        match crit_action {
            PNG_CRC_NO_CHANGE => {}
            PNG_CRC_WARN_USE => {
                core.flags &= !PNG_FLAG_CRC_CRITICAL_MASK;
                core.flags |= PNG_FLAG_CRC_CRITICAL_USE;
            }
            PNG_CRC_QUIET_USE => {
                core.flags &= !PNG_FLAG_CRC_CRITICAL_MASK;
                core.flags |= PNG_FLAG_CRC_CRITICAL_USE | PNG_FLAG_CRC_CRITICAL_IGNORE;
            }
            PNG_CRC_WARN_DISCARD => {
                let _ =
                    unsafe { call_warning(png_ptr, b"Can't discard critical data on CRC error\0") };
                core.flags &= !PNG_FLAG_CRC_CRITICAL_MASK;
            }
            PNG_CRC_ERROR_QUIT | PNG_CRC_DEFAULT => {
                core.flags &= !PNG_FLAG_CRC_CRITICAL_MASK;
            }
            _ => {
                core.flags &= !PNG_FLAG_CRC_CRITICAL_MASK;
            }
        }

        match ancil_action {
            PNG_CRC_NO_CHANGE => {}
            PNG_CRC_WARN_USE => {
                core.flags &= !PNG_FLAG_CRC_ANCILLARY_MASK;
                core.flags |= PNG_FLAG_CRC_ANCILLARY_USE;
            }
            PNG_CRC_QUIET_USE => {
                core.flags &= !PNG_FLAG_CRC_ANCILLARY_MASK;
                core.flags |= PNG_FLAG_CRC_ANCILLARY_USE | PNG_FLAG_CRC_ANCILLARY_NOWARN;
            }
            PNG_CRC_ERROR_QUIT => {
                core.flags &= !PNG_FLAG_CRC_ANCILLARY_MASK;
                core.flags |= PNG_FLAG_CRC_ANCILLARY_NOWARN;
            }
            PNG_CRC_WARN_DISCARD | PNG_CRC_DEFAULT => {
                core.flags &= !PNG_FLAG_CRC_ANCILLARY_MASK;
            }
            _ => {
                core.flags &= !PNG_FLAG_CRC_ANCILLARY_MASK;
            }
        }

        write_core(png_ptr, &core);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_current_row_number(png_ptr: png_const_structrp) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), { read_core(png_ptr).row_number })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_current_pass_number(png_ptr: png_const_structrp) -> png_byte {
    crate::abi_guard!(png_ptr.cast_mut(), { read_core(png_ptr).pass as png_byte })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_process_data(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    buffer: png_bytep,
    buffer_size: usize,
) {
    crate::abi_guard!(png_ptr, unsafe {
        longjmp_on_zero(
            png_ptr,
            crate::read_progressive::png_safe_rust_process_data(
                png_ptr,
                info_ptr,
                buffer,
                buffer_size,
            ),
        );
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_progressive_combine_row(
    png_ptr: png_const_structrp,
    old_row: png_bytep,
    new_row: png_const_bytep,
) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        if png_ptr.is_null() || old_row.is_null() || new_row.is_null() {
            return;
        }

        let rowbytes = read_core(png_ptr).rowbytes;
        if rowbytes != 0 {
            let core = read_core(png_ptr);
            let width = usize::try_from(core.width).unwrap_or(0);
            let src = core::slice::from_raw_parts(new_row, rowbytes);
            crate::bridge_ffi::copy_packed_row_preserving_padding(
                old_row,
                src,
                rowbytes,
                width,
                usize::from(core.pixel_depth),
            );
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_pixels_per_meter(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        read_phys(png_ptr, info_ptr)
            .filter(|(x, y, unit)| *unit == PNG_RESOLUTION_METER && x == y)
            .map(|(x, _, _)| x)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_x_pixels_per_meter(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        read_phys(png_ptr, info_ptr)
            .filter(|(_, _, unit)| *unit == PNG_RESOLUTION_METER)
            .map(|(x, _, _)| x)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_y_pixels_per_meter(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        read_phys(png_ptr, info_ptr)
            .filter(|(_, _, unit)| *unit == PNG_RESOLUTION_METER)
            .map(|(_, y, _)| y)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_pixel_aspect_ratio(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> f32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        read_phys(png_ptr, info_ptr)
            .filter(|(x, y, _)| *x != 0 && *y != 0)
            .map(|(x, y, _)| y as f32 / x as f32)
            .unwrap_or(0.0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_pixel_aspect_ratio_fixed(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_fixed_point {
    crate::abi_guard!(png_ptr.cast_mut(), {
        read_phys(png_ptr, info_ptr)
            .filter(|(x, y, _)| {
                *x != 0 && *y != 0 && *x <= PNG_UINT_31_MAX && *y <= PNG_UINT_31_MAX
            })
            .and_then(|(x, y, _)| {
                let numerator = i128::from(y) * i128::from(PNG_FP_1);
                let rounded = (numerator + i128::from(x) / 2) / i128::from(x);
                checked_fixed_point(rounded)
            })
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_x_offset_pixels(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_int_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        read_offsets(png_ptr, info_ptr)
            .filter(|(_, _, unit)| *unit == PNG_OFFSET_PIXEL)
            .map(|(x, _, _)| x)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_y_offset_pixels(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_int_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        read_offsets(png_ptr, info_ptr)
            .filter(|(_, _, unit)| *unit == PNG_OFFSET_PIXEL)
            .map(|(_, y, _)| y)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_x_offset_microns(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_int_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        read_offsets(png_ptr, info_ptr)
            .filter(|(_, _, unit)| *unit == PNG_OFFSET_MICROMETER)
            .map(|(x, _, _)| x)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_y_offset_microns(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_int_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        read_offsets(png_ptr, info_ptr)
            .filter(|(_, _, unit)| *unit == PNG_OFFSET_MICROMETER)
            .map(|(_, y, _)| y)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_signature(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_const_bytep {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() || info_ptr.is_null() {
            ptr::null()
        } else {
            PNG_SIGNATURE.as_ptr()
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_handle_as_unknown(
    png_ptr: png_const_structrp,
    chunk_name: png_const_bytep,
) -> c_int {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() || chunk_name.is_null() {
            return crate::read_util::PNG_HANDLE_CHUNK_AS_DEFAULT;
        }

        let mut name = [0u8; 4];
        unsafe { ptr::copy_nonoverlapping(chunk_name, name.as_mut_ptr(), name.len()) };
        state::with_png(png_ptr.cast_mut(), |png_state| {
            png_state
                .unknown_chunk_list
                .iter()
                .find(|entry| entry.name == name)
                .map(|entry| c_int::from(entry.keep))
                .unwrap_or(crate::read_util::PNG_HANDLE_CHUNK_AS_DEFAULT)
        })
        .unwrap_or(crate::read_util::PNG_HANDLE_CHUNK_AS_DEFAULT)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_unknown_chunks(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    unknowns: png_const_unknown_chunkp,
    num_unknowns: c_int,
) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        store_unknown_chunks_impl(png_ptr, info_ptr, unknowns, num_unknowns);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_unknown_chunk_location(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    chunk: c_int,
    location: c_int,
) {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if info_ptr.is_null() || chunk < 0 {
            return;
        }

        state::update_info(info_ptr, |info_state| {
            let Some(entry) = info_state.unknown_chunks.get_mut(chunk as usize) else {
                return;
            };
            entry.location = normalize_unknown_chunk_location(png_ptr, location);
            rebuild_unknown_chunk_cache(info_state);
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_unknown_chunks(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    entries: png_unknown_chunkpp,
) -> c_int {
    crate::abi_guard!(png_ptr.cast_mut(), {
        state::with_info_mut(info_ptr, |info_state| {
            rebuild_unknown_chunk_cache(info_state);
            if !entries.is_null() {
                unsafe {
                    *entries = if info_state.unknown_chunk_cache.is_empty() {
                        ptr::null_mut()
                    } else {
                        info_state.unknown_chunk_cache.as_mut_ptr()
                    };
                }
            }
            info_state.unknown_chunks.len() as c_int
        })
        .unwrap_or_else(|| {
            if !entries.is_null() {
                unsafe { *entries = ptr::null_mut() };
            }
            0
        })
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_invalid(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    mask: c_int,
) {
    crate::abi_guard!(png_ptr.cast_mut(), {
        state::update_info(info_ptr, |info_state| {
            info_state.core.valid &= !(mask as png_uint_32);
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_png(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    transforms: c_int,
    _params: png_voidp,
) {
    crate::abi_guard!(png_ptr, unsafe {
        if png_ptr.is_null() || info_ptr.is_null() {
            return;
        }

        png_read_info(png_ptr, info_ptr);

        let height = crate::get::png_get_image_height(png_ptr, info_ptr);
        let max_height = usize::MAX / size_of::<png_bytep>();
        if usize::try_from(height).unwrap_or(usize::MAX) > max_height {
            crate::error::png_error(
                png_ptr,
                b"Image is too high to process with png_read_png()\0"
                    .as_ptr()
                    .cast(),
            );
        }

        apply_read_png_transforms(png_ptr, info_ptr, transforms);
        let _ = crate::interlace::png_set_interlace_handling(png_ptr);
        png_read_update_info(png_ptr, info_ptr);
        crate::memory::png_free_data(png_ptr, info_ptr, PNG_FREE_ROWS, 0);
        ensure_read_png_rows(png_ptr, info_ptr);
        png_read_image(png_ptr, crate::get::png_get_rows(png_ptr, info_ptr));
        state::update_info(info_ptr, |info_state| {
            info_state.core.valid |= PNG_INFO_IDAT;
        });
        png_read_end(png_ptr, info_ptr);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_permit_mng_features(
    png_ptr: png_structrp,
    mng_features: png_uint_32,
) -> png_uint_32 {
    crate::abi_guard!(png_ptr, {
        if png_ptr.is_null() {
            return 0;
        }

        let permitted = mng_features & PNG_ALL_MNG_FEATURES;
        state::update_png(png_ptr, |png_state| {
            png_state.mng_features_permitted = permitted;
        });
        permitted
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_pixels_per_inch(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        ppi_from_ppm(png_get_pixels_per_meter(png_ptr, info_ptr))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_x_pixels_per_inch(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        ppi_from_ppm(png_get_x_pixels_per_meter(png_ptr, info_ptr))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_y_pixels_per_inch(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        ppi_from_ppm(png_get_y_pixels_per_meter(png_ptr, info_ptr))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_x_offset_inches_fixed(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_fixed_point {
    crate::abi_guard!(png_ptr.cast_mut(), {
        fixed_inches_from_microns(png_get_x_offset_microns(png_ptr, info_ptr))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_y_offset_inches_fixed(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_fixed_point {
    crate::abi_guard!(png_ptr.cast_mut(), {
        fixed_inches_from_microns(png_get_y_offset_microns(png_ptr, info_ptr))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_x_offset_inches(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> f32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        png_get_x_offset_microns(png_ptr, info_ptr) as f32 * 0.000_039_37
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_y_offset_inches(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> f32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        png_get_y_offset_microns(png_ptr, info_ptr) as f32 * 0.000_039_37
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_pHYs_dpi(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
    res_x: *mut png_uint_32,
    res_y: *mut png_uint_32,
    unit_type: *mut c_int,
) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        let valid = unsafe { crate::get::png_get_pHYs(png_ptr, info_ptr, res_x, res_y, unit_type) };
        if (valid & PNG_INFO_pHYs) != 0 && !unit_type.is_null() {
            unsafe {
                if *unit_type == PNG_RESOLUTION_METER {
                    if !res_x.is_null() {
                        *res_x = ppi_from_ppm(*res_x);
                    }
                    if !res_y.is_null() {
                        *res_y = ppi_from_ppm(*res_y);
                    }
                }
            }
        }
        valid
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_uint_32(buf: png_const_bytep) -> png_uint_32 {
    crate::abi_guard_no_png!({
        if buf.is_null() {
            return 0;
        }

        (u32::from(unsafe { *buf }) << 24)
            | (u32::from(unsafe { *buf.add(1) }) << 16)
            | (u32::from(unsafe { *buf.add(2) }) << 8)
            | u32::from(unsafe { *buf.add(3) })
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_uint_16(buf: png_const_bytep) -> png_uint_16 {
    crate::abi_guard_no_png!({
        if buf.is_null() {
            return 0;
        }

        ((u16::from(unsafe { *buf }) << 8) | u16::from(unsafe { *buf.add(1) })) as png_uint_16
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_int_32(buf: png_const_bytep) -> png_int_32 {
    crate::abi_guard_no_png!({
        let value = unsafe { png_get_uint_32(buf) };
        if (value & 0x8000_0000) == 0 {
            value as png_int_32
        } else {
            let magnitude = (value ^ 0xffff_ffff).wrapping_add(1);
            if (magnitude & 0x8000_0000) == 0 {
                -(magnitude as png_int_32)
            } else {
                0
            }
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_uint_31(
    png_ptr: png_const_structrp,
    buf: png_const_bytep,
) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        let value = unsafe { png_get_uint_32(buf) };
        if value > PNG_UINT_31_MAX {
            unsafe {
                crate::error::png_error(
                    png_ptr,
                    b"PNG unsigned integer out of range\0".as_ptr().cast(),
                );
            }
        }
        value
    })
}
