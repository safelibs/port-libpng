use crate::types::*;
use core::ffi::c_int;

unsafe extern "C" {
    fn upstream_png_calloc(png_ptr: png_const_structrp, size: png_alloc_size_t) -> png_voidp;
    fn upstream_png_malloc(png_ptr: png_const_structrp, size: png_alloc_size_t) -> png_voidp;
    fn upstream_png_malloc_default(
        png_ptr: png_const_structrp,
        size: png_alloc_size_t,
    ) -> png_voidp;
    fn upstream_png_malloc_warn(png_ptr: png_const_structrp, size: png_alloc_size_t) -> png_voidp;
    fn upstream_png_free(png_ptr: png_const_structrp, ptr_to_free: png_voidp);
    fn upstream_png_free_default(png_ptr: png_const_structrp, ptr_to_free: png_voidp);
    fn upstream_png_set_mem_fn(
        png_ptr: png_structrp,
        mem_ptr: png_voidp,
        malloc_fn: png_malloc_ptr,
        free_fn: png_free_ptr,
    );
    fn upstream_png_get_mem_ptr(png_ptr: png_const_structrp) -> png_voidp;
    fn upstream_png_create_read_struct(
        user_png_ver: png_const_charp,
        error_ptr: png_voidp,
        error_fn: png_error_ptr,
        warn_fn: png_error_ptr,
    ) -> png_structp;
    fn upstream_png_create_read_struct_2(
        user_png_ver: png_const_charp,
        error_ptr: png_voidp,
        error_fn: png_error_ptr,
        warn_fn: png_error_ptr,
        mem_ptr: png_voidp,
        malloc_fn: png_malloc_ptr,
        free_fn: png_free_ptr,
    ) -> png_structp;
    fn upstream_png_create_write_struct(
        user_png_ver: png_const_charp,
        error_ptr: png_voidp,
        error_fn: png_error_ptr,
        warn_fn: png_error_ptr,
    ) -> png_structp;
    fn upstream_png_create_write_struct_2(
        user_png_ver: png_const_charp,
        error_ptr: png_voidp,
        error_fn: png_error_ptr,
        warn_fn: png_error_ptr,
        mem_ptr: png_voidp,
        malloc_fn: png_malloc_ptr,
        free_fn: png_free_ptr,
    ) -> png_structp;
    fn upstream_png_create_info_struct(png_ptr: png_const_structrp) -> png_infop;
    fn upstream_png_destroy_info_struct(png_ptr: png_const_structrp, info_ptr_ptr: png_infopp);
    fn upstream_png_info_init_3(ptr_ptr: png_infopp, png_info_struct_size: usize);
    fn upstream_png_data_freer(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        freer: c_int,
        mask: png_uint_32,
    );
    fn upstream_png_free_data(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        mask: png_uint_32,
        num: c_int,
    );
    fn upstream_png_destroy_read_struct(
        png_ptr_ptr: png_structpp,
        info_ptr_ptr: png_infopp,
        end_info_ptr_ptr: png_infopp,
    );
    fn upstream_png_destroy_write_struct(png_ptr_ptr: png_structpp, info_ptr_ptr: png_infopp);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_calloc(
    png_ptr: png_const_structrp,
    size: png_alloc_size_t,
) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe { upstream_png_calloc(png_ptr, size) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_malloc(
    png_ptr: png_const_structrp,
    size: png_alloc_size_t,
) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe { upstream_png_malloc(png_ptr, size) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_malloc_default(
    png_ptr: png_const_structrp,
    size: png_alloc_size_t,
) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_malloc_default(png_ptr, size)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_malloc_warn(
    png_ptr: png_const_structrp,
    size: png_alloc_size_t,
) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe { upstream_png_malloc_warn(png_ptr, size) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_free(png_ptr: png_const_structrp, ptr_to_free: png_voidp) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_free(png_ptr, ptr_to_free)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_free_default(png_ptr: png_const_structrp, ptr_to_free: png_voidp) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_free_default(png_ptr, ptr_to_free)
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
        upstream_png_set_mem_fn(png_ptr, mem_ptr, malloc_fn, free_fn)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_mem_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe { upstream_png_get_mem_ptr(png_ptr) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_create_read_struct(
    user_png_ver: png_const_charp,
    error_ptr: png_voidp,
    error_fn: png_error_ptr,
    warn_fn: png_error_ptr,
) -> png_structp {
    crate::abi_guard_no_png!(unsafe {
        upstream_png_create_read_struct(user_png_ver, error_ptr, error_fn, warn_fn)
    })
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
    crate::abi_guard_no_png!(unsafe {
        upstream_png_create_read_struct_2(
            user_png_ver,
            error_ptr,
            error_fn,
            warn_fn,
            mem_ptr,
            malloc_fn,
            free_fn,
        )
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_create_write_struct(
    user_png_ver: png_const_charp,
    error_ptr: png_voidp,
    error_fn: png_error_ptr,
    warn_fn: png_error_ptr,
) -> png_structp {
    crate::abi_guard_no_png!(unsafe {
        upstream_png_create_write_struct(user_png_ver, error_ptr, error_fn, warn_fn)
    })
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
    crate::abi_guard_no_png!(unsafe {
        upstream_png_create_write_struct_2(
            user_png_ver,
            error_ptr,
            error_fn,
            warn_fn,
            mem_ptr,
            malloc_fn,
            free_fn,
        )
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_create_info_struct(png_ptr: png_const_structrp) -> png_infop {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe { upstream_png_create_info_struct(png_ptr) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_destroy_info_struct(
    png_ptr: png_const_structrp,
    info_ptr_ptr: png_infopp,
) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_destroy_info_struct(png_ptr, info_ptr_ptr)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_info_init_3(ptr_ptr: png_infopp, png_info_struct_size: usize) {
    crate::abi_guard_no_png!(unsafe { upstream_png_info_init_3(ptr_ptr, png_info_struct_size) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_data_freer(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    freer: c_int,
    mask: png_uint_32,
) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_data_freer(png_ptr, info_ptr, freer, mask)
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
        upstream_png_free_data(png_ptr, info_ptr, mask, num)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_destroy_read_struct(
    png_ptr_ptr: png_structpp,
    info_ptr_ptr: png_infopp,
    end_info_ptr_ptr: png_infopp,
) {
    let png_ptr = if png_ptr_ptr.is_null() {
        core::ptr::null_mut()
    } else {
        unsafe { *png_ptr_ptr }
    };

    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_destroy_read_struct(png_ptr_ptr, info_ptr_ptr, end_info_ptr_ptr)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_destroy_write_struct(
    png_ptr_ptr: png_structpp,
    info_ptr_ptr: png_infopp,
) {
    let png_ptr = if png_ptr_ptr.is_null() {
        core::ptr::null_mut()
    } else {
        unsafe { *png_ptr_ptr }
    };

    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_destroy_write_struct(png_ptr_ptr, info_ptr_ptr)
    });
}
