use crate::chunks::{
    call_app_error, call_benign_error, call_error, call_warning, read_core, read_info_core,
    write_core, write_info_core,
};
use crate::types::*;
use core::ffi::c_int;

const PNG_HAVE_IHDR: png_uint_32 = 0x01;
const PNG_FLAG_ROW_INIT: png_uint_32 = 0x0040;
const PNG_FLAG_ASSUME_SRGB: png_uint_32 = 0x1000;
const PNG_FLAG_OPTIMIZE_ALPHA: png_uint_32 = 0x2000;
const PNG_FLAG_DETECT_UNINITIALIZED: png_uint_32 = 0x4000;

const PNG_COMPOSE: png_uint_32 = 0x0080;
const PNG_BACKGROUND_EXPAND: png_uint_32 = 0x0100;
const PNG_EXPAND: png_uint_32 = 0x1000;
const PNG_STRIP_ALPHA: png_uint_32 = 0x40000;
const PNG_RGB_TO_GRAY_ERR: png_uint_32 = 0x200000;
const PNG_RGB_TO_GRAY_WARN: png_uint_32 = 0x400000;
const PNG_RGB_TO_GRAY: png_uint_32 = 0x600000;
const PNG_ENCODE_ALPHA: png_uint_32 = 0x800000;

const PNG_COLOR_TYPE_PALETTE: png_byte = 3;

const PNG_BACKGROUND_GAMMA_UNKNOWN: c_int = 0;
const PNG_BACKGROUND_GAMMA_FILE: png_byte = 2;

const PNG_COLORSPACE_HAVE_GAMMA: png_uint_16 = 0x0001;
const PNG_COLORSPACE_HAVE_ENDPOINTS: png_uint_16 = 0x0002;
const PNG_COLORSPACE_FROM_CHRM: png_uint_16 = 0x0010;
const PNG_COLORSPACE_INVALID: png_uint_16 = 0x8000;
const PNG_INFO_CHRM: png_uint_32 = 0x0004;

const PNG_FP_1: png_fixed_point = 100_000;
const PNG_DEFAULT_SRGB: png_fixed_point = -1;
const PNG_GAMMA_MAC_18: png_fixed_point = -2;
const PNG_GAMMA_SRGB: png_fixed_point = 220_000;
const PNG_GAMMA_MAC_OLD: png_fixed_point = 151_724;
const PNG_GAMMA_MAC_INVERSE: png_fixed_point = 65_909;
const PNG_GAMMA_SRGB_INVERSE: png_fixed_point = 45_455;

const PNG_ERROR_ACTION_NONE: c_int = 1;
const PNG_ERROR_ACTION_WARN: c_int = 2;
const PNG_ERROR_ACTION_ERROR: c_int = 3;

const PNG_ALPHA_PNG: c_int = 0;
const PNG_ALPHA_ASSOCIATED: c_int = 1;
const PNG_ALPHA_OPTIMIZED: c_int = 2;
const PNG_ALPHA_BROKEN: c_int = 3;

fn rtran_ok(png_ptr: png_structrp, need_ihdr: bool) -> bool {
    if png_ptr.is_null() {
        return false;
    }

    let mut core = read_core(png_ptr);
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
            let _ = call_app_error(png_ptr, b"invalid before the PNG header has been read\0");
        }
        return false;
    }

    core.flags |= PNG_FLAG_DETECT_UNINITIALIZED;
    write_core(png_ptr, &core);
    true
}

fn reciprocal(value: png_fixed_point) -> png_fixed_point {
    (((i64::from(PNG_FP_1) * i64::from(PNG_FP_1)) + i64::from(value / 2)) / i64::from(value))
        as png_fixed_point
}

