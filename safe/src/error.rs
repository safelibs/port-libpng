use crate::common::{
    INTERNAL_PANIC_MESSAGE, PNG_FLAG_APP_ERRORS_WARN, PNG_FLAG_APP_WARNINGS_WARN,
    PNG_FLAG_BENIGN_ERRORS_WARN, build_chunk_message, write_stderr, write_stderr_cstr,
};
use crate::state::{png_ptr_state, png_ptr_state_const};
use crate::types::*;
use core::ffi::c_int;

unsafe fn default_warning(_png_ptr: png_const_structrp, warning_message: png_const_charp) {
    write_stderr(b"libpng warning: ");
    if warning_message.is_null() {
        write_stderr(b"undefined");
    } else {
        write_stderr_cstr(warning_message);
    }
    write_stderr(b"\n");
}

unsafe fn default_error(png_ptr: png_const_structrp, error_message: png_const_charp) -> ! {
    write_stderr(b"libpng error: ");
    if error_message.is_null() {
        write_stderr(b"undefined");
    } else {
        write_stderr_cstr(error_message);
    }
    write_stderr(b"\n");
    png_longjmp(png_ptr, 1);
}

pub(crate) unsafe fn panic_to_png_error(png_ptr: png_structrp) -> ! {
    png_error(png_ptr, INTERNAL_PANIC_MESSAGE.as_ptr().cast());
}

pub(crate) unsafe fn png_app_warning(png_ptr: png_const_structrp, error_message: png_const_charp) {
    if let Some(state) = png_ptr_state_const(png_ptr) {
        if (state.flags & PNG_FLAG_APP_WARNINGS_WARN) != 0 {
            png_warning(png_ptr, error_message);
        } else {
            png_error(png_ptr, error_message);
        }
    }
}

pub(crate) unsafe fn png_app_error(png_ptr: png_const_structrp, error_message: png_const_charp) {
    if let Some(state) = png_ptr_state_const(png_ptr) {
        if (state.flags & PNG_FLAG_APP_ERRORS_WARN) != 0 {
            png_warning(png_ptr, error_message);
        } else {
            png_error(png_ptr, error_message);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_warning(
    png_ptr: png_const_structrp,
    warning_message: png_const_charp,
) {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if let Some(state) = png_ptr_state_const(png_ptr) {
            if let Some(warning_fn) = state.warning_fn {
                warning_fn(png_ptr.cast_mut(), warning_message);
            } else {
                default_warning(png_ptr, warning_message);
            }
        } else {
            default_warning(png_ptr, warning_message);
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_error(
    png_ptr: png_const_structrp,
    error_message: png_const_charp,
) -> ! {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if let Some(state) = png_ptr_state_const(png_ptr) {
            if let Some(error_fn) = state.error_fn {
                error_fn(png_ptr.cast_mut(), error_message);
            }
        }
        default_error(png_ptr, error_message)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_benign_error(
    png_ptr: png_const_structrp,
    error_message: png_const_charp,
) {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if let Some(state) = png_ptr_state_const(png_ptr) {
            if (state.flags & PNG_FLAG_BENIGN_ERRORS_WARN) != 0 {
                if (state.mode & crate::common::PNG_IS_READ_STRUCT) != 0 && state.chunk_name != 0 {
                    png_chunk_warning(png_ptr, error_message);
                } else {
                    png_warning(png_ptr, error_message);
                }
            } else if (state.mode & crate::common::PNG_IS_READ_STRUCT) != 0 && state.chunk_name != 0
            {
                png_chunk_error(png_ptr, error_message);
            } else {
                png_error(png_ptr, error_message);
            }
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_chunk_warning(
    png_ptr: png_const_structrp,
    warning_message: png_const_charp,
) {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if let Some(state) = png_ptr_state_const(png_ptr) {
            let mut buffer = [0i8; 18 + 196];
            build_chunk_message(state.chunk_name, warning_message, &mut buffer);
            png_warning(png_ptr, buffer.as_ptr());
        } else {
            png_warning(png_ptr, warning_message);
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_chunk_error(
    png_ptr: png_const_structrp,
    error_message: png_const_charp,
) -> ! {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if let Some(state) = png_ptr_state_const(png_ptr) {
            let mut buffer = [0i8; 18 + 196];
            build_chunk_message(state.chunk_name, error_message, &mut buffer);
            png_error(png_ptr, buffer.as_ptr())
        } else {
            png_error(png_ptr, error_message)
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_chunk_benign_error(
    png_ptr: png_const_structrp,
    error_message: png_const_charp,
) {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if let Some(state) = png_ptr_state_const(png_ptr) {
            if (state.flags & PNG_FLAG_BENIGN_ERRORS_WARN) != 0 {
                png_chunk_warning(png_ptr, error_message);
            } else {
                png_chunk_error(png_ptr, error_message);
            }
        }
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
        if let Some(state) = png_ptr_state(png_ptr) {
            state.error_ptr = error_ptr;
            state.error_fn = error_fn;
            state.warning_fn = warning_fn;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_error_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        png_ptr_state_const(png_ptr)
            .map(|state| state.error_ptr)
            .unwrap_or(core::ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_longjmp_fn(
    png_ptr: png_structrp,
    longjmp_fn: png_longjmp_ptr,
    jmp_buf_size: usize,
) -> *mut JmpBuf {
    crate::abi_guard!(png_ptr, {
        let Some(state) = png_ptr_state(png_ptr) else {
            return core::ptr::null_mut();
        };
        let builtin_jmp_buf_size = state.longjmp_storage_size;

        if state.jmp_buf_ptr.is_null() {
            if state.longjmp_storage.is_null() || builtin_jmp_buf_size == 0 {
                return core::ptr::null_mut();
            }

            if jmp_buf_size <= builtin_jmp_buf_size {
                state.jmp_buf_ptr = crate::state::png_safe_longjmp_state_buf(state.longjmp_storage)
                    .cast::<JmpBuf>();
                state.jmp_buf_size = 0;
            } else {
                let allocated = crate::memory::png_malloc_warn(png_ptr, jmp_buf_size);
                if allocated.is_null() {
                    return core::ptr::null_mut();
                }

                state.jmp_buf_ptr = allocated.cast::<JmpBuf>();
                state.jmp_buf_size = jmp_buf_size;
            }
        } else {
            let allocated_size = if state.jmp_buf_size == 0 {
                builtin_jmp_buf_size
            } else {
                state.jmp_buf_size
            };

            if allocated_size != jmp_buf_size {
                png_warning(
                    png_ptr,
                    b"Application jmp_buf size changed\0".as_ptr().cast(),
                );
                return core::ptr::null_mut();
            }
        }

        state.longjmp_fn = longjmp_fn;
        state.jmp_buf_ptr
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_longjmp(png_ptr: png_const_structrp, val: c_int) -> ! {
    if let Some(state) = png_ptr_state_const(png_ptr) {
        if let (Some(longjmp_fn), false) = (state.longjmp_fn, state.jmp_buf_ptr.is_null()) {
            longjmp_fn(state.jmp_buf_ptr, val);
        }
    }

    libc::abort();
}
