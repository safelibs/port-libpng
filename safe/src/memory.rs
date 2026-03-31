use crate::common::{
    PNG_DESTROY_WILL_FREE_DATA, PNG_FREE_MUL, PNG_FREE_ROWS, PNG_USER_WILL_FREE_DATA,
};
use crate::state::{self, PngStructState};
use crate::types::*;
use core::ffi::c_int;
use core::ptr;

unsafe fn alloc_raw(size: png_alloc_size_t, zeroed: bool) -> png_voidp {
    if size == 0 {
        return ptr::null_mut();
    }

    if zeroed {
        unsafe { libc::calloc(1, size).cast() }
    } else {
        unsafe { libc::malloc(size).cast() }
    }
}

unsafe fn alloc_impl(
    png_ptr: png_const_structrp,
    size: png_alloc_size_t,
    zeroed: bool,
    use_default: bool,
    warn: bool,
) -> png_voidp {
    let (malloc_fn, _mem_ptr) = if use_default {
        (None, ptr::null_mut())
    } else {
        state::with_png(png_ptr.cast_mut(), |png_state| (png_state.malloc_fn, png_state.mem_ptr))
            .unwrap_or((None, ptr::null_mut()))
    };

    let allocation = if let Some(callback) = malloc_fn {
        let ptr = unsafe { callback(png_ptr.cast_mut(), size) };
        if !ptr.is_null() && zeroed {
            unsafe {
                ptr::write_bytes(ptr.cast::<u8>(), 0, size);
            }
        }
        ptr
    } else {
        unsafe { alloc_raw(size, zeroed) }
    };

    if allocation.is_null() && !warn && !png_ptr.is_null() {
        unsafe {
            crate::error::png_error(png_ptr, c"Out of memory".as_ptr());
        }
    }

    allocation
}

unsafe fn free_impl(png_ptr: png_const_structrp, ptr_to_free: png_voidp, use_default: bool) {
    if ptr_to_free.is_null() {
        return;
    }

    let free_fn = if use_default {
        None
    } else {
        state::with_png(png_ptr.cast_mut(), |png_state| png_state.free_fn).flatten()
    };

    if let Some(callback) = free_fn {
        unsafe { callback(png_ptr.cast_mut(), ptr_to_free) };
    } else {
        unsafe { libc::free(ptr_to_free.cast()) };
    }
}

fn register_read_state(
    png_ptr: png_structrp,
    error_ptr: png_voidp,
    error_fn: png_error_ptr,
    warn_fn: png_error_ptr,
    mem_ptr: png_voidp,
    malloc_fn: png_malloc_ptr,
    free_fn: png_free_ptr,
) {
    state::register_png(
        png_ptr,
        PngStructState::new_read(error_ptr, error_fn, warn_fn, mem_ptr, malloc_fn, free_fn),
    );
}

fn register_write_state(
    png_ptr: png_structrp,
    error_ptr: png_voidp,
    error_fn: png_error_ptr,
    warn_fn: png_error_ptr,
    mem_ptr: png_voidp,
    malloc_fn: png_malloc_ptr,
    free_fn: png_free_ptr,
) {
    state::register_png(
        png_ptr,
        PngStructState::new_write(error_ptr, error_fn, warn_fn, mem_ptr, malloc_fn, free_fn),
    );
}

unsafe fn create_png_handle() -> png_structp {
    Box::into_raw(Box::new(png_struct { _private: 0 }))
}

unsafe fn create_info_handle() -> png_infop {
    Box::into_raw(Box::new(png_info { _private: 0 }))
}

unsafe fn release_png_handle(png_ptr: png_structp) {
    if !png_ptr.is_null() {
        unsafe {
            drop(Box::from_raw(png_ptr));
        }
    }
}

unsafe fn release_info_handle(info_ptr: png_infop) {
    if !info_ptr.is_null() {
        unsafe {
            drop(Box::from_raw(info_ptr));
        }
    }
}