fn translate_gamma_flags(
    core: &mut png_safe_read_core,
    output_gamma: png_fixed_point,
    is_screen: bool,
) -> png_fixed_point {
    if output_gamma == PNG_DEFAULT_SRGB || output_gamma == (PNG_FP_1 / PNG_DEFAULT_SRGB) {
        core.flags |= PNG_FLAG_ASSUME_SRGB;
        if is_screen {
            PNG_GAMMA_SRGB
        } else {
            PNG_GAMMA_SRGB_INVERSE
        }
    } else if output_gamma == PNG_GAMMA_MAC_18 || output_gamma == (PNG_FP_1 / PNG_GAMMA_MAC_18) {
        if is_screen {
            PNG_GAMMA_MAC_OLD
        } else {
            PNG_GAMMA_MAC_INVERSE
        }
    } else {
        output_gamma
    }
}

fn float_to_fixed(value: f64) -> Option<png_fixed_point> {
    if !value.is_finite() {
        return None;
    }
    let scaled = (value * 100_000.0).round();
    if scaled < f64::from(i32::MIN) || scaled > f64::from(i32::MAX) {
        None
    } else {
        Some(scaled as png_fixed_point)
    }
}

fn fixed_to_float(value: png_fixed_point) -> f64 {
    f64::from(value) / 100_000.0
}

fn set_endpoint_values(
    info: &mut png_safe_info_core,
    values: [png_fixed_point; 9],
) -> Result<(), ()> {
    let fits_fixed = |value: i64| -> bool { (0..=i64::from(i32::MAX)).contains(&value) };
    let sums = [
        i64::from(values[0])
            .checked_add(i64::from(values[1]))
            .and_then(|sum| sum.checked_add(i64::from(values[2]))),
        i64::from(values[3])
            .checked_add(i64::from(values[4]))
            .and_then(|sum| sum.checked_add(i64::from(values[5]))),
        i64::from(values[6])
            .checked_add(i64::from(values[7]))
            .and_then(|sum| sum.checked_add(i64::from(values[8]))),
    ];
    if values.iter().any(|value| *value < 0)
        || sums
            .iter()
            .any(|sum| sum.is_none_or(|v| v <= 0 || !fits_fixed(v)))
    {
        return Err(());
    }

    let white_x = i64::from(values[0]) + i64::from(values[3]) + i64::from(values[6]);
    let white_y = i64::from(values[1]) + i64::from(values[4]) + i64::from(values[7]);
    let white_z = i64::from(values[2]) + i64::from(values[5]) + i64::from(values[8]);
    if !fits_fixed(white_x) || !fits_fixed(white_y) || !fits_fixed(white_z) {
        return Err(());
    }
    let white_sum = white_x
        .checked_add(white_y)
        .and_then(|sum| sum.checked_add(white_z))
        .filter(|sum| *sum > 0 && fits_fixed(*sum))
        .ok_or(())?;

    let xy_from_xyz =
        |x: i64, y: i64, sum: i64| -> Result<(png_fixed_point, png_fixed_point), ()> {
            let x = ((x * i64::from(PNG_FP_1)) / sum)
                .try_into()
                .map_err(|_| ())?;
            let y = ((y * i64::from(PNG_FP_1)) / sum)
                .try_into()
                .map_err(|_| ())?;
            Ok((x, y))
        };

    let (redx, redy) = xy_from_xyz(i64::from(values[0]), i64::from(values[1]), sums[0].unwrap())?;
    let (greenx, greeny) =
        xy_from_xyz(i64::from(values[3]), i64::from(values[4]), sums[1].unwrap())?;
    let (bluex, bluey) = xy_from_xyz(i64::from(values[6]), i64::from(values[7]), sums[2].unwrap())?;
    let (whitex, whitey) = xy_from_xyz(white_x, white_y, white_sum)?;

    info.colorspace.end_points_XYZ = png_XYZ {
        red_X: values[0],
        red_Y: values[1],
        red_Z: values[2],
        green_X: values[3],
        green_Y: values[4],
        green_Z: values[5],
        blue_X: values[6],
        blue_Y: values[7],
        blue_Z: values[8],
    };
    info.colorspace.end_points_xy = png_xy {
        redx,
        redy,
        greenx,
        greeny,
        bluex,
        bluey,
        whitex,
        whitey,
    };
    info.colorspace.flags &= !PNG_COLORSPACE_INVALID;
    info.colorspace.flags |= PNG_COLORSPACE_HAVE_ENDPOINTS | PNG_COLORSPACE_FROM_CHRM;
    info.valid |= PNG_INFO_CHRM;
    Ok(())
}

