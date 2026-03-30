use crate::chunks::with_colorspace_state;
use crate::types::*;

#[derive(Clone, Copy)]
struct XyzGuard {
    sums: [i64; 3],
    degenerate: bool,
}

fn checked_xyz_sum(x: png_fixed_point, y: png_fixed_point, z: png_fixed_point) -> Option<i64> {
    let x = i64::from(x);
    let y = i64::from(y);
    let z = i64::from(z);
    x.checked_add(y)?.checked_add(z)
}

fn guard_xyz_endpoints(values: [png_fixed_point; 9]) -> XyzGuard {
    let sums = [
        checked_xyz_sum(values[0], values[1], values[2]),
        checked_xyz_sum(values[3], values[4], values[5]),
        checked_xyz_sum(values[6], values[7], values[8]),
    ];

    let degenerate = values.iter().any(|value| *value < 0)
        || sums.iter().any(|sum| sum.is_none_or(|value| value <= 0));

    XyzGuard {
        sums: [
            sums[0].unwrap_or(0),
            sums[1].unwrap_or(0),
            sums[2].unwrap_or(0),
        ],
        degenerate,
    }
}

fn guard_rgb_to_gray_coefficients(
    red: png_fixed_point,
    green: png_fixed_point,
) -> Option<(png_uint_16, png_uint_16)> {
    if red < 0 || green < 0 {
        return None;
    }

    let total = i64::from(red).checked_add(i64::from(green))?;
    if total > 100_000 {
        return None;
    }

    let red_coeff = ((u64::try_from(red).ok()? * 32_768) / 100_000) as png_uint_16;
    let green_coeff = ((u64::try_from(green).ok()? * 32_768) / 100_000) as png_uint_16;
    Some((red_coeff, green_coeff))
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

unsafe extern "C" {
    fn upstream_png_set_rgb_to_gray(
        png_ptr: png_structrp,
        error_action: core::ffi::c_int,
        red: f64,
        green: f64,
    );
    fn upstream_png_set_rgb_to_gray_fixed(
        png_ptr: png_structrp,
        error_action: core::ffi::c_int,
        red: png_fixed_point,
        green: png_fixed_point,
    );
    fn upstream_png_set_background(
        png_ptr: png_structrp,
        background_color: png_const_color_16p,
        background_gamma_code: core::ffi::c_int,
        need_expand: core::ffi::c_int,
        background_gamma: f64,
    );
    fn upstream_png_set_background_fixed(
        png_ptr: png_structrp,
        background_color: png_const_color_16p,
        background_gamma_code: core::ffi::c_int,
        need_expand: core::ffi::c_int,
        background_gamma: png_fixed_point,
    );
    fn upstream_png_set_alpha_mode(
        png_ptr: png_structrp,
        mode: core::ffi::c_int,
        output_gamma: f64,
    );
    fn upstream_png_set_alpha_mode_fixed(
        png_ptr: png_structrp,
        mode: core::ffi::c_int,
        output_gamma: png_fixed_point,
    );
    fn upstream_png_set_cHRM_XYZ(
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
    );
    fn upstream_png_set_cHRM_XYZ_fixed(
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
    );
    fn upstream_png_get_cHRM_XYZ(
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
    ) -> png_uint_32;
    fn upstream_png_get_cHRM_XYZ_fixed(
        png_ptr: png_const_structrp,
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
    ) -> png_uint_32;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_rgb_to_gray(
    png_ptr: png_structrp,
    error_action: core::ffi::c_int,
    red: f64,
    green: f64,
) {
    let coefficients = float_to_fixed(red)
        .zip(float_to_fixed(green))
        .and_then(|(red, green)| guard_rgb_to_gray_coefficients(red, green));
    with_colorspace_state(png_ptr, |state| state.rgb_to_gray_coefficients = coefficients);
    unsafe {
        upstream_png_set_rgb_to_gray(png_ptr, error_action, red, green);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_rgb_to_gray_fixed(
    png_ptr: png_structrp,
    error_action: core::ffi::c_int,
    red: png_fixed_point,
    green: png_fixed_point,
) {
    let coefficients = guard_rgb_to_gray_coefficients(red, green);
    with_colorspace_state(png_ptr, |state| state.rgb_to_gray_coefficients = coefficients);
    unsafe {
        upstream_png_set_rgb_to_gray_fixed(png_ptr, error_action, red, green);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_background(
    png_ptr: png_structrp,
    background_color: png_const_color_16p,
    background_gamma_code: core::ffi::c_int,
    need_expand: core::ffi::c_int,
    background_gamma: f64,
) {
    with_colorspace_state(png_ptr, |state| state.background_requested = true);
    unsafe {
        upstream_png_set_background(
            png_ptr,
            background_color,
            background_gamma_code,
            need_expand,
            background_gamma,
        );
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_background_fixed(
    png_ptr: png_structrp,
    background_color: png_const_color_16p,
    background_gamma_code: core::ffi::c_int,
    need_expand: core::ffi::c_int,
    background_gamma: png_fixed_point,
) {
    with_colorspace_state(png_ptr, |state| state.background_requested = true);
    unsafe {
        upstream_png_set_background_fixed(
            png_ptr,
            background_color,
            background_gamma_code,
            need_expand,
            background_gamma,
        );
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_alpha_mode(
    png_ptr: png_structrp,
    mode: core::ffi::c_int,
    output_gamma: f64,
) {
    with_colorspace_state(png_ptr, |state| state.alpha_mode_requested = true);
    unsafe {
        upstream_png_set_alpha_mode(png_ptr, mode, output_gamma);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_alpha_mode_fixed(
    png_ptr: png_structrp,
    mode: core::ffi::c_int,
    output_gamma: png_fixed_point,
) {
    with_colorspace_state(png_ptr, |state| state.alpha_mode_requested = true);
    unsafe {
        upstream_png_set_alpha_mode_fixed(png_ptr, mode, output_gamma);
    }
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
    let guard = float_to_fixed(red_X)
        .zip(float_to_fixed(red_Y))
        .zip(float_to_fixed(red_Z))
        .zip(float_to_fixed(green_X))
        .zip(float_to_fixed(green_Y))
        .zip(float_to_fixed(green_Z))
        .zip(float_to_fixed(blue_X))
        .zip(float_to_fixed(blue_Y))
        .zip(float_to_fixed(blue_Z))
        .map(|values| {
            let ((((((((red_X, red_Y), red_Z), green_X), green_Y), green_Z), blue_X), blue_Y), blue_Z) =
                values;
            guard_xyz_endpoints([
                red_X, red_Y, red_Z, green_X, green_Y, green_Z, blue_X, blue_Y, blue_Z,
            ])
        })
        .unwrap_or(XyzGuard {
            sums: [0; 3],
            degenerate: true,
        });
    with_colorspace_state(png_ptr.cast_mut(), |state| {
        state.last_xyz_sums = guard.sums;
        state.degenerate_xyz = guard.degenerate;
    });
    unsafe {
        upstream_png_set_cHRM_XYZ(
            png_ptr, info_ptr, red_X, red_Y, red_Z, green_X, green_Y, green_Z, blue_X, blue_Y,
            blue_Z,
        );
    }
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
    let guard = guard_xyz_endpoints([
        red_X, red_Y, red_Z, green_X, green_Y, green_Z, blue_X, blue_Y, blue_Z,
    ]);
    with_colorspace_state(png_ptr.cast_mut(), |state| {
        state.last_xyz_sums = guard.sums;
        state.degenerate_xyz = guard.degenerate;
    });
    unsafe {
        upstream_png_set_cHRM_XYZ_fixed(
            png_ptr, info_ptr, red_X, red_Y, red_Z, green_X, green_Y, green_Z, blue_X, blue_Y,
            blue_Z,
        );
    }
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
    let result = unsafe {
        upstream_png_get_cHRM_XYZ(
            png_ptr, info_ptr, red_X, red_Y, red_Z, green_X, green_Y, green_Z, blue_X, blue_Y,
            blue_Z,
        )
    };
    if result != 0 {
        let guard = red_X
            .as_ref()
            .zip(red_Y.as_ref())
            .zip(red_Z.as_ref())
            .zip(green_X.as_ref())
            .zip(green_Y.as_ref())
            .zip(green_Z.as_ref())
            .zip(blue_X.as_ref())
            .zip(blue_Y.as_ref())
            .zip(blue_Z.as_ref())
            .and_then(|values| {
                let ((((((((red_X, red_Y), red_Z), green_X), green_Y), green_Z), blue_X), blue_Y), blue_Z) =
                    values;
                Some(guard_xyz_endpoints([
                    float_to_fixed(*red_X)?,
                    float_to_fixed(*red_Y)?,
                    float_to_fixed(*red_Z)?,
                    float_to_fixed(*green_X)?,
                    float_to_fixed(*green_Y)?,
                    float_to_fixed(*green_Z)?,
                    float_to_fixed(*blue_X)?,
                    float_to_fixed(*blue_Y)?,
                    float_to_fixed(*blue_Z)?,
                ]))
            });
        if let Some(guard) = guard {
            with_colorspace_state(png_ptr.cast_mut(), |state| {
                state.last_xyz_sums = guard.sums;
                state.degenerate_xyz = guard.degenerate;
            });
        }
    }
    result
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_cHRM_XYZ_fixed(
    png_ptr: png_const_structrp,
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
    let result = unsafe {
        upstream_png_get_cHRM_XYZ_fixed(
            png_ptr, info_ptr, red_X, red_Y, red_Z, green_X, green_Y, green_Z, blue_X, blue_Y,
            blue_Z,
        )
    };
    if result != 0 {
        let guard = red_X
            .as_ref()
            .zip(red_Y.as_ref())
            .zip(red_Z.as_ref())
            .zip(green_X.as_ref())
            .zip(green_Y.as_ref())
            .zip(green_Z.as_ref())
            .zip(blue_X.as_ref())
            .zip(blue_Y.as_ref())
            .zip(blue_Z.as_ref())
            .map(|values| {
                let ((((((((red_X, red_Y), red_Z), green_X), green_Y), green_Z), blue_X), blue_Y), blue_Z) =
                    values;
                guard_xyz_endpoints([
                    *red_X, *red_Y, *red_Z, *green_X, *green_Y, *green_Z, *blue_X, *blue_Y,
                    *blue_Z,
                ])
            });
        if let Some(guard) = guard {
            with_colorspace_state(png_ptr.cast_mut(), |state| {
                state.last_xyz_sums = guard.sums;
                state.degenerate_xyz = guard.degenerate;
            });
        }
    }
    result
}
