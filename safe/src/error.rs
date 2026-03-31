use crate::common::INTERNAL_PANIC_MESSAGE;
use crate::types::*;
use core::ffi::c_int;

unsafe extern "C" {
    fn upstream_png_warning(png_ptr: png_const_structrp, warning_message: png_const_charp);
    fn upstream_png_error(png_ptr: png_const_structrp, error_message: png_const_charp) -> !;
    fn upstream_png_benign_error(png_ptr: png_const_structrp, error_message: png_const_charp);
    fn upstream_png_chunk_warning(png_ptr: png_const_structrp, warning_message: png_const_charp);
    fn upstream_png_chunk_error(png_ptr: png_const_structrp, error_message: png_const_charp) -> !;
    fn upstream_png_chunk_benign_error(
        png_ptr: png_const_structrp,
        error_message: png_const_charp,
    );
    fn upstream_png_set_error_fn(
        png_ptr: png_structrp,
        error_ptr: png_voidp,
        error_fn: png_error_ptr,
        warning_fn: png_error_ptr,
    );
    fn upstream_png_get_error_ptr(png_ptr: png_const_structrp) -> png_voidp;
    fn upstream_png_set_longjmp_fn(
        png_ptr: png_structrp,
        longjmp_fn: png_longjmp_ptr,
        jmp_buf_size: usize,
    ) -> *mut JmpBuf;
    fn upstream_png_longjmp(png_ptr: png_const_structrp, val: c_int) -> !;
}

pub(crate) unsafe fn panic_to_png_error(png_ptr: png_structrp) -> ! {
    unsafe { upstream_png_error(png_ptr, INTERNAL_PANIC_MESSAGE.as_ptr().cast()) }
}

pub(crate) unsafe fn png_app_warning(png_ptr: png_const_structrp, error_message: png_const_charp) {
    unsafe { upstream_png_warning(png_ptr, error_message) }
}

pub(crate) unsafe fn png_app_error(png_ptr: png_const_structrp, error_message: png_const_charp) {
    unsafe { upstream_png_error(png_ptr, error_message) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_warning(
    png_ptr: png_const_structrp,
    warning_message: png_const_charp,
) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_warning(png_ptr, warning_message)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_error(
    png_ptr: png_const_structrp,
    error_message: png_const_charp,
) -> ! {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_error(png_ptr, error_message)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_benign_error(
    png_ptr: png_const_structrp,
    error_message: png_const_charp,
) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_benign_error(png_ptr, error_message)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_chunk_warning(
    png_ptr: png_const_structrp,
    warning_message: png_const_charp,
) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_chunk_warning(png_ptr, warning_message)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_chunk_error(
    png_ptr: png_const_structrp,
    error_message: png_const_charp,
) -> ! {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_chunk_error(png_ptr, error_message)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_chunk_benign_error(
    png_ptr: png_const_structrp,
    error_message: png_const_charp,
) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_chunk_benign_error(png_ptr, error_message)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_error_fn(
    png_ptr: png_structrp,
    error_ptr: png_voidp,
    error_fn: png_error_ptr,
    warning_fn: png_error_ptr,
) {
    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_set_error_fn(png_ptr, error_ptr, error_fn, warning_fn)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_error_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe { upstream_png_get_error_ptr(png_ptr) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_longjmp_fn(
    png_ptr: png_structrp,
    longjmp_fn: png_longjmp_ptr,
    jmp_buf_size: usize,
) -> *mut JmpBuf {
    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_set_longjmp_fn(png_ptr, longjmp_fn, jmp_buf_size)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_longjmp(png_ptr: png_const_structrp, val: c_int) -> ! {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe { upstream_png_longjmp(png_ptr, val) })
}
