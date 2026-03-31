use crate::state;
use crate::types::*;
use core::ffi::c_int;
use core::ptr;

const INTERNAL_PANIC_MESSAGE: &[u8] = b"libpng safe internal panic\0";

unsafe extern "C" {
    fn upstream_png_warning(png_ptr: png_const_structrp, warning_message: png_const_charp);
    fn upstream_png_error(png_ptr: png_const_structrp, error_message: png_const_charp) -> !;
    fn upstream_png_benign_error(png_ptr: png_const_structrp, error_message: png_const_charp);
    fn upstream_png_chunk_warning(png_ptr: png_const_structrp, warning_message: png_const_charp);
    fn upstream_png_chunk_error(png_ptr: png_const_structrp, error_message: png_const_charp) -> !;
    fn upstream_png_chunk_benign_error(png_ptr: png_const_structrp, error_message: png_const_charp);
    fn upstream_png_set_error_fn(
        png_ptr: png_structrp,
        error_ptr: png_voidp,
        error_fn: png_error_ptr,
        warning_fn: png_error_ptr,
    );
    fn upstream_png_get_error_ptr(png_ptr: png_const_structrp) -> png_voidp;
    fn upstream_png_malloc_warn(png_ptr: png_const_structrp, size: png_alloc_size_t) -> png_voidp;
    fn png_safe_longjmp_state_size() -> usize;
    fn png_safe_longjmp_local_buffer(png_ptr: png_structrp) -> *mut JmpBuf;
    fn png_safe_longjmp_get_buffer(png_ptr: png_const_structrp) -> *mut JmpBuf;
    fn png_safe_longjmp_get_size(png_ptr: png_const_structrp) -> usize;
    fn png_safe_longjmp_set_fields(
        png_ptr: png_structrp,
        longjmp_fn: png_longjmp_ptr,
        jmp_buf_ptr: *mut JmpBuf,
        jmp_buf_size: usize,
    );
    fn png_safe_longjmp_call(png_ptr: png_const_structrp, val: c_int);
}

pub(crate) unsafe fn panic_to_png_error(png_ptr: png_structrp) -> ! {
    unsafe { upstream_png_error(png_ptr, INTERNAL_PANIC_MESSAGE.as_ptr().cast()) }
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
        upstream_png_set_error_fn(png_ptr, error_ptr, error_fn, warning_fn);
        state::update_png(png_ptr, |state| {
            state.error_ptr = error_ptr;
            state.error_fn = error_fn;
            state.warning_fn = warning_fn;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_error_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        state::get_png(png_ptr.cast_mut())
            .map(|state| state.error_ptr)
            .unwrap_or_else(|| unsafe { upstream_png_get_error_ptr(png_ptr) })
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_longjmp_fn(
    png_ptr: png_structrp,
    longjmp_fn: png_longjmp_ptr,
    jmp_buf_size: usize,
) -> *mut JmpBuf {
    crate::abi_guard!(png_ptr, unsafe {
        if png_ptr.is_null() {
            return ptr::null_mut();
        }

        let local_size = png_safe_longjmp_state_size();
        let local_buffer = png_safe_longjmp_local_buffer(png_ptr);
        let mut current_buffer = png_safe_longjmp_get_buffer(png_ptr);
        let current_size = png_safe_longjmp_get_size(png_ptr);

        if current_buffer.is_null() {
            current_buffer = if jmp_buf_size <= local_size {
                local_buffer
            } else {
                upstream_png_malloc_warn(png_ptr, jmp_buf_size).cast()
            };

            if current_buffer.is_null() {
                return ptr::null_mut();
            }

            let stored_size = if current_buffer == local_buffer {
                0
            } else {
                jmp_buf_size
            };
            png_safe_longjmp_set_fields(png_ptr, longjmp_fn, current_buffer, stored_size);
        } else {
            let effective_size = if current_size == 0 {
                if current_buffer != local_buffer {
                    upstream_png_error(png_ptr, c"Libpng jmp_buf still allocated".as_ptr());
                }
                local_size
            } else {
                current_size
            };

            if effective_size != jmp_buf_size {
                upstream_png_warning(png_ptr, c"Application jmp_buf size changed".as_ptr());
                return ptr::null_mut();
            }

            png_safe_longjmp_set_fields(png_ptr, longjmp_fn, current_buffer, current_size);
        }

        state::update_png(png_ptr, |state| {
            state.longjmp_fn = longjmp_fn;
            state.jmp_buf_ptr = current_buffer;
            state.jmp_buf_size = png_safe_longjmp_get_size(png_ptr);
        });

        current_buffer
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_longjmp(png_ptr: png_const_structrp, val: c_int) -> ! {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        png_safe_longjmp_call(png_ptr, val);
        std::process::abort()
    })
}
