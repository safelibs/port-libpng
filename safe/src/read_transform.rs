use crate::chunks::{refresh_output_info, with_transform_state};
use crate::interlace::{mask_info_rows, mask_packed_row_padding};
use crate::types::*;

unsafe extern "C" {
    fn upstream_png_read_info(png_ptr: png_structrp, info_ptr: png_inforp);
    fn upstream_png_read_update_info(png_ptr: png_structrp, info_ptr: png_inforp);
    fn upstream_png_read_row(png_ptr: png_structrp, row: png_bytep, display_row: png_bytep);
    fn upstream_png_read_png(
        png_ptr: png_structrp,
        info_ptr: png_inforp,
        transforms: core::ffi::c_int,
        params: png_voidp,
    );

    fn upstream_png_set_expand(png_ptr: png_structrp);
    fn upstream_png_set_expand_16(png_ptr: png_structrp);
    fn upstream_png_set_palette_to_rgb(png_ptr: png_structrp);
    fn upstream_png_set_tRNS_to_alpha(png_ptr: png_structrp);
    fn upstream_png_set_gray_to_rgb(png_ptr: png_structrp);
    fn upstream_png_set_scale_16(png_ptr: png_structrp);
    fn upstream_png_set_strip_16(png_ptr: png_structrp);
    fn upstream_png_set_quantize(
        png_ptr: png_structrp,
        palette: png_colorp,
        num_palette: core::ffi::c_int,
        maximum_colors: core::ffi::c_int,
        histogram: png_const_uint_16p,
        full_quantize: core::ffi::c_int,
    );
    fn upstream_png_set_shift(png_ptr: png_structrp, true_bits: png_const_color_8p);
    fn upstream_png_set_swap_alpha(png_ptr: png_structrp);
    fn upstream_png_set_invert_alpha(png_ptr: png_structrp);
    fn upstream_png_set_invert_mono(png_ptr: png_structrp);
    fn upstream_png_set_bgr(png_ptr: png_structrp);
}

macro_rules! forward_transform_setter {
    ($name:ident, $upstream:ident, $field:ident) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(png_ptr: png_structrp) {
            with_transform_state(png_ptr, |state| state.$field = true);
            unsafe {
                $upstream(png_ptr);
            }
        }
    };
}

forward_transform_setter!(png_set_expand, upstream_png_set_expand, expand);
forward_transform_setter!(png_set_expand_16, upstream_png_set_expand_16, expand_16);
forward_transform_setter!(
    png_set_palette_to_rgb,
    upstream_png_set_palette_to_rgb,
    palette_to_rgb
);
forward_transform_setter!(
    png_set_tRNS_to_alpha,
    upstream_png_set_tRNS_to_alpha,
    trns_to_alpha
);
forward_transform_setter!(png_set_gray_to_rgb, upstream_png_set_gray_to_rgb, gray_to_rgb);
forward_transform_setter!(png_set_scale_16, upstream_png_set_scale_16, scale_16);
forward_transform_setter!(png_set_strip_16, upstream_png_set_strip_16, strip_16);
forward_transform_setter!(png_set_swap_alpha, upstream_png_set_swap_alpha, swap_alpha);
forward_transform_setter!(
    png_set_invert_alpha,
    upstream_png_set_invert_alpha,
    invert_alpha
);
forward_transform_setter!(png_set_invert_mono, upstream_png_set_invert_mono, invert_mono);
forward_transform_setter!(png_set_bgr, upstream_png_set_bgr, bgr);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_info(png_ptr: png_structrp, info_ptr: png_inforp) {
    unsafe {
        upstream_png_read_info(png_ptr, info_ptr);
    }
    let _ = unsafe { refresh_output_info(png_ptr, info_ptr) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_update_info(png_ptr: png_structrp, info_ptr: png_inforp) {
    unsafe {
        upstream_png_read_update_info(png_ptr, info_ptr);
    }
    let _ = unsafe { refresh_output_info(png_ptr, info_ptr) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_row(
    png_ptr: png_structrp,
    row: png_bytep,
    display_row: png_bytep,
) {
    unsafe {
        upstream_png_read_row(png_ptr, row, display_row);
    }
    unsafe {
        mask_packed_row_padding(png_ptr, row);
        mask_packed_row_padding(png_ptr, display_row);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_png(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    transforms: core::ffi::c_int,
    params: png_voidp,
) {
    unsafe {
        upstream_png_read_png(png_ptr, info_ptr, transforms, params);
        mask_info_rows(png_ptr, info_ptr);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_quantize(
    png_ptr: png_structrp,
    palette: png_colorp,
    num_palette: core::ffi::c_int,
    maximum_colors: core::ffi::c_int,
    histogram: png_const_uint_16p,
    full_quantize: core::ffi::c_int,
) {
    with_transform_state(png_ptr, |state| state.quantize = true);
    unsafe {
        upstream_png_set_quantize(
            png_ptr,
            palette,
            num_palette,
            maximum_colors,
            histogram,
            full_quantize,
        );
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_shift(
    png_ptr: png_structrp,
    true_bits: png_const_color_8p,
) {
    with_transform_state(png_ptr, |state| state.shift = true);
    unsafe {
        upstream_png_set_shift(png_ptr, true_bits);
    }
}
