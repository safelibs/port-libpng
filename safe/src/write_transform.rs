use crate::types::*;

unsafe extern "C" {
    fn runtime_png_set_filter(
        png_ptr: png_structrp,
        method: core::ffi::c_int,
        filters: core::ffi::c_int,
    );
    fn runtime_png_set_filter_heuristics(
        png_ptr: png_structrp,
        heuristic_method: core::ffi::c_int,
        num_weights: core::ffi::c_int,
        filter_weights: png_const_doublep,
        filter_costs: png_const_doublep,
    );
    fn runtime_png_set_filter_heuristics_fixed(
        png_ptr: png_structrp,
        heuristic_method: core::ffi::c_int,
        num_weights: core::ffi::c_int,
        filter_weights: png_const_fixed_point_p,
        filter_costs: png_const_fixed_point_p,
    );
    fn runtime_png_set_add_alpha(
        png_ptr: png_structrp,
        filler: png_uint_32,
        flags: core::ffi::c_int,
    );
    fn runtime_png_set_filler(png_ptr: png_structrp, filler: png_uint_32, flags: core::ffi::c_int);
    fn runtime_png_set_packing(png_ptr: png_structrp);
    fn runtime_png_set_packswap(png_ptr: png_structrp);
    fn runtime_png_set_swap(png_ptr: png_structrp);
}

fn touch_write_user_transform_state(png_ptr: png_structrp) {
    let _ = crate::io::write_user_transform_registration(png_ptr);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_filter(
    png_ptr: png_structrp,
    method: core::ffi::c_int,
    filters: core::ffi::c_int,
) {
    crate::abi_guard!(png_ptr, unsafe {
        touch_write_user_transform_state(png_ptr);
        runtime_png_set_filter(png_ptr, method, filters);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_filter_heuristics(
    png_ptr: png_structrp,
    heuristic_method: core::ffi::c_int,
    num_weights: core::ffi::c_int,
    filter_weights: png_const_doublep,
    filter_costs: png_const_doublep,
) {
    crate::abi_guard!(png_ptr, unsafe {
        touch_write_user_transform_state(png_ptr);
        runtime_png_set_filter_heuristics(
            png_ptr,
            heuristic_method,
            num_weights,
            filter_weights,
            filter_costs,
        );
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_filter_heuristics_fixed(
    png_ptr: png_structrp,
    heuristic_method: core::ffi::c_int,
    num_weights: core::ffi::c_int,
    filter_weights: png_const_fixed_point_p,
    filter_costs: png_const_fixed_point_p,
) {
    crate::abi_guard!(png_ptr, unsafe {
        touch_write_user_transform_state(png_ptr);
        runtime_png_set_filter_heuristics_fixed(
            png_ptr,
            heuristic_method,
            num_weights,
            filter_weights,
            filter_costs,
        );
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_add_alpha(
    png_ptr: png_structrp,
    filler: png_uint_32,
    flags: core::ffi::c_int,
) {
    crate::abi_guard!(png_ptr, unsafe {
        touch_write_user_transform_state(png_ptr);
        runtime_png_set_add_alpha(png_ptr, filler, flags);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_filler(
    png_ptr: png_structrp,
    filler: png_uint_32,
    flags: core::ffi::c_int,
) {
    crate::abi_guard!(png_ptr, unsafe {
        touch_write_user_transform_state(png_ptr);
        runtime_png_set_filler(png_ptr, filler, flags);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_packing(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, unsafe {
        touch_write_user_transform_state(png_ptr);
        runtime_png_set_packing(png_ptr);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_packswap(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, unsafe {
        touch_write_user_transform_state(png_ptr);
        runtime_png_set_packswap(png_ptr);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_swap(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, unsafe {
        touch_write_user_transform_state(png_ptr);
        runtime_png_set_swap(png_ptr);
    });
}
