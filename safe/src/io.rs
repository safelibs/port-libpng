use crate::state;
use crate::types::*;

unsafe extern "C" {
    fn runtime_png_get_io_ptr(png_ptr: png_const_structrp) -> png_voidp;
    fn runtime_png_init_io(png_ptr: png_structrp, fp: png_FILE_p);
    fn runtime_png_set_read_fn(png_ptr: png_structrp, io_ptr: png_voidp, read_data_fn: png_rw_ptr);
    fn runtime_png_set_write_fn(
        png_ptr: png_structrp,
        io_ptr: png_voidp,
        write_data_fn: png_rw_ptr,
        output_flush_fn: png_flush_ptr,
    );
    fn runtime_png_set_read_status_fn(png_ptr: png_structrp, read_row_fn: png_read_status_ptr);
    fn runtime_png_set_write_status_fn(png_ptr: png_structrp, write_row_fn: png_write_status_ptr);
    fn runtime_png_set_progressive_read_fn(
        png_ptr: png_structrp,
        progressive_ptr: png_voidp,
        info_fn: png_progressive_info_ptr,
        row_fn: png_progressive_row_ptr,
        end_fn: png_progressive_end_ptr,
    );
    fn runtime_png_get_progressive_ptr(png_ptr: png_const_structrp) -> png_voidp;
    fn runtime_png_set_read_user_chunk_fn(
        png_ptr: png_structrp,
        user_chunk_ptr: png_voidp,
        read_user_chunk_fn: png_user_chunk_ptr,
    );
    fn runtime_png_get_user_chunk_ptr(png_ptr: png_const_structrp) -> png_voidp;
    fn runtime_png_set_read_user_transform_fn(
        png_ptr: png_structrp,
        read_user_transform_fn: png_user_transform_ptr,
    );
    fn runtime_png_set_write_user_transform_fn(
        png_ptr: png_structrp,
        write_user_transform_fn: png_user_transform_ptr,
    );
    fn runtime_png_set_user_transform_info(
        png_ptr: png_structrp,
        user_transform_ptr: png_voidp,
        user_transform_depth: core::ffi::c_int,
        user_transform_channels: core::ffi::c_int,
    );
    fn runtime_png_get_user_transform_ptr(png_ptr: png_const_structrp) -> png_voidp;
    fn runtime_png_get_io_state(png_ptr: png_const_structrp) -> png_uint_32;
    fn runtime_png_get_io_chunk_type(png_ptr: png_const_structrp) -> png_uint_32;
}

