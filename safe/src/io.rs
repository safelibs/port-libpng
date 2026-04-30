use crate::chunks::read_core;
use crate::state;
use crate::types::*;

unsafe extern "C" fn stdio_read_data(png_ptr: png_structp, out: png_bytep, length: usize) {
    if png_ptr.is_null() || out.is_null() || length == 0 {
        return;
    }

    let io_ptr =
        state::with_png(png_ptr, |png_state| png_state.io_ptr).unwrap_or(core::ptr::null_mut());
    if io_ptr.is_null() {
        unsafe { crate::error::png_error(png_ptr, c"Read Error".as_ptr()) };
    }

    let read = unsafe { libc::fread(out.cast(), 1, length, io_ptr.cast()) };
    if read != length {
        unsafe { crate::error::png_error(png_ptr, c"Read Error".as_ptr()) };
    }
}

unsafe extern "C" fn stdio_write_data(png_ptr: png_structp, data: png_bytep, length: usize) {
    if png_ptr.is_null() || data.is_null() || length == 0 {
        return;
    }

    let io_ptr =
        state::with_png(png_ptr, |png_state| png_state.io_ptr).unwrap_or(core::ptr::null_mut());
    if io_ptr.is_null() {
        unsafe { crate::error::png_error(png_ptr, c"Write Error".as_ptr()) };
    }

    let written = unsafe { libc::fwrite(data.cast(), 1, length, io_ptr.cast()) };
    if written != length {
        unsafe { crate::error::png_error(png_ptr, c"Write Error".as_ptr()) };
    }
}

unsafe extern "C" fn stdio_flush(png_ptr: png_structp) {
    if png_ptr.is_null() {
        return;
    }

    let io_ptr =
        state::with_png(png_ptr, |png_state| png_state.io_ptr).unwrap_or(core::ptr::null_mut());
    if !io_ptr.is_null() {
        unsafe {
            libc::fflush(io_ptr.cast());
        }
    }
}

