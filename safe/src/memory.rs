use crate::common::{
    PNG_DESTROY_WILL_FREE_DATA, PNG_FREE_ALL, PNG_FREE_EXIF, PNG_FREE_HIST, PNG_FREE_ICCP,
    PNG_FREE_MUL, PNG_FREE_PCAL, PNG_FREE_PLTE, PNG_FREE_ROWS, PNG_FREE_SCAL, PNG_FREE_SPLT,
    PNG_FREE_TEXT, PNG_FREE_TRNS, PNG_FREE_UNKN, PNG_INFO_IDAT, PNG_INFO_PLTE, PNG_INFO_eXIf,
    PNG_INFO_hIST, PNG_INFO_iCCP, PNG_INFO_pCAL, PNG_INFO_sCAL, PNG_INFO_sPLT, PNG_INFO_tRNS,
    PNG_LIBPNG_VER_STRING, PNG_USER_WILL_FREE_DATA, matches_version, safecat, zero_bytes,
};
use crate::error::png_warning;
use crate::io::{initialize_default_read_io, initialize_default_write_io};
use crate::state::{
    PngInfoState, PngStructState, alloc_longjmp_storage, info_ptr_state, png_ptr_state,
};
use crate::types::*;
use core::ffi::c_int;
use core::mem;
use core::ptr;

pub(crate) unsafe fn malloc_base(png_ptr: png_const_structrp, size: png_alloc_size_t) -> png_voidp {
    if size == 0 {
        return ptr::null_mut();
    }

    if let Some(state) = png_ptr_state(png_ptr.cast_mut()) {
        if let Some(malloc_fn) = state.malloc_fn {
            return malloc_fn(png_ptr.cast_mut(), size);
        }
    }

    libc::malloc(size)
}

unsafe fn malloc_base_with_template(template: png_structrp, size: png_alloc_size_t) -> png_voidp {
    if size == 0 {
        return ptr::null_mut();
    }

    if let Some(state) = png_ptr_state(template) {
        if let Some(malloc_fn) = state.malloc_fn {
            return malloc_fn(template, size);
        }
    }

    libc::malloc(size)
}

unsafe fn free_with_template(template: png_structrp, ptr_to_free: png_voidp) {
    if ptr_to_free.is_null() {
        return;
    }

    if let Some(state) = png_ptr_state(template) {
        if let Some(free_fn) = state.free_fn {
            free_fn(template, ptr_to_free);
            return;
        }
    }

    libc::free(ptr_to_free);
}

unsafe extern "C" fn internal_longjmp(jmp_buf_ptr: *mut JmpBuf, value: c_int) {
    crate::state::png_safe_longjmp_state_jump(jmp_buf_ptr.cast(), value)
}

unsafe fn create_png_struct(
    user_png_ver: png_const_charp,
    error_ptr: png_voidp,
    error_fn: png_error_ptr,
    warn_fn: png_error_ptr,
    mem_ptr: png_voidp,
    malloc_fn: png_malloc_ptr,
    free_fn: png_free_ptr,
    is_read: bool,
) -> png_structp {
    let mut template = PngStructState::defaults(
        is_read, error_ptr, error_fn, warn_fn, mem_ptr, malloc_fn, free_fn,
    );

    if !matches_version(user_png_ver) {
        let mut message = [0i8; 128];
        let mut pos = 0usize;
        pos = safecat(
            message.as_mut_ptr(),
            message.len(),
            pos,
            b"Application built with libpng-\0".as_ptr().cast(),
        );
        pos = safecat(message.as_mut_ptr(), message.len(), pos, user_png_ver);
        pos = safecat(
            message.as_mut_ptr(),
            message.len(),
            pos,
            b" but running with \0".as_ptr().cast(),
        );
        let _ = safecat(
            message.as_mut_ptr(),
            message.len(),
            pos,
            PNG_LIBPNG_VER_STRING.as_ptr().cast(),
        );
        png_warning(
            (&mut template as *mut PngStructState).cast(),
            message.as_ptr(),
        );
        return ptr::null_mut();
    }

    let raw = malloc_base_with_template(
        (&mut template as *mut PngStructState).cast(),
        mem::size_of::<PngStructState>(),
    )
    .cast::<PngStructState>();
    if raw.is_null() {
        return ptr::null_mut();
    }

    let (longjmp_storage, longjmp_storage_size, _) = alloc_longjmp_storage();
    if longjmp_storage.is_null() {
        free_with_template((&mut template as *mut PngStructState).cast(), raw.cast());
        return ptr::null_mut();
    }

    let mut state = template;
    state.longjmp_storage = longjmp_storage;
    state.longjmp_storage_size = longjmp_storage_size;
    state.jmp_buf_ptr = ptr::null_mut();
    state.jmp_buf_size = 0;

    ptr::write(raw, state);
    let png_ptr = raw.cast::<png_struct>();
    if is_read {
        initialize_default_read_io(png_ptr);
    } else {
        initialize_default_write_io(png_ptr);
    }

    png_ptr
}

