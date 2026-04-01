use crate::common::{
    PNG_DESTROY_WILL_FREE_DATA, PNG_FREE_MUL, PNG_FREE_ROWS, PNG_FREE_SCAL, PNG_FREE_TEXT,
    PNG_INFO_sCAL, PNG_USER_WILL_FREE_DATA,
};
use crate::state::{self, PngStructState};
use crate::types::*;
use core::ffi::c_int;
use core::mem::{MaybeUninit, size_of};
use core::ptr;

#[cfg(all(target_arch = "x86_64", target_env = "gnu", target_os = "linux"))]
#[repr(C)]
struct CreateJmpBufTag {
    __jmpbuf: [libc::c_long; 8],
    __mask_was_saved: c_int,
    __saved_mask: libc::sigset_t,
}

#[cfg(all(target_arch = "x86_64", target_env = "gnu", target_os = "linux"))]
type CreateJmpBuf = [CreateJmpBufTag; 1];

#[cfg(all(target_arch = "x86_64", target_env = "gnu", target_os = "linux"))]
unsafe extern "C" {
    fn _setjmp(env: *mut CreateJmpBufTag) -> c_int;
    fn longjmp(env: *mut CreateJmpBufTag, value: c_int) -> !;
}

#[cfg(all(target_arch = "x86_64", target_env = "gnu", target_os = "linux"))]
unsafe extern "C" fn create_longjmp(env: png_jmpbufp, value: c_int) {
    let jump_value = if value == 0 { 1 } else { value };
    unsafe { longjmp(env.cast::<CreateJmpBufTag>(), jump_value) }
}

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
    unsafe { alloc_raw(size_of::<png_struct>(), true).cast() }
}

unsafe fn create_info_handle() -> png_infop {
    unsafe { alloc_raw(size_of::<png_info_alias_text_prefix>(), true).cast() }
}

unsafe fn release_png_handle(png_ptr: png_structp) {
    if !png_ptr.is_null() {
        unsafe {
            libc::free(png_ptr.cast());
        }
    }
}

unsafe fn release_info_handle(info_ptr: png_infop) {
    if !info_ptr.is_null() {
        unsafe {
            libc::free(info_ptr.cast());
        }
    }
}

unsafe fn allocate_png_handle_with_callbacks(
    error_ptr: png_voidp,
    error_fn: png_error_ptr,
    warn_fn: png_error_ptr,
    mem_ptr: png_voidp,
    malloc_fn: png_malloc_ptr,
    free_fn: png_free_ptr,
) -> png_structp {
    if malloc_fn.is_none() {
        return unsafe { create_png_handle() };
    }

    #[cfg(all(target_arch = "x86_64", target_env = "gnu", target_os = "linux"))]
    {
        let mut create_handle = png_struct { _private: 0 };
        let create_ptr: png_structrp = &mut create_handle;
        let mut create_state =
            PngStructState::new_read(error_ptr, error_fn, warn_fn, mem_ptr, malloc_fn, free_fn);
        let mut create_jmp = MaybeUninit::<CreateJmpBuf>::uninit();
        let mut allocated: png_structp = ptr::null_mut();

        create_state.longjmp_fn = Some(create_longjmp);
        create_state.jmp_buf_ptr = create_jmp.as_mut_ptr().cast();
        create_state.jmp_buf_size = 0;
        state::register_png(create_ptr, create_state);

        if unsafe { _setjmp(create_jmp.as_mut_ptr().cast::<CreateJmpBufTag>()) } == 0 {
            allocated = unsafe {
                alloc_impl(create_ptr, size_of::<png_struct>(), false, false, true).cast()
            };
            if !allocated.is_null() {
                unsafe {
                    ptr::write_bytes(allocated.cast::<u8>(), 0, size_of::<png_struct>());
                }
            }
        }

        state::remove_png(create_ptr);
        return allocated;
    }

    #[cfg(not(all(target_arch = "x86_64", target_env = "gnu", target_os = "linux")))]
    {
        let _ = (error_ptr, error_fn, warn_fn, mem_ptr, free_fn);
        unsafe {
            let png_ptr = alloc_impl(
                ptr::null_mut(),
                size_of::<png_struct>(),
                false,
                false,
                true,
            )
            .cast::<png_struct>();
            if !png_ptr.is_null() {
                ptr::write_bytes(png_ptr.cast::<u8>(), 0, size_of::<png_struct>());
            }
            png_ptr
        }
    }
}

