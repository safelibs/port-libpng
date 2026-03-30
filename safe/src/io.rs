use crate::common::PNG_USER_TRANSFORM;
use crate::error::{png_app_error, png_warning};
use crate::state::{png_ptr_state, png_ptr_state_const};
use crate::types::*;
use core::ptr;

unsafe extern "C" fn default_read_data(png_ptr: png_structp, data: png_bytep, length: usize) {
    if png_ptr.is_null() {
        return;
    }

    let io_ptr = png_get_io_ptr(png_ptr).cast::<libc::FILE>();
    let check = libc::fread(data.cast(), 1, length, io_ptr);
    if check != length {
        crate::error::png_error(png_ptr, b"Read Error\0".as_ptr().cast());
    }
}

unsafe extern "C" fn default_write_data(png_ptr: png_structp, data: png_bytep, length: usize) {
    if png_ptr.is_null() {
        return;
    }

    let io_ptr = png_get_io_ptr(png_ptr).cast::<libc::FILE>();
    let check = libc::fwrite(data.cast(), 1, length, io_ptr);
    if check != length {
        crate::error::png_error(png_ptr, b"Write Error\0".as_ptr().cast());
    }
}

unsafe extern "C" fn default_flush(png_ptr: png_structp) {
    if png_ptr.is_null() {
        return;
    }

    let io_ptr = png_get_io_ptr(png_ptr).cast::<libc::FILE>();
    let _ = libc::fflush(io_ptr);
}

pub(crate) unsafe fn initialize_default_read_io(png_ptr: png_structp) {
    if let Some(state) = png_ptr_state(png_ptr) {
        state.read_data_fn = Some(default_read_data);
        state.write_data_fn = None;
        state.output_flush_fn = None;
    }
}

pub(crate) unsafe fn initialize_default_write_io(png_ptr: png_structp) {
    if let Some(state) = png_ptr_state(png_ptr) {
        state.write_data_fn = Some(default_write_data);
        state.read_data_fn = None;
        state.output_flush_fn = Some(default_flush);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_io_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        png_ptr_state_const(png_ptr)
            .map(|state| state.io_ptr)
            .unwrap_or(ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_init_io(png_ptr: png_structrp, fp: png_FILE_p) {
    crate::abi_guard!(png_ptr, {
        if let Some(state) = png_ptr_state(png_ptr) {
            state.io_ptr = fp.cast();
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_read_fn(
    png_ptr: png_structrp,
    io_ptr: png_voidp,
    read_data_fn: png_rw_ptr,
) {
    crate::abi_guard!(png_ptr, {
        let Some(state) = png_ptr_state(png_ptr) else {
            return;
        };

        state.io_ptr = io_ptr;
        state.read_data_fn = read_data_fn.or(Some(default_read_data));
        if state.write_data_fn.is_some() {
            state.write_data_fn = None;
            png_warning(
                png_ptr,
                b"Can't set both read_data_fn and write_data_fn in the same structure\0"
                    .as_ptr()
                    .cast(),
            );
        }
        state.output_flush_fn = None;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_write_fn(
    png_ptr: png_structrp,
    io_ptr: png_voidp,
    write_data_fn: png_rw_ptr,
    output_flush_fn: png_flush_ptr,
) {
    crate::abi_guard!(png_ptr, {
        let Some(state) = png_ptr_state(png_ptr) else {
            return;
        };

        state.io_ptr = io_ptr;
        state.write_data_fn = write_data_fn.or(Some(default_write_data));
        state.output_flush_fn = output_flush_fn.or(Some(default_flush));
        if state.read_data_fn.is_some() {
            state.read_data_fn = None;
            png_warning(
                png_ptr,
                b"Can't set both read_data_fn and write_data_fn in the same structure\0"
                    .as_ptr()
                    .cast(),
            );
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_read_status_fn(
    png_ptr: png_structrp,
    read_row_fn: png_read_status_ptr,
) {
    crate::abi_guard!(png_ptr, {
        if let Some(state) = png_ptr_state(png_ptr) {
            state.read_row_fn = read_row_fn;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_write_status_fn(
    png_ptr: png_structrp,
    write_row_fn: png_write_status_ptr,
) {
    crate::abi_guard!(png_ptr, {
        if let Some(state) = png_ptr_state(png_ptr) {
            state.write_row_fn = write_row_fn;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_progressive_read_fn(
    png_ptr: png_structrp,
    progressive_ptr: png_voidp,
    info_fn: png_progressive_info_ptr,
    row_fn: png_progressive_row_ptr,
    end_fn: png_progressive_end_ptr,
) {
    crate::abi_guard!(png_ptr, {
        let Some(state) = png_ptr_state(png_ptr) else {
            return;
        };

        state.info_fn = info_fn;
        state.row_fn = row_fn;
        state.end_fn = end_fn;
        state.io_ptr = progressive_ptr;
        state.read_data_fn = None;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_progressive_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), { png_get_io_ptr(png_ptr) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_read_user_chunk_fn(
    png_ptr: png_structrp,
    user_chunk_ptr: png_voidp,
    read_user_chunk_fn: png_user_chunk_ptr,
) {
    crate::abi_guard!(png_ptr, {
        if let Some(state) = png_ptr_state(png_ptr) {
            state.user_chunk_ptr = user_chunk_ptr;
            state.read_user_chunk_fn = read_user_chunk_fn;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_user_chunk_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        png_ptr_state_const(png_ptr)
            .map(|state| state.user_chunk_ptr)
            .unwrap_or(ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_read_user_transform_fn(
    png_ptr: png_structrp,
    read_user_transform_fn: png_user_transform_ptr,
) {
    crate::abi_guard!(png_ptr, {
        if let Some(state) = png_ptr_state(png_ptr) {
            state.transformations |= PNG_USER_TRANSFORM;
            state.read_user_transform_fn = read_user_transform_fn;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_write_user_transform_fn(
    png_ptr: png_structrp,
    write_user_transform_fn: png_user_transform_ptr,
) {
    crate::abi_guard!(png_ptr, {
        if let Some(state) = png_ptr_state(png_ptr) {
            state.transformations |= PNG_USER_TRANSFORM;
            state.write_user_transform_fn = write_user_transform_fn;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_user_transform_info(
    png_ptr: png_structrp,
    user_transform_ptr: png_voidp,
    user_transform_depth: core::ffi::c_int,
    user_transform_channels: core::ffi::c_int,
) {
    crate::abi_guard!(png_ptr, {
        let Some(state) = png_ptr_state(png_ptr) else {
            return;
        };

        if (state.mode & crate::common::PNG_IS_READ_STRUCT) != 0
            && (state.flags & crate::common::PNG_FLAG_ROW_INIT) != 0
        {
            png_app_error(
                png_ptr,
                b"info change after png_start_read_image or png_read_update_info\0"
                    .as_ptr()
                    .cast(),
            );
            return;
        }

        state.user_transform_ptr = user_transform_ptr;
        state.user_transform_depth = user_transform_depth as png_byte;
        state.user_transform_channels = user_transform_channels as png_byte;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_user_transform_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        png_ptr_state_const(png_ptr)
            .map(|state| state.user_transform_ptr)
            .unwrap_or(ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_io_state(png_ptr: png_const_structrp) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        png_ptr_state_const(png_ptr)
            .map(|state| state.io_state)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_io_chunk_type(png_ptr: png_const_structrp) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        png_ptr_state_const(png_ptr)
            .map(|state| state.chunk_name)
            .unwrap_or(0)
    })
}
