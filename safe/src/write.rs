use crate::state;
use crate::types::*;

unsafe extern "C" {
    fn runtime_png_write_info_before_PLTE(png_ptr: png_structrp, info_ptr: png_const_inforp);
    fn runtime_png_write_info(png_ptr: png_structrp, info_ptr: png_const_inforp);
    fn runtime_png_write_row(png_ptr: png_structrp, row: png_const_bytep);
    fn runtime_png_write_rows(png_ptr: png_structrp, row: png_bytepp, num_rows: png_uint_32);
    fn runtime_png_write_image(png_ptr: png_structrp, image: png_bytepp);
    fn runtime_png_write_end(png_ptr: png_structrp, info_ptr: png_inforp);
    fn runtime_png_write_png(
        png_ptr: png_structrp,
        info_ptr: png_inforp,
        transforms: png_uint_32,
        params: png_voidp,
    );
    fn runtime_png_set_flush(png_ptr: png_structrp, nrows: core::ffi::c_int);
    fn runtime_png_write_flush(png_ptr: png_structrp);
}

fn touch_write_registrations(png_ptr: png_structrp) {
    let _ = crate::io::write_callback_registration(png_ptr);
    let _ = crate::io::write_user_transform_registration(png_ptr);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_write_info_before_PLTE(
    png_ptr: png_structrp,
    info_ptr: png_const_inforp,
) {
    crate::abi_guard!(png_ptr, unsafe {
        touch_write_registrations(png_ptr);
        runtime_png_write_info_before_PLTE(png_ptr, info_ptr);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_write_info(png_ptr: png_structrp, info_ptr: png_const_inforp) {
    crate::abi_guard!(png_ptr, unsafe {
        touch_write_registrations(png_ptr);
        runtime_png_write_info(png_ptr, info_ptr);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_write_row(png_ptr: png_structrp, row: png_const_bytep) {
    crate::abi_guard!(png_ptr, unsafe {
        touch_write_registrations(png_ptr);
        runtime_png_write_row(png_ptr, row);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_write_rows(
    png_ptr: png_structrp,
    row: png_bytepp,
    num_rows: png_uint_32,
) {
    crate::abi_guard!(png_ptr, unsafe {
        touch_write_registrations(png_ptr);
        runtime_png_write_rows(png_ptr, row, num_rows);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_write_image(png_ptr: png_structrp, image: png_bytepp) {
    crate::abi_guard!(png_ptr, unsafe {
        touch_write_registrations(png_ptr);
        runtime_png_write_image(png_ptr, image);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_write_end(png_ptr: png_structrp, info_ptr: png_inforp) {
    crate::abi_guard!(png_ptr, unsafe {
        touch_write_registrations(png_ptr);
        runtime_png_write_end(png_ptr, info_ptr);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_write_png(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    transforms: png_uint_32,
    params: png_voidp,
) {
    crate::abi_guard!(png_ptr, unsafe {
        touch_write_registrations(png_ptr);
        runtime_png_write_png(png_ptr, info_ptr, transforms, params);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_flush(png_ptr: png_structrp, nrows: core::ffi::c_int) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_flush(png_ptr, nrows);
        state::update_png(png_ptr, |state| {
            state.flush_rows = nrows;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_write_flush(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, unsafe {
        touch_write_registrations(png_ptr);
        runtime_png_write_flush(png_ptr);
    });
}