fn set_rgb_to_gray_fixed_impl(
    png_ptr: png_structrp,
    error_action: c_int,
    red: png_fixed_point,
    green: png_fixed_point,
) {
    if !rtran_ok(png_ptr, true) {
        return;
    }

    let mut core = read_core(png_ptr);
    core.transformations &= !PNG_RGB_TO_GRAY;
    match error_action {
        PNG_ERROR_ACTION_NONE => core.transformations |= PNG_RGB_TO_GRAY,
        PNG_ERROR_ACTION_WARN => core.transformations |= PNG_RGB_TO_GRAY_WARN,
        PNG_ERROR_ACTION_ERROR => core.transformations |= PNG_RGB_TO_GRAY_ERR,
        _ => {
            let _ = unsafe { call_error(png_ptr, b"invalid error action to rgb_to_gray\0") };
            return;
        }
    }

    if core.color_type == PNG_COLOR_TYPE_PALETTE {
        core.transformations |= PNG_EXPAND;
    }

    if red >= 0 && green >= 0 && i64::from(red) + i64::from(green) <= i64::from(PNG_FP_1) {
        core.rgb_to_gray_red_coeff =
            (((u64::try_from(red).unwrap_or(0)) * 32768) / 100000) as png_uint_16;
        core.rgb_to_gray_green_coeff =
            (((u64::try_from(green).unwrap_or(0)) * 32768) / 100000) as png_uint_16;
        core.rgb_to_gray_coefficients_set = 1;
    } else {
        if red >= 0 && green >= 0 {
            let _ = unsafe {
                call_warning(
                    png_ptr,
                    b"ignoring out of range rgb_to_gray coefficients\0",
                )
            };
        }
        if core.rgb_to_gray_red_coeff == 0 && core.rgb_to_gray_green_coeff == 0 {
            core.rgb_to_gray_red_coeff = 6968;
            core.rgb_to_gray_green_coeff = 23434;
        }
    }

    write_core(png_ptr, &core);
}

fn set_background_fixed_impl(
    png_ptr: png_structrp,
    background_color: png_const_color_16p,
    background_gamma_code: c_int,
    need_expand: c_int,
    background_gamma: png_fixed_point,
) {
    if background_color.is_null() || !rtran_ok(png_ptr, false) {
        return;
    }

    if background_gamma_code == PNG_BACKGROUND_GAMMA_UNKNOWN {
        let _ = unsafe {
            call_warning(
                png_ptr,
                b"Application must supply a known background gamma\0",
            )
        };
        return;
    }

    let mut core = read_core(png_ptr);
    core.transformations |= PNG_COMPOSE | PNG_STRIP_ALPHA;
    core.transformations &= !PNG_ENCODE_ALPHA;
    core.flags &= !PNG_FLAG_OPTIMIZE_ALPHA;
    core.background = unsafe { *background_color };
    core.background_gamma = background_gamma;
    core.background_gamma_type = background_gamma_code as png_byte;
    if need_expand != 0 {
        core.transformations |= PNG_BACKGROUND_EXPAND;
    } else {
        core.transformations &= !PNG_BACKGROUND_EXPAND;
    }
    write_core(png_ptr, &core);
}