unsafe fn destroy_png_struct(png_ptr: png_structrp) {
    let Some(state) = png_ptr_state(png_ptr) else {
        return;
    };

    let destroy_dummy = *state;
    let heap_jmp_buf = if destroy_dummy.jmp_buf_size > 0 {
        destroy_dummy.jmp_buf_ptr
    } else {
        ptr::null_mut()
    };

    free_with_template(png_ptr, png_ptr.cast());

    if !heap_jmp_buf.is_null() && !destroy_dummy.longjmp_storage.is_null() {
        let mut cleanup_dummy = destroy_dummy;
        let cleanup_dummy_png_ptr =
            (&mut cleanup_dummy as *mut PngStructState).cast::<png_struct>();
        cleanup_dummy.jmp_buf_ptr =
            crate::state::png_safe_longjmp_state_buf(cleanup_dummy.longjmp_storage)
                .cast::<JmpBuf>();
        cleanup_dummy.jmp_buf_size = 0;
        cleanup_dummy.longjmp_fn = Some(internal_longjmp);

        if crate::state::png_safe_longjmp_state_set(cleanup_dummy.longjmp_storage) == 0 {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                free_with_template(cleanup_dummy_png_ptr, heap_jmp_buf.cast());
            }));
        }
    }

    if !destroy_dummy.longjmp_storage.is_null() {
        libc::free(destroy_dummy.longjmp_storage);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_calloc(
    png_ptr: png_const_structrp,
    size: png_alloc_size_t,
) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        let ret = png_malloc(png_ptr, size);
        if !ret.is_null() {
            zero_bytes(ret, size);
        }
        ret
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_malloc(
    png_ptr: png_const_structrp,
    size: png_alloc_size_t,
) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() {
            return ptr::null_mut();
        }

        let ret = malloc_base(png_ptr, size);
        if ret.is_null() {
            crate::error::png_error(png_ptr, b"Out of memory\0".as_ptr().cast());
        }
        ret
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_malloc_default(
    png_ptr: png_const_structrp,
    size: png_alloc_size_t,
) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() {
            return ptr::null_mut();
        }

        let ret = if size == 0 {
            ptr::null_mut()
        } else {
            libc::malloc(size)
        };

        if ret.is_null() {
            crate::error::png_error(png_ptr, b"Out of Memory\0".as_ptr().cast());
        }

        ret
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_malloc_warn(
    png_ptr: png_const_structrp,
    size: png_alloc_size_t,
) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() {
            return ptr::null_mut();
        }

        let ret = malloc_base(png_ptr, size);
        if ret.is_null() {
            png_warning(png_ptr, b"Out of memory\0".as_ptr().cast());
        }
        ret
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_free(png_ptr: png_const_structrp, ptr_to_free: png_voidp) {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() || ptr_to_free.is_null() {
            return;
        }

        if let Some(state) = png_ptr_state(png_ptr.cast_mut()) {
            if let Some(free_fn) = state.free_fn {
                free_fn(png_ptr.cast_mut(), ptr_to_free);
                return;
            }
        }

        libc::free(ptr_to_free);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_free_default(png_ptr: png_const_structrp, ptr_to_free: png_voidp) {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() || ptr_to_free.is_null() {
            return;
        }
        libc::free(ptr_to_free);
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
        if let Some(state) = png_ptr_state(png_ptr) {
            state.mem_ptr = mem_ptr;
            state.malloc_fn = malloc_fn;
            state.free_fn = free_fn;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_mem_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        png_ptr_state(png_ptr.cast_mut())
            .map(|state| state.mem_ptr)
            .unwrap_or(ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_create_read_struct(
    user_png_ver: png_const_charp,
    error_ptr: png_voidp,
    error_fn: png_error_ptr,
    warn_fn: png_error_ptr,
) -> png_structp {
    crate::abi_guard_no_png!(png_create_read_struct_2(
        user_png_ver,
        error_ptr,
        error_fn,
        warn_fn,
        ptr::null_mut(),
        None,
        None,
    ))
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
    crate::abi_guard_no_png!(create_png_struct(
        user_png_ver,
        error_ptr,
        error_fn,
        warn_fn,
        mem_ptr,
        malloc_fn,
        free_fn,
        true,
    ))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_create_write_struct(
    user_png_ver: png_const_charp,
    error_ptr: png_voidp,
    error_fn: png_error_ptr,
    warn_fn: png_error_ptr,
) -> png_structp {
    crate::abi_guard_no_png!(png_create_write_struct_2(
        user_png_ver,
        error_ptr,
        error_fn,
        warn_fn,
        ptr::null_mut(),
        None,
        None,
    ))
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
    crate::abi_guard_no_png!(create_png_struct(
        user_png_ver,
        error_ptr,
        error_fn,
        warn_fn,
        mem_ptr,
        malloc_fn,
        free_fn,
        false,
    ))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_create_info_struct(png_ptr: png_const_structrp) -> png_infop {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() {
            return ptr::null_mut();
        }

        let raw = malloc_base(png_ptr, mem::size_of::<PngInfoState>()).cast::<PngInfoState>();
        if raw.is_null() {
            return ptr::null_mut();
        }

        ptr::write(raw, PngInfoState::zeroed());
        raw.cast::<png_info>()
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_destroy_info_struct(
    png_ptr: png_const_structrp,
    info_ptr_ptr: png_infopp,
) {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() || info_ptr_ptr.is_null() {
            return;
        }

        let info_ptr = *info_ptr_ptr;
        if info_ptr.is_null() {
            return;
        }

        *info_ptr_ptr = ptr::null_mut();
        png_free_data(png_ptr, info_ptr, PNG_FREE_ALL, -1);
        zero_bytes(info_ptr.cast(), mem::size_of::<PngInfoState>());
        png_free(png_ptr, info_ptr.cast());
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_info_init_3(ptr_ptr: png_infopp, png_info_struct_size: usize) {
    crate::abi_guard_no_png!({
        if ptr_ptr.is_null() || (*ptr_ptr).is_null() {
            return;
        }

        if mem::size_of::<PngInfoState>() > png_info_struct_size {
            libc::free((*ptr_ptr).cast());
            let new_ptr = libc::malloc(mem::size_of::<PngInfoState>()).cast::<png_info>();
            if new_ptr.is_null() {
                *ptr_ptr = ptr::null_mut();
                return;
            }
            *ptr_ptr = new_ptr;
        }

        zero_bytes((*ptr_ptr).cast(), mem::size_of::<PngInfoState>());
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_data_freer(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    freer: core::ffi::c_int,
    mask: png_uint_32,
) {
    crate::abi_guard!(png_ptr.cast_mut(), {
        let Some(info) = info_ptr_state(info_ptr) else {
            return;
        };
        if png_ptr.is_null() {
            return;
        }

        if freer == PNG_DESTROY_WILL_FREE_DATA {
            info.free_me |= mask;
        } else if freer == PNG_USER_WILL_FREE_DATA {
            info.free_me &= !mask;
        } else {
            crate::error::png_error(
                png_ptr,
                b"Unknown freer parameter in png_data_freer\0"
                    .as_ptr()
                    .cast(),
            );
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_free_data(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    mut mask: png_uint_32,
    num: core::ffi::c_int,
) {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() {
            return;
        }
        let Some(info) = info_ptr_state(info_ptr) else {
            return;
        };

        if !info.text.is_null() && ((mask & PNG_FREE_TEXT) & info.free_me) != 0 {
            if num != -1 {
                let item = info.text.add(num as usize);
                png_free(png_ptr, (*item).key.cast());
                (*item).key = ptr::null_mut();
            } else {
                for index in 0..info.num_text.max(0) as usize {
                    png_free(png_ptr, (*info.text.add(index)).key.cast());
                }
                png_free(png_ptr, info.text.cast());
                info.text = ptr::null_mut();
                info.num_text = 0;
                info.max_text = 0;
            }
        }

        if ((mask & PNG_FREE_TRNS) & info.free_me) != 0 {
            info.valid &= !PNG_INFO_tRNS;
            png_free(png_ptr, info.trans_alpha.cast());
            info.trans_alpha = ptr::null_mut();
            info.num_trans = 0;
            if let Some(state) = png_ptr_state(png_ptr.cast_mut()) {
                state.num_trans = 0;
            }
        }

        if ((mask & PNG_FREE_SCAL) & info.free_me) != 0 {
            png_free(png_ptr, info.scal_s_width.cast());
            png_free(png_ptr, info.scal_s_height.cast());
            info.scal_s_width = ptr::null_mut();
            info.scal_s_height = ptr::null_mut();
            info.valid &= !PNG_INFO_sCAL;
        }

        if ((mask & PNG_FREE_PCAL) & info.free_me) != 0 {
            png_free(png_ptr, info.pcal_purpose.cast());
            png_free(png_ptr, info.pcal_units.cast());
            info.pcal_purpose = ptr::null_mut();
            info.pcal_units = ptr::null_mut();
            if !info.pcal_params.is_null() {
                for index in 0..info.pcal_nparams as usize {
                    png_free(png_ptr, (*info.pcal_params.add(index)).cast());
                }
                png_free(png_ptr, info.pcal_params.cast());
                info.pcal_params = ptr::null_mut();
            }
            info.valid &= !PNG_INFO_pCAL;
        }

        if ((mask & PNG_FREE_ICCP) & info.free_me) != 0 {
            png_free(png_ptr, info.iccp_name.cast());
            png_free(png_ptr, info.iccp_profile.cast());
            info.iccp_name = ptr::null_mut();
            info.iccp_profile = ptr::null_mut();
            info.valid &= !PNG_INFO_iCCP;
        }

        if !info.splt_palettes.is_null() && ((mask & PNG_FREE_SPLT) & info.free_me) != 0 {
            if num != -1 {
                let item = info.splt_palettes.add(num as usize);
                png_free(png_ptr, (*item).name.cast());
                png_free(png_ptr, (*item).entries.cast());
                (*item).name = ptr::null_mut();
                (*item).entries = ptr::null_mut();
            } else {
                for index in 0..info.splt_palettes_num.max(0) as usize {
                    let item = info.splt_palettes.add(index);
                    png_free(png_ptr, (*item).name.cast());
                    png_free(png_ptr, (*item).entries.cast());
                }
                png_free(png_ptr, info.splt_palettes.cast());
                info.splt_palettes = ptr::null_mut();
                info.splt_palettes_num = 0;
                info.valid &= !PNG_INFO_sPLT;
            }
        }

        if !info.unknown_chunks.is_null() && ((mask & PNG_FREE_UNKN) & info.free_me) != 0 {
            if num != -1 {
                let item = info.unknown_chunks.add(num as usize);
                png_free(png_ptr, (*item).data.cast());
                (*item).data = ptr::null_mut();
            } else {
                for index in 0..info.unknown_chunks_num.max(0) as usize {
                    png_free(png_ptr, (*info.unknown_chunks.add(index)).data.cast());
                }
                png_free(png_ptr, info.unknown_chunks.cast());
                info.unknown_chunks = ptr::null_mut();
                info.unknown_chunks_num = 0;
            }
        }

        if ((mask & PNG_FREE_EXIF) & info.free_me) != 0 {
            png_free(png_ptr, info.eXIf_buf.cast());
            png_free(png_ptr, info.exif.cast());
            info.eXIf_buf = ptr::null_mut();
            info.exif = ptr::null_mut();
            info.valid &= !PNG_INFO_eXIf;
        }

        if ((mask & PNG_FREE_HIST) & info.free_me) != 0 {
            png_free(png_ptr, info.hist.cast());
            info.hist = ptr::null_mut();
            info.valid &= !PNG_INFO_hIST;
        }

        if ((mask & PNG_FREE_PLTE) & info.free_me) != 0 {
            png_free(png_ptr, info.palette.cast());
            info.palette = ptr::null_mut();
            info.valid &= !PNG_INFO_PLTE;
            info.num_palette = 0;
        }

        if ((mask & PNG_FREE_ROWS) & info.free_me) != 0 {
            if !info.row_pointers.is_null() {
                for row in 0..info.height as usize {
                    png_free(png_ptr, (*info.row_pointers.add(row)).cast());
                }
                png_free(png_ptr, info.row_pointers.cast());
                info.row_pointers = ptr::null_mut();
            }
            info.valid &= !PNG_INFO_IDAT;
        }

        if num != -1 {
            mask &= !PNG_FREE_MUL;
        }
        info.free_me &= !mask;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_destroy_read_struct(
    png_ptr_ptr: png_structpp,
    info_ptr_ptr: png_infopp,
    end_info_ptr_ptr: png_infopp,
) {
    if png_ptr_ptr.is_null() || (*png_ptr_ptr).is_null() {
        return;
    }

    let png_ptr = *png_ptr_ptr;
    crate::abi_guard!(png_ptr, {
        png_destroy_info_struct(png_ptr, end_info_ptr_ptr);
        png_destroy_info_struct(png_ptr, info_ptr_ptr);
        *png_ptr_ptr = ptr::null_mut();
        destroy_png_struct(png_ptr);
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_destroy_write_struct(
    png_ptr_ptr: png_structpp,
    info_ptr_ptr: png_infopp,
) {
    if png_ptr_ptr.is_null() || (*png_ptr_ptr).is_null() {
        return;
    }

    let png_ptr = *png_ptr_ptr;
    crate::abi_guard!(png_ptr, {
        png_destroy_info_struct(png_ptr, info_ptr_ptr);
        *png_ptr_ptr = ptr::null_mut();
        destroy_png_struct(png_ptr);
    })
}