unsafe fn create_png_struct_with_state(mut png_state: PngStructState) -> png_structp {
    let png_ptr = unsafe {
        allocate_png_handle_with_callbacks(
            png_state.error_ptr,
            png_state.error_fn,
            png_state.warning_fn,
            png_state.mem_ptr,
            png_state.malloc_fn,
            png_state.free_fn,
        )
    };
    if png_ptr.is_null() {
        return ptr::null_mut();
    }

    if png_state.is_read_struct {
        png_state.core.mode |= crate::common::PNG_IS_READ_STRUCT;
    } else {
        png_state.core.mode &= !crate::common::PNG_IS_READ_STRUCT;
    }
    state::register_png(png_ptr, png_state);
    png_ptr
}

unsafe fn destroy_png_handle(png_ptr: png_structp) {
    let Some(png_state) = state::get_png(png_ptr) else {
        unsafe { release_png_handle(png_ptr) };
        return;
    };

    let mut proxy_ptr = ptr::null_mut();
    if png_state.free_fn.is_some() {
        proxy_ptr = unsafe { create_png_handle() };
        if !proxy_ptr.is_null() {
            state::register_png(proxy_ptr, png_state.clone());
        }
    }

    let free_context = if proxy_ptr.is_null() { png_ptr } else { proxy_ptr };
    unsafe { free_impl(free_context, png_ptr.cast(), false) };
    if !png_state.jmp_buf_ptr.is_null() {
        unsafe { free_impl(free_context, png_state.jmp_buf_ptr.cast(), false) };
    }
    state::remove_png(png_ptr);

    if !proxy_ptr.is_null() {
        state::remove_png(proxy_ptr);
        unsafe { release_png_handle(proxy_ptr) };
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
        create_png_struct_with_state(PngStructState::new_read(
            error_ptr,
            error_fn,
            warn_fn,
            ptr::null_mut(),
            None,
            None,
        ))
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
        create_png_struct_with_state(PngStructState::new_read(
            error_ptr, error_fn, warn_fn, mem_ptr, malloc_fn, free_fn,
        ))
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
        create_png_struct_with_state(PngStructState::new_write(
            error_ptr,
            error_fn,
            warn_fn,
            ptr::null_mut(),
            None,
            None,
        ))
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
        create_png_struct_with_state(PngStructState::new_write(
            error_ptr, error_fn, warn_fn, mem_ptr, malloc_fn, free_fn,
        ))
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
            if (mask & PNG_FREE_TEXT) != 0 && (info_state.core.free_me & PNG_FREE_TEXT) != 0 {
                if num == -1 {
                    info_state.text_chunks.clear();
                    info_state.text_cache.clear();
                    info_state.text_key_storage.clear();
                    info_state.text_value_storage.clear();
                    info_state.text_lang_storage.clear();
                    info_state.text_lang_key_storage.clear();
                }
            }

            if (mask & PNG_FREE_ROWS) != 0 && (info_state.core.free_me & PNG_FREE_ROWS) != 0 {
                info_state.core.row_pointers = ptr::null_mut();
            }

            if (mask & PNG_FREE_SCAL) != 0 && (info_state.core.free_me & PNG_FREE_SCAL) != 0 {
                info_state.scal_unit = 0;
                info_state.scal_width.clear();
                info_state.scal_height.clear();
                info_state.core.valid &= !PNG_INFO_sCAL;
            }

            let mut cleared_mask = mask;
            if num != -1 {
                cleared_mask &= !PNG_FREE_MUL;
            }
            info_state.core.free_me &= !cleared_mask;
        });
        unsafe { crate::bridge_ffi::png_safe_sync_png_info_aliases(ptr::null_mut(), info_ptr) };
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
            if !png_ptr_ptr.is_null() {
                unsafe { *png_ptr_ptr = ptr::null_mut() };
            }
            unsafe { destroy_png_handle(png_ptr) };
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
            if !png_ptr_ptr.is_null() {
                unsafe { *png_ptr_ptr = ptr::null_mut() };
            }
            unsafe { destroy_png_handle(png_ptr) };
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
