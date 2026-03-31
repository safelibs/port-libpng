use crate::types::*;

unsafe extern "C" {
    fn upstream_png_get_io_ptr(png_ptr: png_const_structrp) -> png_voidp;
    fn upstream_png_init_io(png_ptr: png_structrp, fp: png_FILE_p);
    fn upstream_png_set_read_fn(
        png_ptr: png_structrp,
        io_ptr: png_voidp,
        read_data_fn: png_rw_ptr,
    );
    fn upstream_png_set_write_fn(
        png_ptr: png_structrp,
        io_ptr: png_voidp,
        write_data_fn: png_rw_ptr,
        output_flush_fn: png_flush_ptr,
    );
    fn upstream_png_set_read_status_fn(
        png_ptr: png_structrp,
        read_row_fn: png_read_status_ptr,
    );
    fn upstream_png_set_write_status_fn(
        png_ptr: png_structrp,
        write_row_fn: png_write_status_ptr,
    );
    fn upstream_png_set_progressive_read_fn(
        png_ptr: png_structrp,
        progressive_ptr: png_voidp,
        info_fn: png_progressive_info_ptr,
        row_fn: png_progressive_row_ptr,
        end_fn: png_progressive_end_ptr,
    );
    fn upstream_png_get_progressive_ptr(png_ptr: png_const_structrp) -> png_voidp;
    fn upstream_png_set_read_user_chunk_fn(
        png_ptr: png_structrp,
        user_chunk_ptr: png_voidp,
        read_user_chunk_fn: png_user_chunk_ptr,
    );
    fn upstream_png_get_user_chunk_ptr(png_ptr: png_const_structrp) -> png_voidp;
    fn upstream_png_set_read_user_transform_fn(
        png_ptr: png_structrp,
        read_user_transform_fn: png_user_transform_ptr,
    );
    fn upstream_png_set_write_user_transform_fn(
        png_ptr: png_structrp,
        write_user_transform_fn: png_user_transform_ptr,
    );
    fn upstream_png_set_user_transform_info(
        png_ptr: png_structrp,
        user_transform_ptr: png_voidp,
        user_transform_depth: core::ffi::c_int,
        user_transform_channels: core::ffi::c_int,
    );
    fn upstream_png_get_user_transform_ptr(png_ptr: png_const_structrp) -> png_voidp;
    fn upstream_png_get_io_state(png_ptr: png_const_structrp) -> png_uint_32;
    fn upstream_png_get_io_chunk_type(png_ptr: png_const_structrp) -> png_uint_32;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_io_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe { upstream_png_get_io_ptr(png_ptr) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_init_io(png_ptr: png_structrp, fp: png_FILE_p) {
    crate::abi_guard!(png_ptr, unsafe { upstream_png_init_io(png_ptr, fp) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_read_fn(
    png_ptr: png_structrp,
    io_ptr: png_voidp,
    read_data_fn: png_rw_ptr,
) {
    crate::abi_guard!(png_ptr, unsafe { upstream_png_set_read_fn(png_ptr, io_ptr, read_data_fn) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_write_fn(
    png_ptr: png_structrp,
    io_ptr: png_voidp,
    write_data_fn: png_rw_ptr,
    output_flush_fn: png_flush_ptr,
) {
    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_set_write_fn(png_ptr, io_ptr, write_data_fn, output_flush_fn)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_read_status_fn(
    png_ptr: png_structrp,
    read_row_fn: png_read_status_ptr,
) {
    crate::abi_guard!(png_ptr, unsafe { upstream_png_set_read_status_fn(png_ptr, read_row_fn) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_write_status_fn(
    png_ptr: png_structrp,
    write_row_fn: png_write_status_ptr,
) {
    crate::abi_guard!(png_ptr, unsafe { upstream_png_set_write_status_fn(png_ptr, write_row_fn) });
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
        upstream_png_set_progressive_read_fn(png_ptr, progressive_ptr, info_fn, row_fn, end_fn)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_progressive_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_get_progressive_ptr(png_ptr)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_read_user_chunk_fn(
    png_ptr: png_structrp,
    user_chunk_ptr: png_voidp,
    read_user_chunk_fn: png_user_chunk_ptr,
) {
    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_set_read_user_chunk_fn(png_ptr, user_chunk_ptr, read_user_chunk_fn)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_user_chunk_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_get_user_chunk_ptr(png_ptr)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_read_user_transform_fn(
    png_ptr: png_structrp,
    read_user_transform_fn: png_user_transform_ptr,
) {
    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_set_read_user_transform_fn(png_ptr, read_user_transform_fn)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_write_user_transform_fn(
    png_ptr: png_structrp,
    write_user_transform_fn: png_user_transform_ptr,
) {
    crate::abi_guard!(png_ptr, unsafe {
        upstream_png_set_write_user_transform_fn(png_ptr, write_user_transform_fn)
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
        upstream_png_set_user_transform_info(
            png_ptr,
            user_transform_ptr,
            user_transform_depth,
            user_transform_channels,
        )
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_user_transform_ptr(png_ptr: png_const_structrp) -> png_voidp {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_get_user_transform_ptr(png_ptr)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_io_state(png_ptr: png_const_structrp) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe { upstream_png_get_io_state(png_ptr) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_io_chunk_type(png_ptr: png_const_structrp) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        upstream_png_get_io_chunk_type(png_ptr)
    })
}
