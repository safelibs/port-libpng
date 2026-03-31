use crate::common::{
    PNG_DESTROY_WILL_FREE_DATA, PNG_FREE_MUL, PNG_FREE_ROWS, PNG_USER_WILL_FREE_DATA,
};
use crate::state::{self, PngStructState};
use crate::types::*;
use core::ffi::c_int;
use core::ptr;

unsafe extern "C" {
    fn runtime_png_calloc(png_ptr: png_const_structrp, size: png_alloc_size_t) -> png_voidp;
    fn runtime_png_malloc(png_ptr: png_const_structrp, size: png_alloc_size_t) -> png_voidp;
    fn runtime_png_malloc_default(
        png_ptr: png_const_structrp,
        size: png_alloc_size_t,
    ) -> png_voidp;
    fn runtime_png_malloc_warn(png_ptr: png_const_structrp, size: png_alloc_size_t) -> png_voidp;
    fn runtime_png_free(png_ptr: png_const_structrp, ptr_to_free: png_voidp);
    fn runtime_png_free_default(png_ptr: png_const_structrp, ptr_to_free: png_voidp);
    fn runtime_png_set_mem_fn(
        png_ptr: png_structrp,
        mem_ptr: png_voidp,
        malloc_fn: png_malloc_ptr,
        free_fn: png_free_ptr,
    );
    fn runtime_png_get_mem_ptr(png_ptr: png_const_structrp) -> png_voidp;
    fn runtime_png_create_read_struct(
        user_png_ver: png_const_charp,
        error_ptr: png_voidp,
        error_fn: png_error_ptr,
        warn_fn: png_error_ptr,
    ) -> png_structp;
    fn runtime_png_create_read_struct_2(
        user_png_ver: png_const_charp,
        error_ptr: png_voidp,
        error_fn: png_error_ptr,
        warn_fn: png_error_ptr,
        mem_ptr: png_voidp,
        malloc_fn: png_malloc_ptr,
        free_fn: png_free_ptr,
    ) -> png_structp;
    fn runtime_png_create_write_struct(
        user_png_ver: png_const_charp,
        error_ptr: png_voidp,
        error_fn: png_error_ptr,
        warn_fn: png_error_ptr,
    ) -> png_structp;
    fn runtime_png_create_write_struct_2(
        user_png_ver: png_const_charp,
        error_ptr: png_voidp,
        error_fn: png_error_ptr,
        warn_fn: png_error_ptr,
        mem_ptr: png_voidp,
        malloc_fn: png_malloc_ptr,
        free_fn: png_free_ptr,
    ) -> png_structp;
    fn runtime_png_create_info_struct(png_ptr: png_const_structrp) -> png_infop;
    fn runtime_png_destroy_info_struct(png_ptr: png_const_structrp, info_ptr_ptr: png_infopp);
    fn runtime_png_info_init_3(ptr_ptr: png_infopp, png_info_struct_size: usize);
    fn runtime_png_data_freer(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        freer: c_int,
        mask: png_uint_32,
    );
    fn runtime_png_free_data(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        mask: png_uint_32,
        num: c_int,
    );
    fn runtime_png_destroy_read_struct(
        png_ptr_ptr: png_structpp,
        info_ptr_ptr: png_infopp,
        end_info_ptr_ptr: png_infopp,
    );
    fn runtime_png_destroy_write_struct(png_ptr_ptr: png_structpp, info_ptr_ptr: png_infopp);
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
                crate::error::panic_to_png_error(png_ptr);
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
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        runtime_png_calloc(png_ptr, size)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_malloc(
    png_ptr: png_const_structrp,
    size: png_alloc_size_t,
) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        runtime_png_malloc(png_ptr, size)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_malloc_default(
    png_ptr: png_const_structrp,
    size: png_alloc_size_t,
) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        runtime_png_malloc_default(png_ptr, size)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_malloc_warn(
    png_ptr: png_const_structrp,
    size: png_alloc_size_t,
) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        runtime_png_malloc_warn(png_ptr, size)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_free(png_ptr: png_const_structrp, ptr_to_free: png_voidp) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        runtime_png_free(png_ptr, ptr_to_free)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_free_default(png_ptr: png_const_structrp, ptr_to_free: png_voidp) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        runtime_png_free_default(png_ptr, ptr_to_free)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_mem_fn(
    png_ptr: png_structrp,
    mem_ptr: png_voidp,
    malloc_fn: png_malloc_ptr,
    free_fn: png_free_ptr,
) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_mem_fn(png_ptr, mem_ptr, malloc_fn, free_fn);
        state::update_png(png_ptr, |state| {
            state.mem_ptr = mem_ptr;
            state.malloc_fn = malloc_fn;
            state.free_fn = free_fn;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_mem_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        state::get_png(png_ptr.cast_mut())
            .map(|state| state.mem_ptr)
            .unwrap_or_else(|| unsafe { runtime_png_get_mem_ptr(png_ptr) })
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_create_read_struct(
    user_png_ver: png_const_charp,
    error_ptr: png_voidp,
    error_fn: png_error_ptr,
    warn_fn: png_error_ptr,
) -> png_structp {
    unsafe {
        create_png_struct_with_state(
            || runtime_png_create_read_struct(user_png_ver, error_ptr, error_fn, warn_fn),
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
    user_png_ver: png_const_charp,
    error_ptr: png_voidp,
    error_fn: png_error_ptr,
    warn_fn: png_error_ptr,
    mem_ptr: png_voidp,
    malloc_fn: png_malloc_ptr,
    free_fn: png_free_ptr,
) -> png_structp {
    unsafe {
        create_png_struct_with_state(
            || {
                runtime_png_create_read_struct_2(
                    user_png_ver,
                    error_ptr,
                    error_fn,
                    warn_fn,
                    mem_ptr,
                    malloc_fn,
                    free_fn,
                )
            },
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
    user_png_ver: png_const_charp,
    error_ptr: png_voidp,
    error_fn: png_error_ptr,
    warn_fn: png_error_ptr,
) -> png_structp {
    unsafe {
        create_png_struct_with_state(
            || runtime_png_create_write_struct(user_png_ver, error_ptr, error_fn, warn_fn),
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
    user_png_ver: png_const_charp,
    error_ptr: png_voidp,
    error_fn: png_error_ptr,
    warn_fn: png_error_ptr,
    mem_ptr: png_voidp,
    malloc_fn: png_malloc_ptr,
    free_fn: png_free_ptr,
) -> png_structp {
    unsafe {
        create_png_struct_with_state(
            || {
                runtime_png_create_write_struct_2(
                    user_png_ver,
                    error_ptr,
                    error_fn,
                    warn_fn,
                    mem_ptr,
                    malloc_fn,
                    free_fn,
                )
            },
            |png_ptr| {
                register_write_state(
                    png_ptr, error_ptr, error_fn, warn_fn, mem_ptr, malloc_fn, free_fn,
                );
            },
        )
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_create_info_struct(png_ptr: png_const_structrp) -> png_infop {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        let info_ptr = runtime_png_create_info_struct(png_ptr);
        if !info_ptr.is_null() {
            state::register_default_info(info_ptr);
        }
        info_ptr
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_destroy_info_struct(
    png_ptr: png_const_structrp,
    info_ptr_ptr: png_infopp,
) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        let info_ptr = if png_ptr.is_null() || info_ptr_ptr.is_null() {
            ptr::null_mut()
        } else {
            *info_ptr_ptr
        };

        if !png_ptr.is_null() && !info_ptr.is_null() {
            state::remove_info(info_ptr);
        }

        runtime_png_destroy_info_struct(png_ptr, info_ptr_ptr);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_info_init_3(ptr_ptr: png_infopp, png_info_struct_size: usize) {
    crate::abi_guard_no_png!(unsafe {
        let old_info_ptr = if ptr_ptr.is_null() {
            ptr::null_mut()
        } else {
            *ptr_ptr
        };

        runtime_png_info_init_3(ptr_ptr, png_info_struct_size);

        if !ptr_ptr.is_null() {
            let new_info_ptr = *ptr_ptr;
            if old_info_ptr != new_info_ptr {
                state::move_info(old_info_ptr, new_info_ptr);
            } else if !new_info_ptr.is_null() && state::get_info(new_info_ptr).is_none() {
                state::register_default_info(new_info_ptr);
            }
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_data_freer(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    freer: c_int,
    mask: png_uint_32,
) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        runtime_png_data_freer(png_ptr, info_ptr, freer, mask);
        state::update_info(info_ptr, |state| match freer {
            PNG_DESTROY_WILL_FREE_DATA => state.free_me |= mask,
            PNG_USER_WILL_FREE_DATA => state.free_me &= !mask,
            _ => {}
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_free_data(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    mask: png_uint_32,
    num: c_int,
) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        runtime_png_free_data(png_ptr, info_ptr, mask, num);
        state::update_info(info_ptr, |state| {
            if (mask & PNG_FREE_ROWS) != 0 && (state.free_me & PNG_FREE_ROWS) != 0 {
                state.row_pointers = ptr::null_mut();
            }

            let mut cleared_mask = mask;
            if num != -1 {
                cleared_mask &= !PNG_FREE_MUL;
            }
            state.free_me &= !cleared_mask;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_destroy_read_struct(
    png_ptr_ptr: png_structpp,
    info_ptr_ptr: png_infopp,
    end_info_ptr_ptr: png_infopp,
) {
    crate::abi_guard!(
        if png_ptr_ptr.is_null() {
            ptr::null_mut()
        } else {
            unsafe { *png_ptr_ptr }
        },
        unsafe {
            let png_ptr = if png_ptr_ptr.is_null() {
                ptr::null_mut()
            } else {
                *png_ptr_ptr
            };

            if !png_ptr.is_null() {
                state::remove_png(png_ptr);

                if !info_ptr_ptr.is_null() {
                    state::remove_info(*info_ptr_ptr);
                }
                if !end_info_ptr_ptr.is_null() {
                    state::remove_info(*end_info_ptr_ptr);
                }
            }

            runtime_png_destroy_read_struct(png_ptr_ptr, info_ptr_ptr, end_info_ptr_ptr)
        }
    );
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_destroy_write_struct(
    png_ptr_ptr: png_structpp,
    info_ptr_ptr: png_infopp,
) {
    crate::abi_guard!(
        if png_ptr_ptr.is_null() {
            ptr::null_mut()
        } else {
            unsafe { *png_ptr_ptr }
        },
        unsafe {
            let png_ptr = if png_ptr_ptr.is_null() {
                ptr::null_mut()
            } else {
                *png_ptr_ptr
            };

            if !png_ptr.is_null() {
                state::remove_png(png_ptr);

                if !info_ptr_ptr.is_null() {
                    state::remove_info(*info_ptr_ptr);
                }
            }

            runtime_png_destroy_write_struct(png_ptr_ptr, info_ptr_ptr)
        }
    );
}