fn set_alpha_mode_fixed_impl(png_ptr: png_structrp, mode: c_int, output_gamma: png_fixed_point) {
    if !rtran_ok(png_ptr, false) {
        return;
    }

    let mut core = read_core(png_ptr);
    let mut output_gamma = translate_gamma_flags(&mut core, output_gamma, true);
    if !(1000..=10000000).contains(&output_gamma) {
        let _ = unsafe { call_error(png_ptr, b"output gamma out of expected range\0") };
        return;
    }

    let file_gamma = reciprocal(output_gamma);
    let mut compose = false;

    match mode {
        PNG_ALPHA_PNG => {
            core.transformations &= !PNG_ENCODE_ALPHA;
            core.flags &= !PNG_FLAG_OPTIMIZE_ALPHA;
        }
        PNG_ALPHA_ASSOCIATED => {
            compose = true;
            core.transformations &= !PNG_ENCODE_ALPHA;
            core.flags &= !PNG_FLAG_OPTIMIZE_ALPHA;
            output_gamma = PNG_FP_1;
        }
        PNG_ALPHA_OPTIMIZED => {
            compose = true;
            core.transformations &= !PNG_ENCODE_ALPHA;
            core.flags |= PNG_FLAG_OPTIMIZE_ALPHA;
        }
        PNG_ALPHA_BROKEN => {
            compose = true;
            core.transformations |= PNG_ENCODE_ALPHA;
            core.flags &= !PNG_FLAG_OPTIMIZE_ALPHA;
        }
        _ => {
            let _ = unsafe { call_error(png_ptr, b"invalid alpha mode\0") };
            return;
        }
    }

    if (core.colorspace.flags & PNG_COLORSPACE_HAVE_GAMMA) == 0 {
        core.colorspace.gamma = file_gamma;
        core.colorspace.flags |= PNG_COLORSPACE_HAVE_GAMMA;
    }
    core.screen_gamma = output_gamma;

    if compose {
        if (core.transformations & PNG_COMPOSE) != 0 {
            let _ = unsafe {
                call_error(
                    png_ptr,
                    b"conflicting calls to set alpha mode and background\0",
                )
            };
            return;
        }
        core.background = png_color_16::default();
        core.background_gamma = core.colorspace.gamma;
        core.background_gamma_type = PNG_BACKGROUND_GAMMA_FILE;
        core.transformations &= !PNG_BACKGROUND_EXPAND;
        core.transformations |= PNG_COMPOSE;
    }
    write_core(png_ptr, &core);
}

fn set_cHRM_xyz_fixed_impl(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    values: [png_fixed_point; 9],
) {
    if png_ptr.is_null() || info_ptr.is_null() {
        return;
    }

    let mut info = read_info_core(info_ptr);
    if set_endpoint_values(&mut info, values).is_err() {
        info.colorspace.flags |= PNG_COLORSPACE_INVALID;
        info.colorspace.flags &= !PNG_COLORSPACE_HAVE_ENDPOINTS;
        info.valid &= !PNG_INFO_CHRM;
        write_info_core(info_ptr, &info);
        let _ = unsafe { call_benign_error(png_ptr.cast_mut(), b"invalid end points\0") };
        return;
    }

    write_info_core(info_ptr, &info);
}

