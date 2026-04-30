use crate::state;
use crate::types::*;
use core::ffi::c_int;
use core::ptr;

const INTERNAL_PANIC_MESSAGE: &[u8] = b"libpng safe internal panic\0";

fn callback_or_default(
    png_ptr: png_const_structrp,
) -> (
    png_voidp,
    png_error_ptr,
    png_error_ptr,
    png_longjmp_ptr,
    *mut JmpBuf,
    usize,
) {
    state::with_png(png_ptr.cast_mut(), |png_state| {
        (
            png_state.error_ptr,
            png_state.error_fn,
            png_state.warning_fn,
            png_state.longjmp_fn,
            png_state.jmp_buf_ptr,
            png_state.jmp_buf_size,
        )
    })
    .unwrap_or((ptr::null_mut(), None, None, None, ptr::null_mut(), 0))
}

unsafe fn invoke_warning_callback(png_ptr: png_const_structrp, message: png_const_charp) {
    let (_, _, warning_fn, _, _, _) = callback_or_default(png_ptr);
    if let Some(callback) = warning_fn {
        unsafe { callback(png_ptr.cast_mut(), message) };
    }
}

unsafe fn invoke_error_callback(png_ptr: png_const_structrp, message: png_const_charp) {
    let (_, error_fn, _, _, _, _) = callback_or_default(png_ptr);
    if let Some(callback) = error_fn {
        unsafe { callback(png_ptr.cast_mut(), message) };
    }
}

pub(crate) unsafe fn panic_to_png_error(png_ptr: png_structrp) -> ! {
    unsafe { png_error(png_ptr, INTERNAL_PANIC_MESSAGE.as_ptr().cast()) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_warning(
    png_ptr: png_const_structrp,
    warning_message: png_const_charp,
) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        invoke_warning_callback(png_ptr, warning_message);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_error(
    png_ptr: png_const_structrp,
    error_message: png_const_charp,
) -> ! {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        invoke_error_callback(png_ptr, error_message);
        png_longjmp(png_ptr, 1);
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_benign_error(
    png_ptr: png_const_structrp,
    error_message: png_const_charp,
) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        let benign = state::with_png(png_ptr.cast_mut(), |png_state| png_state.benign_errors != 0)
            .unwrap_or(true);
        if benign {
            invoke_warning_callback(png_ptr, error_message);
        } else {
            png_error(png_ptr, error_message);
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_chunk_warning(
    png_ptr: png_const_structrp,
    warning_message: png_const_charp,
) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        invoke_warning_callback(png_ptr, warning_message);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_chunk_error(
    png_ptr: png_const_structrp,
    error_message: png_const_charp,
) -> ! {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        invoke_error_callback(png_ptr, error_message);
        png_longjmp(png_ptr, 1);
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_chunk_benign_error(
    png_ptr: png_const_structrp,
    error_message: png_const_charp,
) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        png_benign_error(png_ptr, error_message);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_error_fn(
    png_ptr: png_structrp,
    error_ptr: png_voidp,
    error_fn: png_error_ptr,
    warning_fn: png_error_ptr,
) {
    crate::abi_guard!(png_ptr, {
        state::update_png(png_ptr, |png_state| {
            png_state.error_ptr = error_ptr;
            png_state.error_fn = error_fn;
            png_state.warning_fn = warning_fn;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_error_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        state::with_png(png_ptr.cast_mut(), |png_state| png_state.error_ptr)
            .unwrap_or(ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_longjmp_fn(
    png_ptr: png_structrp,
    longjmp_fn: png_longjmp_ptr,
    jmp_buf_size: usize,
) -> *mut JmpBuf {
    crate::abi_guard!(png_ptr, unsafe {
        if png_ptr.is_null() || jmp_buf_size == 0 {
            return ptr::null_mut();
        }

        let existing = state::with_png(png_ptr, |png_state| {
            (png_state.jmp_buf_ptr, png_state.jmp_buf_size)
        })
        .unwrap_or((ptr::null_mut(), 0));

        let buffer = if existing.0.is_null() {
            crate::memory::png_malloc_warn(png_ptr, jmp_buf_size).cast::<JmpBuf>()
        } else if existing.1 == jmp_buf_size {
            existing.0
        } else {
            invoke_warning_callback(png_ptr, c"Application jmp_buf size changed".as_ptr());
            return ptr::null_mut();
        };

        if buffer.is_null() {
            return ptr::null_mut();
        }

        state::update_png(png_ptr, |png_state| {
            png_state.longjmp_fn = longjmp_fn;
            png_state.jmp_buf_ptr = buffer;
            png_state.jmp_buf_size = jmp_buf_size;
        });

        buffer
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_longjmp(png_ptr: png_const_structrp, val: c_int) -> ! {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        let (_, _, _, longjmp_fn, jmp_buf_ptr, _) = callback_or_default(png_ptr);
        if let Some(callback) = longjmp_fn {
            if !jmp_buf_ptr.is_null() {
                callback(jmp_buf_ptr, val);
            }
        }
        std::process::abort()
    })
}

pub(crate) unsafe fn release_longjmp_buffer(png_state: &mut state::PngStructState) {
    if !png_state.jmp_buf_ptr.is_null() {
        unsafe {
            libc::free(png_state.jmp_buf_ptr.cast());
        }
        png_state.jmp_buf_ptr = ptr::null_mut();
        png_state.jmp_buf_size = 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_warning(
    png_ptr: png_const_structrp,
    warning_message: png_const_charp,
) {
    unsafe { png_warning(png_ptr, warning_message) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_error(
    png_ptr: png_const_structrp,
    error_message: png_const_charp,
) -> ! {
    unsafe { png_error(png_ptr, error_message) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_benign_error(
    png_ptr: png_const_structrp,
    error_message: png_const_charp,
) {
    unsafe { png_benign_error(png_ptr, error_message) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_chunk_warning(
    png_ptr: png_const_structrp,
    warning_message: png_const_charp,
) {
    unsafe { png_chunk_warning(png_ptr, warning_message) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_chunk_error(
    png_ptr: png_const_structrp,
    error_message: png_const_charp,
) -> ! {
    unsafe { png_chunk_error(png_ptr, error_message) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_chunk_benign_error(
    png_ptr: png_const_structrp,
    error_message: png_const_charp,
) {
    unsafe { png_chunk_benign_error(png_ptr, error_message) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_error_fn(
    png_ptr: png_structrp,
    error_ptr: png_voidp,
    error_fn: png_error_ptr,
    warning_fn: png_error_ptr,
) {
    unsafe { png_set_error_fn(png_ptr, error_ptr, error_fn, warning_fn) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_error_ptr(png_ptr: png_const_structrp) -> png_voidp {
    unsafe { png_get_error_ptr(png_ptr) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_malloc_warn(
    png_ptr: png_const_structrp,
    size: png_alloc_size_t,
) -> png_voidp {
    unsafe { crate::memory::png_malloc_warn(png_ptr, size) }
}