unsafe fn create_png_struct_with_state(
    create: impl FnOnce() -> png_structp,
    register: impl FnOnce(png_structrp),
) -> png_structp {
    let mut png_ptr = ptr::null_mut();

    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        png_ptr = create();
        if !png_ptr.is_null() {
            register(png_ptr);
        }
        png_ptr
    })) {
        Ok(png_ptr) => png_ptr,
        Err(_) => {
            if !png_ptr.is_null() {
                unsafe { crate::error::panic_to_png_error(png_ptr) };
            }
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_calloc(
    png_ptr: png_const_structrp,
    size: png_alloc_size_t,
) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe { alloc_impl(png_ptr, size, true, false, false) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_malloc(
    png_ptr: png_const_structrp,
    size: png_alloc_size_t,
) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        alloc_impl(png_ptr, size, false, false, false)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_malloc_default(
    png_ptr: png_const_structrp,
    size: png_alloc_size_t,
) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        alloc_impl(png_ptr, size, false, true, false)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_malloc_warn(
    png_ptr: png_const_structrp,
    size: png_alloc_size_t,
) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        alloc_impl(png_ptr, size, false, false, true)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_free(png_ptr: png_const_structrp, ptr_to_free: png_voidp) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        free_impl(png_ptr, ptr_to_free, false)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_free_default(png_ptr: png_const_structrp, ptr_to_free: png_voidp) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        free_impl(png_ptr, ptr_to_free, true)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_mem_fn(
    png_ptr: png_structrp,
    mem_ptr: png_voidp,
    malloc_fn: png_malloc_ptr,
    free_fn: png_free_ptr,
) {
    crate::abi_guard!(png_ptr, {
        state::update_png(png_ptr, |png_state| {
            png_state.mem_ptr = mem_ptr;
            png_state.malloc_fn = malloc_fn;
            png_state.free_fn = free_fn;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_mem_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        state::with_png(png_ptr.cast_mut(), |png_state| png_state.mem_ptr).unwrap_or(ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_create_read_struct(
    _user_png_ver: png_const_charp,
    error_ptr: png_voidp,
    error_fn: png_error_ptr,
    warn_fn: png_error_ptr,
) -> png_structp {
    unsafe {
        create_png_struct_with_state(
            || create_png_handle(),
            |png_ptr| {
                register_read_state(
                    png_ptr,
                    error_ptr,
                    error_fn,
                    warn_fn,
                    ptr::null_mut(),
                    None,
                    None,
                );
            },
        )
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_create_read_struct_2(
    _user_png_ver: png_const_charp,
    error_ptr: png_voidp,
    error_fn: png_error_ptr,
    warn_fn: png_error_ptr,
    mem_ptr: png_voidp,
    malloc_fn: png_malloc_ptr,
    free_fn: png_free_ptr,
) -> png_structp {
    unsafe {
        create_png_struct_with_state(
            || create_png_handle(),
            |png_ptr| {
                register_read_state(
                    png_ptr, error_ptr, error_fn, warn_fn, mem_ptr, malloc_fn, free_fn,
                );
            },
        )
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_create_write_struct(
    _user_png_ver: png_const_charp,
    error_ptr: png_voidp,
    error_fn: png_error_ptr,
    warn_fn: png_error_ptr,
) -> png_structp {
    unsafe {
        create_png_struct_with_state(
            || create_png_handle(),
            |png_ptr| {
                register_write_state(
                    png_ptr,
                    error_ptr,
                    error_fn,
                    warn_fn,
                    ptr::null_mut(),
                    None,
                    None,
                );
            },
        )
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_create_write_struct_2(
    _user_png_ver: png_const_charp,
    error_ptr: png_voidp,
    error_fn: png_error_ptr,
    warn_fn: png_error_ptr,
    mem_ptr: png_voidp,
    malloc_fn: png_malloc_ptr,
    free_fn: png_free_ptr,
) -> png_structp {
    unsafe {
        create_png_struct_with_state(
            || create_png_handle(),
            |png_ptr| {
                register_write_state(
                    png_ptr, error_ptr, error_fn, warn_fn, mem_ptr, malloc_fn, free_fn,
                );
            },
        )
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_create_info_struct(_png_ptr: png_const_structrp) -> png_infop {
    crate::abi_guard_no_png!(unsafe {
        let info_ptr = create_info_handle();
        if !info_ptr.is_null() {
            state::register_default_info(info_ptr);
        }
        info_ptr
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_destroy_info_struct(
    _png_ptr: png_const_structrp,
    info_ptr_ptr: png_infopp,
) {
    crate::abi_guard_no_png!({
        if info_ptr_ptr.is_null() {
            return;
        }

        let info_ptr = unsafe { *info_ptr_ptr };
        if !info_ptr.is_null() {
            state::remove_info(info_ptr);
            unsafe { release_info_handle(info_ptr) };
            unsafe { *info_ptr_ptr = ptr::null_mut() };
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_info_init_3(ptr_ptr: png_infopp, _png_info_struct_size: usize) {
    crate::abi_guard_no_png!({
        if ptr_ptr.is_null() {
            return;
        }

        if unsafe { (*ptr_ptr).is_null() } {
            let info_ptr = unsafe { create_info_handle() };
            unsafe { *ptr_ptr = info_ptr };
            if !info_ptr.is_null() {
                state::register_default_info(info_ptr);
            }
        } else if state::get_info(unsafe { *ptr_ptr }).is_none() {
            state::register_default_info(unsafe { *ptr_ptr });
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_data_freer(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    freer: c_int,
    mask: png_uint_32,
) {
    crate::abi_guard_no_png!({
        state::update_info(info_ptr, |info_state| match freer {
            PNG_DESTROY_WILL_FREE_DATA => info_state.core.free_me |= mask,
            PNG_USER_WILL_FREE_DATA => info_state.core.free_me &= !mask,
            _ => {}
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_free_data(
    _png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    mask: png_uint_32,
    num: c_int,
) {
    crate::abi_guard_no_png!({
        state::update_info(info_ptr, |info_state| {
            if (mask & PNG_FREE_ROWS) != 0 && (info_state.core.free_me & PNG_FREE_ROWS) != 0 {
                info_state.core.row_pointers = ptr::null_mut();
            }

            let mut cleared_mask = mask;
            if num != -1 {
                cleared_mask &= !PNG_FREE_MUL;
            }
            info_state.core.free_me &= !cleared_mask;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_destroy_read_struct(
    png_ptr_ptr: png_structpp,
    info_ptr_ptr: png_infopp,
    end_info_ptr_ptr: png_infopp,
) {
    crate::abi_guard_no_png!({
        let png_ptr = if png_ptr_ptr.is_null() {
            ptr::null_mut()
        } else {
            unsafe { *png_ptr_ptr }
        };

        if !png_ptr.is_null() {
            if let Some(mut png_state) = state::remove_png(png_ptr) {
                unsafe { crate::error::release_longjmp_buffer(&mut png_state) };
            }
            unsafe { release_png_handle(png_ptr) };
            if !png_ptr_ptr.is_null() {
                unsafe { *png_ptr_ptr = ptr::null_mut() };
            }
        }

        if !info_ptr_ptr.is_null() {
            let info_ptr = unsafe { *info_ptr_ptr };
            if !info_ptr.is_null() {
                state::remove_info(info_ptr);
                unsafe { release_info_handle(info_ptr) };
                unsafe { *info_ptr_ptr = ptr::null_mut() };
            }
        }

        if !end_info_ptr_ptr.is_null() {
            let info_ptr = unsafe { *end_info_ptr_ptr };
            if !info_ptr.is_null() {
                state::remove_info(info_ptr);
                unsafe { release_info_handle(info_ptr) };
                unsafe { *end_info_ptr_ptr = ptr::null_mut() };
            }
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_destroy_write_struct(
    png_ptr_ptr: png_structpp,
    info_ptr_ptr: png_infopp,
) {
    crate::abi_guard_no_png!({
        let png_ptr = if png_ptr_ptr.is_null() {
            ptr::null_mut()
        } else {
            unsafe { *png_ptr_ptr }
        };

        if !png_ptr.is_null() {
            if let Some(mut png_state) = state::remove_png(png_ptr) {
                unsafe { crate::error::release_longjmp_buffer(&mut png_state) };
            }
            unsafe { release_png_handle(png_ptr) };
            if !png_ptr_ptr.is_null() {
                unsafe { *png_ptr_ptr = ptr::null_mut() };
            }
        }

        if !info_ptr_ptr.is_null() {
            let info_ptr = unsafe { *info_ptr_ptr };
            if !info_ptr.is_null() {
                state::remove_info(info_ptr);
                unsafe { release_info_handle(info_ptr) };
                unsafe { *info_ptr_ptr = ptr::null_mut() };
            }
        }
    });
}
