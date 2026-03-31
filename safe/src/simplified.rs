use crate::types::*;
use core::ffi::{c_char, c_int};
use libc::FILE;

const PNG_IMAGE_VERSION: png_uint_32 = 1;
const PNG_IMAGE_ERROR: png_uint_32 = 2;

const PNG_FORMAT_FLAG_ALPHA: png_uint_32 = 0x01;
const PNG_FORMAT_FLAG_COLOR: png_uint_32 = 0x02;
const PNG_FORMAT_FLAG_LINEAR: png_uint_32 = 0x04;
const PNG_FORMAT_FLAG_COLORMAP: png_uint_32 = 0x08;

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

fn sample_channels(format: png_uint_32) -> png_uint_32 {
    (format & (PNG_FORMAT_FLAG_COLOR | PNG_FORMAT_FLAG_ALPHA)) + 1
}

fn sample_component_size(format: png_uint_32) -> png_uint_32 {
    ((format & PNG_FORMAT_FLAG_LINEAR) >> 2) + 1
}

fn pixel_channels(format: png_uint_32) -> png_uint_32 {
    if (format & PNG_FORMAT_FLAG_COLORMAP) != 0 {
        1
    } else {
        sample_channels(format)
    }
}

fn pixel_component_size(format: png_uint_32) -> png_uint_32 {
    if (format & PNG_FORMAT_FLAG_COLORMAP) != 0 {
        1
    } else {
        sample_component_size(format)
    }
}

fn write_image_message(image: png_imagep, message: &[u8]) {
    if image.is_null() {
        return;
    }

    let image = unsafe { &mut *image };
    image.message.fill(0);

    let end = message
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(message.len())
        .min(image.message.len().saturating_sub(1));

    for (dst, src) in image.message.iter_mut().zip(message.iter()).take(end) {
        *dst = *src as c_char;
    }
}

fn image_error(image: png_imagep, message: &[u8]) -> c_int {
    if !image.is_null() {
        write_image_message(image, message);
        unsafe {
            (*image).warning_or_error |= PNG_IMAGE_ERROR;
        }
        unsafe { png_image_free(image) };
    }

    0
}

fn validate_finish_read(
    image: png_imagep,
    buffer: png_voidp,
    row_stride: png_int_32,
    colormap: png_voidp,
) -> Result<(), c_int> {
    if image.is_null() {
        return Ok(());
    }

    let image_ref = unsafe { &*image };
    if image_ref.version != PNG_IMAGE_VERSION {
        return Err(image_error(
            image,
            b"png_image_finish_read: damaged PNG_IMAGE_VERSION\0",
        ));
    }

    let channels = pixel_channels(image_ref.format);
    let Some(min_stride) = image_ref.width.checked_mul(channels) else {
        return Err(image_error(
            image,
            b"png_image_finish_read: row_stride too large\0",
        ));
    };
    if min_stride > i32::MAX as png_uint_32 {
        return Err(image_error(
            image,
            b"png_image_finish_read: row_stride too large\0",
        ));
    }

    let stride = if row_stride == 0 {
        min_stride
    } else {
        let Some(abs_stride) = row_stride.checked_abs() else {
            return Err(image_error(
                image,
                b"png_image_finish_read: row_stride too large\0",
            ));
        };
        let abs_stride = abs_stride as png_uint_32;
        if abs_stride < min_stride {
            return Err(image_error(
                image,
                b"png_image_finish_read: invalid argument\0",
            ));
        }
        abs_stride
    };

    if image_ref.opaque.is_null() || buffer.is_null() {
        return Err(image_error(
            image,
            b"png_image_finish_read: invalid argument\0",
        ));
    }

    let component_size = pixel_component_size(image_ref.format);
    let Some(total_bytes) = u64::from(image_ref.height)
        .checked_mul(u64::from(component_size))
        .and_then(|value| value.checked_mul(u64::from(stride)))
    else {
        return Err(image_error(
            image,
            b"png_image_finish_read: image too large\0",
        ));
    };
    if total_bytes > u64::from(u32::MAX) {
        return Err(image_error(
            image,
            b"png_image_finish_read: image too large\0",
        ));
    }

    if (image_ref.format & PNG_FORMAT_FLAG_COLORMAP) != 0
        && (image_ref.colormap_entries == 0 || colormap.is_null())
    {
        return Err(image_error(
            image,
            b"png_image_finish_read[color-map]: no color-map\0",
        ));
    }

    Ok(())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_begin_read_from_file(
    image: png_imagep,
    file_name: png_const_charp,
) -> c_int {
    crate::abi_guard_no_png!(unsafe { upstream_png_image_begin_read_from_file(image, file_name) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_begin_read_from_stdio(
    image: png_imagep,
    file: *mut FILE,
) -> c_int {
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
    crate::abi_guard_no_png!({
        if let Err(result) = validate_finish_read(image, buffer, row_stride, colormap) {
            return result;
        }

        unsafe { upstream_png_image_finish_read(image, background, buffer, row_stride, colormap) }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_free(image: png_imagep) {
    crate::abi_guard_no_png!(unsafe { upstream_png_image_free(image) });
}
