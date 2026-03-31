use crate::types::*;
use core::ffi::{c_char, c_int};
use core::mem::MaybeUninit;

unsafe extern "C" {
    fn png_safe_read_core_get(png_ptr: png_const_structrp, out: *mut png_safe_read_core);
    fn png_safe_read_core_set(png_ptr: png_structrp, input: *const png_safe_read_core);
    fn png_safe_info_core_get(info_ptr: png_const_inforp, out: *mut png_safe_info_core);
    fn png_safe_info_core_set(info_ptr: png_inforp, input: *const png_safe_info_core);

    fn png_safe_call_warning(png_ptr: png_structrp, message: png_const_charp) -> c_int;
    fn png_safe_call_benign_error(png_ptr: png_structrp, message: png_const_charp) -> c_int;
    fn png_safe_call_app_error(png_ptr: png_structrp, message: png_const_charp) -> c_int;
    fn png_safe_call_error(png_ptr: png_structrp, message: png_const_charp) -> c_int;
}

pub(crate) fn read_core(png_ptr: png_const_structrp) -> png_safe_read_core {
    let mut out = MaybeUninit::<png_safe_read_core>::zeroed();
    unsafe {
        png_safe_read_core_get(png_ptr, out.as_mut_ptr());
        out.assume_init()
    }
}

pub(crate) fn write_core(png_ptr: png_structrp, core: &png_safe_read_core) {
    unsafe {
        png_safe_read_core_set(png_ptr, core);
    }
}

pub(crate) fn read_info_core(info_ptr: png_const_inforp) -> png_safe_info_core {
    let mut out = MaybeUninit::<png_safe_info_core>::zeroed();
    unsafe {
        png_safe_info_core_get(info_ptr, out.as_mut_ptr());
        out.assume_init()
    }
}

pub(crate) fn write_info_core(info_ptr: png_inforp, core: &png_safe_info_core) {
    unsafe {
        png_safe_info_core_set(info_ptr, core);
    }
}

pub(crate) unsafe fn call_warning(png_ptr: png_structrp, message: &[u8]) -> c_int {
    unsafe { png_safe_call_warning(png_ptr, message.as_ptr().cast::<c_char>()) }
}

pub(crate) unsafe fn call_benign_error(png_ptr: png_structrp, message: &[u8]) -> c_int {
    unsafe { png_safe_call_benign_error(png_ptr, message.as_ptr().cast::<c_char>()) }
}

pub(crate) unsafe fn call_app_error(png_ptr: png_structrp, message: &[u8]) -> c_int {
    unsafe { png_safe_call_app_error(png_ptr, message.as_ptr().cast::<c_char>()) }
}

pub(crate) unsafe fn call_error(png_ptr: png_structrp, message: &[u8]) -> c_int {
    unsafe { png_safe_call_error(png_ptr, message.as_ptr().cast::<c_char>()) }
}
