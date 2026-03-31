use crate::types::*;
use core::ffi::c_int;
use libc::FILE;

unsafe extern "C" {
    fn upstream_png_image_begin_read_from_file(
        image: png_imagep,
        file_name: png_const_charp,
    ) -> c_int;
    fn upstream_png_image_begin_read_from_stdio(image: png_imagep, file: *mut FILE) -> c_int;
    fn upstream_png_image_begin_read_from_memory(
        image: png_imagep,
        memory: png_const_voidp,
        size: usize,
    ) -> c_int;
    fn upstream_png_image_finish_read(
        image: png_imagep,
        background: png_const_colorp,
        buffer: png_voidp,
        row_stride: png_int_32,
        colormap: png_voidp,
    ) -> c_int;
    fn upstream_png_image_free(image: png_imagep);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_begin_read_from_file(
    image: png_imagep,
    file_name: png_const_charp,
) -> c_int {
    crate::abi_guard_no_png!(unsafe { upstream_png_image_begin_read_from_file(image, file_name) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_begin_read_from_stdio(image: png_imagep, file: *mut FILE) -> c_int {
    crate::abi_guard_no_png!(unsafe { upstream_png_image_begin_read_from_stdio(image, file) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_begin_read_from_memory(
    image: png_imagep,
    memory: png_const_voidp,
    size: usize,
) -> c_int {
    crate::abi_guard_no_png!(unsafe {
        upstream_png_image_begin_read_from_memory(image, memory, size)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_finish_read(
    image: png_imagep,
    background: png_const_colorp,
    buffer: png_voidp,
    row_stride: png_int_32,
    colormap: png_voidp,
) -> c_int {
    crate::abi_guard_no_png!(unsafe {
        upstream_png_image_finish_read(image, background, buffer, row_stride, colormap)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_free(image: png_imagep) {
    crate::abi_guard_no_png!(unsafe { upstream_png_image_free(image) });
}