fn get_cHRM_xyz_fixed_impl(
    info_ptr: png_const_inforp,
    red_X: *mut png_fixed_point,
    red_Y: *mut png_fixed_point,
    red_Z: *mut png_fixed_point,
    green_X: *mut png_fixed_point,
    green_Y: *mut png_fixed_point,
    green_Z: *mut png_fixed_point,
    blue_X: *mut png_fixed_point,
    blue_Y: *mut png_fixed_point,
    blue_Z: *mut png_fixed_point,
) -> png_uint_32 {
    if info_ptr.is_null() {
        return 0;
    }

    let info = read_info_core(info_ptr);
    if (info.colorspace.flags & PNG_COLORSPACE_HAVE_ENDPOINTS) == 0 {
        return 0;
    }

    unsafe {
        if !red_X.is_null() {
            *red_X = info.colorspace.end_points_XYZ.red_X;
        }
        if !red_Y.is_null() {
            *red_Y = info.colorspace.end_points_XYZ.red_Y;
        }
        if !red_Z.is_null() {
            *red_Z = info.colorspace.end_points_XYZ.red_Z;
        }
        if !green_X.is_null() {
            *green_X = info.colorspace.end_points_XYZ.green_X;
        }
        if !green_Y.is_null() {
            *green_Y = info.colorspace.end_points_XYZ.green_Y;
        }
        if !green_Z.is_null() {
            *green_Z = info.colorspace.end_points_XYZ.green_Z;
        }
        if !blue_X.is_null() {
            *blue_X = info.colorspace.end_points_XYZ.blue_X;
        }
        if !blue_Y.is_null() {
            *blue_Y = info.colorspace.end_points_XYZ.blue_Y;
        }
        if !blue_Z.is_null() {
            *blue_Z = info.colorspace.end_points_XYZ.blue_Z;
        }
    }

    PNG_INFO_CHRM
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_rgb_to_gray_fixed(
    png_ptr: png_structrp,
    error_action: c_int,
    red: png_fixed_point,
    green: png_fixed_point,
) {
    crate::abi_guard!(png_ptr, {
        set_rgb_to_gray_fixed_impl(png_ptr, error_action, red, green);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_rgb_to_gray(
    png_ptr: png_structrp,
    error_action: c_int,
    red: f64,
    green: f64,
) {
    crate::abi_guard!(png_ptr, {
        let red = float_to_fixed(red).unwrap_or(-1);
        let green = float_to_fixed(green).unwrap_or(-1);
        set_rgb_to_gray_fixed_impl(png_ptr, error_action, red, green);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_background_fixed(
    png_ptr: png_structrp,
    background_color: png_const_color_16p,
    background_gamma_code: c_int,
    need_expand: c_int,
    background_gamma: png_fixed_point,
) {
    crate::abi_guard!(png_ptr, {
        set_background_fixed_impl(
            png_ptr,
            background_color,
            background_gamma_code,
            need_expand,
            background_gamma,
        );
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_background(
    png_ptr: png_structrp,
    background_color: png_const_color_16p,
    background_gamma_code: c_int,
    need_expand: c_int,
    background_gamma: f64,
) {
    crate::abi_guard!(png_ptr, {
        let gamma = float_to_fixed(background_gamma).unwrap_or(0);
        set_background_fixed_impl(
            png_ptr,
            background_color,
            background_gamma_code,
            need_expand,
            gamma,
        );
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_alpha_mode_fixed(
    png_ptr: png_structrp,
    mode: c_int,
    output_gamma: png_fixed_point,
) {
    crate::abi_guard!(png_ptr, {
        set_alpha_mode_fixed_impl(png_ptr, mode, output_gamma);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_alpha_mode(png_ptr: png_structrp, mode: c_int, output_gamma: f64) {
    crate::abi_guard!(png_ptr, {
        if let Some(output_gamma) = float_to_fixed(output_gamma) {
            set_alpha_mode_fixed_impl(png_ptr, mode, output_gamma);
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_cHRM_XYZ_fixed(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    red_X: png_fixed_point,
    red_Y: png_fixed_point,
    red_Z: png_fixed_point,
    green_X: png_fixed_point,
    green_Y: png_fixed_point,
    green_Z: png_fixed_point,
    blue_X: png_fixed_point,
    blue_Y: png_fixed_point,
    blue_Z: png_fixed_point,
) {
    crate::abi_guard!(png_ptr.cast_mut(), {
        set_cHRM_xyz_fixed_impl(
            png_ptr,
            info_ptr,
            [
                red_X, red_Y, red_Z, green_X, green_Y, green_Z, blue_X, blue_Y, blue_Z,
            ],
        );
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_cHRM_XYZ(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    red_X: f64,
    red_Y: f64,
    red_Z: f64,
    green_X: f64,
    green_Y: f64,
    green_Z: f64,
    blue_X: f64,
    blue_Y: f64,
    blue_Z: f64,
) {
    crate::abi_guard!(png_ptr.cast_mut(), {
        let values = [
            float_to_fixed(red_X),
            float_to_fixed(red_Y),
            float_to_fixed(red_Z),
            float_to_fixed(green_X),
            float_to_fixed(green_Y),
            float_to_fixed(green_Z),
            float_to_fixed(blue_X),
            float_to_fixed(blue_Y),
            float_to_fixed(blue_Z),
        ];
        if let [
            Some(red_X),
            Some(red_Y),
            Some(red_Z),
            Some(green_X),
            Some(green_Y),
            Some(green_Z),
            Some(blue_X),
            Some(blue_Y),
            Some(blue_Z),
        ] = values
        {
            set_cHRM_xyz_fixed_impl(
                png_ptr,
                info_ptr,
                [
                    red_X, red_Y, red_Z, green_X, green_Y, green_Z, blue_X, blue_Y, blue_Z,
                ],
            );
        } else {
            let _ = call_benign_error(png_ptr.cast_mut(), b"invalid end points\0");
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_cHRM_XYZ_fixed(
    _png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
    red_X: *mut png_fixed_point,
    red_Y: *mut png_fixed_point,
    red_Z: *mut png_fixed_point,
    green_X: *mut png_fixed_point,
    green_Y: *mut png_fixed_point,
    green_Z: *mut png_fixed_point,
    blue_X: *mut png_fixed_point,
    blue_Y: *mut png_fixed_point,
    blue_Z: *mut png_fixed_point,
) -> png_uint_32 {
    crate::abi_guard_no_png!({
        get_cHRM_xyz_fixed_impl(
            info_ptr, red_X, red_Y, red_Z, green_X, green_Y, green_Z, blue_X, blue_Y, blue_Z,
        )
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_cHRM_XYZ(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
    red_X: *mut f64,
    red_Y: *mut f64,
    red_Z: *mut f64,
    green_X: *mut f64,
    green_Y: *mut f64,
    green_Z: *mut f64,
    blue_X: *mut f64,
    blue_Y: *mut f64,
    blue_Z: *mut f64,
) -> png_uint_32 {
    crate::abi_guard_no_png!({
        let mut fixed_red_X = 0;
        let mut fixed_red_Y = 0;
        let mut fixed_red_Z = 0;
        let mut fixed_green_X = 0;
        let mut fixed_green_Y = 0;
        let mut fixed_green_Z = 0;
        let mut fixed_blue_X = 0;
        let mut fixed_blue_Y = 0;
        let mut fixed_blue_Z = 0;
        let _ = png_ptr;
        let result = get_cHRM_xyz_fixed_impl(
            info_ptr,
            &mut fixed_red_X,
            &mut fixed_red_Y,
            &mut fixed_red_Z,
            &mut fixed_green_X,
            &mut fixed_green_Y,
            &mut fixed_green_Z,
            &mut fixed_blue_X,
            &mut fixed_blue_Y,
            &mut fixed_blue_Z,
        );
        if result == 0 {
            return 0;
        }

        unsafe {
            if !red_X.is_null() {
                *red_X = fixed_to_float(fixed_red_X);
            }
            if !red_Y.is_null() {
                *red_Y = fixed_to_float(fixed_red_Y);
            }
            if !red_Z.is_null() {
                *red_Z = fixed_to_float(fixed_red_Z);
            }
            if !green_X.is_null() {
                *green_X = fixed_to_float(fixed_green_X);
            }
            if !green_Y.is_null() {
                *green_Y = fixed_to_float(fixed_green_Y);
            }
            if !green_Z.is_null() {
                *green_Z = fixed_to_float(fixed_green_Z);
            }
            if !blue_X.is_null() {
                *blue_X = fixed_to_float(fixed_blue_X);
            }
            if !blue_Y.is_null() {
                *blue_Y = fixed_to_float(fixed_blue_Y);
            }
            if !blue_Z.is_null() {
                *blue_Z = fixed_to_float(fixed_blue_Z);
            }
        }

        PNG_INFO_CHRM
    })
}