unsafe extern "C" fn png_safe_read_user_chunk_trampoline(
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
        state::get_png(png_ptr.cast_mut())
            .map(|state| state.io_ptr)
            .unwrap_or_else(|| unsafe { runtime_png_get_io_ptr(png_ptr) })
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_init_io(png_ptr: png_structrp, fp: png_FILE_p) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_init_io(png_ptr, fp);
        state::update_png(png_ptr, |state| {
            state.io_ptr = fp.cast();
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_read_fn(
    png_ptr: png_structrp,
    io_ptr: png_voidp,
    read_data_fn: png_rw_ptr,
) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_read_fn(png_ptr, io_ptr, read_data_fn);
        state::update_png(png_ptr, |state| {
            state.io_ptr = io_ptr;
            state.read_data_fn = read_data_fn;
            state.write_data_fn = None;
            state.output_flush_fn = None;
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
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_write_fn(png_ptr, io_ptr, write_data_fn, output_flush_fn);
        state::update_png(png_ptr, |state| {
            state.io_ptr = io_ptr;
            state.read_data_fn = None;
            state.write_data_fn = write_data_fn;
            state.output_flush_fn = output_flush_fn;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_read_status_fn(
    png_ptr: png_structrp,
    read_row_fn: png_read_status_ptr,
) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_read_status_fn(png_ptr, read_row_fn);
        state::update_png(png_ptr, |state| {
            state.read_row_fn = read_row_fn;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_write_status_fn(
    png_ptr: png_structrp,
    write_row_fn: png_write_status_ptr,
) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_write_status_fn(png_ptr, write_row_fn);
        state::update_png(png_ptr, |state| {
            state.write_row_fn = write_row_fn;
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
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_progressive_read_fn(png_ptr, progressive_ptr, info_fn, row_fn, end_fn);
        state::update_png(png_ptr, |state| {
            state.io_ptr = progressive_ptr;
            state.read_data_fn = None;
            state.write_data_fn = None;
            state.output_flush_fn = None;
            state.progressive_ptr = progressive_ptr;
            state.progressive_info_fn = info_fn;
            state.progressive_row_fn = row_fn;
            state.progressive_end_fn = end_fn;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_progressive_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        state::get_png(png_ptr.cast_mut())
            .map(|state| state.io_ptr)
            .unwrap_or_else(|| unsafe { runtime_png_get_progressive_ptr(png_ptr) })
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_read_user_chunk_fn(
    png_ptr: png_structrp,
    user_chunk_ptr: png_voidp,
    read_user_chunk_fn: png_user_chunk_ptr,
) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_read_user_chunk_fn(
            png_ptr,
            user_chunk_ptr,
            if read_user_chunk_fn.is_some() {
                Some(png_safe_read_user_chunk_trampoline)
            } else {
                None
            },
        );
        state::update_png(png_ptr, |state| {
            state.user_chunk_ptr = user_chunk_ptr;
            state.read_user_chunk_fn = read_user_chunk_fn;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_user_chunk_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        state::get_png(png_ptr.cast_mut())
            .map(|state| state.user_chunk_ptr)
            .unwrap_or_else(|| unsafe { runtime_png_get_user_chunk_ptr(png_ptr) })
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_read_user_transform_fn(
    png_ptr: png_structrp,
    read_user_transform_fn: png_user_transform_ptr,
) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_read_user_transform_fn(png_ptr, read_user_transform_fn);
        state::update_png(png_ptr, |state| {
            state.read_user_transform_fn = read_user_transform_fn;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_write_user_transform_fn(
    png_ptr: png_structrp,
    write_user_transform_fn: png_user_transform_ptr,
) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_write_user_transform_fn(png_ptr, write_user_transform_fn);
        state::update_png(png_ptr, |state| {
            state.write_user_transform_fn = write_user_transform_fn;
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
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_user_transform_info(
            png_ptr,
            user_transform_ptr,
            user_transform_depth,
            user_transform_channels,
        );
        state::update_png(png_ptr, |state| {
            state.user_transform_ptr = user_transform_ptr;
            state.user_transform_depth = user_transform_depth;
            state.user_transform_channels = user_transform_channels;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_user_transform_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        state::get_png(png_ptr.cast_mut())
            .map(|state| state.user_transform_ptr)
            .unwrap_or_else(|| unsafe { runtime_png_get_user_transform_ptr(png_ptr) })
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_io_state(png_ptr: png_const_structrp) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        runtime_png_get_io_state(png_ptr)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_io_chunk_type(png_ptr: png_const_structrp) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        runtime_png_get_io_chunk_type(png_ptr)
    })
}

pub(crate) fn read_user_chunk_registration(
    png_ptr: png_const_structrp,
) -> Option<(png_voidp, png_user_chunk_ptr)> {
    state::get_png(png_ptr.cast_mut()).map(|state| (state.user_chunk_ptr, state.read_user_chunk_fn))
}

pub(crate) fn progressive_read_registration(
    png_ptr: png_const_structrp,
) -> Option<(
    png_voidp,
    png_progressive_info_ptr,
    png_progressive_row_ptr,
    png_progressive_end_ptr,
)> {
    state::get_png(png_ptr.cast_mut()).map(|state| {
        (
            state.progressive_ptr,
            state.progressive_info_fn,
            state.progressive_row_fn,
            state.progressive_end_fn,
        )
    })
}

pub(crate) fn write_callback_registration(
    png_ptr: png_const_structrp,
) -> Option<(png_voidp, png_rw_ptr, png_flush_ptr, png_write_status_ptr)> {
    state::get_png(png_ptr.cast_mut()).map(|state| {
        (
            state.io_ptr,
            state.write_data_fn,
            state.output_flush_fn,
            state.write_row_fn,
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
    state::get_png(png_ptr.cast_mut()).map(|state| {
        (
            state.write_user_transform_fn,
            state.user_transform_ptr,
            state.user_transform_depth,
            state.user_transform_channels,
        )
    })
}