pub(crate) unsafe extern "C" fn png_safe_read_user_chunk_trampoline(
    png_ptr: png_structp,
    chunk: png_unknown_chunkp,
) -> core::ffi::c_int {
    if chunk.is_null() {
        return 0;
    }

    match crate::chunks::dispatch_user_chunk_callback(png_ptr, unsafe { &mut *chunk }) {
        Some(result) => result,
        None => 0,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_io_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        state::with_png(png_ptr.cast_mut(), |png_state| png_state.io_ptr)
            .unwrap_or(core::ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_init_io(png_ptr: png_structrp, fp: png_FILE_p) {
    crate::abi_guard!(png_ptr, {
        state::update_png(png_ptr, |png_state| {
            png_state.io_ptr = fp.cast();
            if png_state.is_read_struct {
                png_state.read_data_fn = Some(stdio_read_data);
                png_state.write_data_fn = None;
                png_state.output_flush_fn = None;
            } else {
                png_state.read_data_fn = None;
                png_state.write_data_fn = Some(stdio_write_data);
                png_state.output_flush_fn = Some(stdio_flush);
            }
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_read_fn(
    png_ptr: png_structrp,
    io_ptr: png_voidp,
    read_data_fn: png_rw_ptr,
) {
    crate::abi_guard!(png_ptr, {
        state::update_png(png_ptr, |png_state| {
            png_state.io_ptr = io_ptr;
            png_state.read_data_fn = read_data_fn;
            png_state.write_data_fn = None;
            png_state.output_flush_fn = None;
        });
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
        state::update_png(png_ptr, |png_state| {
            png_state.io_ptr = io_ptr;
            png_state.read_data_fn = None;
            png_state.write_data_fn = write_data_fn;
            png_state.output_flush_fn = output_flush_fn;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_read_status_fn(
    png_ptr: png_structrp,
    read_row_fn: png_read_status_ptr,
) {
    crate::abi_guard!(png_ptr, {
        state::update_png(png_ptr, |png_state| {
            png_state.read_row_fn = read_row_fn;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_write_status_fn(
    png_ptr: png_structrp,
    write_row_fn: png_write_status_ptr,
) {
    crate::abi_guard!(png_ptr, {
        state::update_png(png_ptr, |png_state| {
            png_state.write_row_fn = write_row_fn;
        });
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
        state::update_png(png_ptr, |png_state| {
            png_state.io_ptr = progressive_ptr;
            png_state.read_data_fn = None;
            png_state.write_data_fn = None;
            png_state.output_flush_fn = None;
            png_state.progressive_ptr = progressive_ptr;
            png_state.progressive_info_fn = info_fn;
            png_state.progressive_row_fn = row_fn;
            png_state.progressive_end_fn = end_fn;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_progressive_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        state::with_png(png_ptr.cast_mut(), |png_state| png_state.progressive_ptr)
            .unwrap_or(core::ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_read_user_chunk_fn(
    png_ptr: png_structrp,
    user_chunk_ptr: png_voidp,
    read_user_chunk_fn: png_user_chunk_ptr,
) {
    crate::abi_guard!(png_ptr, {
        state::update_png(png_ptr, |png_state| {
            png_state.user_chunk_ptr = user_chunk_ptr;
            png_state.read_user_chunk_fn = read_user_chunk_fn;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_user_chunk_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        state::with_png(png_ptr.cast_mut(), |png_state| png_state.user_chunk_ptr)
            .unwrap_or(core::ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_read_user_transform_fn(
    png_ptr: png_structrp,
    read_user_transform_fn: png_user_transform_ptr,
) {
    crate::abi_guard!(png_ptr, {
        state::update_png(png_ptr, |png_state| {
            png_state.read_user_transform_fn = read_user_transform_fn;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_write_user_transform_fn(
    png_ptr: png_structrp,
    write_user_transform_fn: png_user_transform_ptr,
) {
    crate::abi_guard!(png_ptr, {
        state::update_png(png_ptr, |png_state| {
            png_state.write_user_transform_fn = write_user_transform_fn;
        });
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
        state::update_png(png_ptr, |png_state| {
            png_state.user_transform_ptr = user_transform_ptr;
            png_state.user_transform_depth = user_transform_depth;
            png_state.user_transform_channels = user_transform_channels;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_user_transform_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        state::with_png(png_ptr.cast_mut(), |png_state| png_state.user_transform_ptr)
            .unwrap_or(core::ptr::null_mut())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_io_state(png_ptr: png_const_structrp) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), { read_core(png_ptr).io_state })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_io_chunk_type(png_ptr: png_const_structrp) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), { read_core(png_ptr).chunk_name })
}

pub(crate) fn read_user_chunk_registration(
    png_ptr: png_const_structrp,
) -> Option<(png_voidp, png_user_chunk_ptr)> {
    state::with_png(png_ptr.cast_mut(), |png_state| {
        (png_state.user_chunk_ptr, png_state.read_user_chunk_fn)
    })
}

pub(crate) fn progressive_read_registration(
    png_ptr: png_const_structrp,
) -> Option<(
    png_voidp,
    png_progressive_info_ptr,
    png_progressive_row_ptr,
    png_progressive_end_ptr,
)> {
    state::with_png(png_ptr.cast_mut(), |png_state| {
        (
            png_state.progressive_ptr,
            png_state.progressive_info_fn,
            png_state.progressive_row_fn,
            png_state.progressive_end_fn,
        )
    })
}

pub(crate) fn write_callback_registration(
    png_ptr: png_const_structrp,
) -> Option<(png_voidp, png_rw_ptr, png_flush_ptr, png_write_status_ptr)> {
    state::with_png(png_ptr.cast_mut(), |png_state| {
        (
            png_state.io_ptr,
            png_state.write_data_fn,
            png_state.output_flush_fn,
            png_state.write_row_fn,
        )
    })
}

pub(crate) fn write_user_transform_registration(
    png_ptr: png_const_structrp,
) -> Option<(
    png_user_transform_ptr,
    png_voidp,
    core::ffi::c_int,
    core::ffi::c_int,
)> {
    state::with_png(png_ptr.cast_mut(), |png_state| {
        (
            png_state.write_user_transform_fn,
            png_state.user_transform_ptr,
            png_state.user_transform_depth,
            png_state.user_transform_channels,
        )
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_io_ptr(png_ptr: png_const_structrp) -> png_voidp {
    unsafe { png_get_io_ptr(png_ptr) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_init_io(png_ptr: png_structrp, fp: png_FILE_p) {
    unsafe { png_init_io(png_ptr, fp) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_read_fn(
    png_ptr: png_structrp,
    io_ptr: png_voidp,
    read_data_fn: png_rw_ptr,
) {
    unsafe { png_set_read_fn(png_ptr, io_ptr, read_data_fn) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_write_fn(
    png_ptr: png_structrp,
    io_ptr: png_voidp,
    write_data_fn: png_rw_ptr,
    output_flush_fn: png_flush_ptr,
) {
    unsafe { png_set_write_fn(png_ptr, io_ptr, write_data_fn, output_flush_fn) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_read_status_fn(
    png_ptr: png_structrp,
    read_row_fn: png_read_status_ptr,
) {
    unsafe { png_set_read_status_fn(png_ptr, read_row_fn) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_write_status_fn(
    png_ptr: png_structrp,
    write_row_fn: png_write_status_ptr,
) {
    unsafe { png_set_write_status_fn(png_ptr, write_row_fn) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_progressive_read_fn(
    png_ptr: png_structrp,
    progressive_ptr: png_voidp,
    info_fn: png_progressive_info_ptr,
    row_fn: png_progressive_row_ptr,
    end_fn: png_progressive_end_ptr,
) {
    unsafe { png_set_progressive_read_fn(png_ptr, progressive_ptr, info_fn, row_fn, end_fn) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_progressive_ptr(png_ptr: png_const_structrp) -> png_voidp {
    unsafe { png_get_progressive_ptr(png_ptr) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_read_user_chunk_fn(
    png_ptr: png_structrp,
    user_chunk_ptr: png_voidp,
    read_user_chunk_fn: png_user_chunk_ptr,
) {
    unsafe { png_set_read_user_chunk_fn(png_ptr, user_chunk_ptr, read_user_chunk_fn) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_user_chunk_ptr(png_ptr: png_const_structrp) -> png_voidp {
    unsafe { png_get_user_chunk_ptr(png_ptr) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_read_user_transform_fn(
    png_ptr: png_structrp,
    read_user_transform_fn: png_user_transform_ptr,
) {
    unsafe { png_set_read_user_transform_fn(png_ptr, read_user_transform_fn) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_write_user_transform_fn(
    png_ptr: png_structrp,
    write_user_transform_fn: png_user_transform_ptr,
) {
    unsafe { png_set_write_user_transform_fn(png_ptr, write_user_transform_fn) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_set_user_transform_info(
    png_ptr: png_structrp,
    user_transform_ptr: png_voidp,
    user_transform_depth: core::ffi::c_int,
    user_transform_channels: core::ffi::c_int,
) {
    unsafe {
        png_set_user_transform_info(
            png_ptr,
            user_transform_ptr,
            user_transform_depth,
            user_transform_channels,
        )
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_user_transform_ptr(
    png_ptr: png_const_structrp,
) -> png_voidp {
    unsafe { png_get_user_transform_ptr(png_ptr) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_io_state(png_ptr: png_const_structrp) -> png_uint_32 {
    unsafe { png_get_io_state(png_ptr) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bridge_png_get_io_chunk_type(png_ptr: png_const_structrp) -> png_uint_32 {
    unsafe { png_get_io_chunk_type(png_ptr) }
}
