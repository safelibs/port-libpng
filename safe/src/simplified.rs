use crate::bridge_ffi::*;
use crate::types::*;
use core::ffi::{c_char, c_int};
use libc::FILE;

const PNG_IMAGE_VERSION: png_uint_32 = 1;
const PNG_IMAGE_ERROR: png_uint_32 = 2;

const PNG_FORMAT_FLAG_ALPHA: png_uint_32 = 0x01;
const PNG_FORMAT_FLAG_COLOR: png_uint_32 = 0x02;
const PNG_FORMAT_FLAG_LINEAR: png_uint_32 = 0x04;
const PNG_FORMAT_FLAG_COLORMAP: png_uint_32 = 0x08;

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

fn validate_image_header(image: png_imagep, operation: &[u8]) -> Result<(), c_int> {
    if image.is_null() {
        return Err(0);
    }

    let image_ref = unsafe { &*image };
    if image_ref.version != PNG_IMAGE_VERSION {
        let mut message = operation.to_vec();
        message.extend_from_slice(b": incorrect PNG_IMAGE_VERSION\0");
        return Err(image_error(image, &message));
    }

    Ok(())
}

fn validate_begin_read_from_file_args(
    image: png_imagep,
    file_name: png_const_charp,
) -> Result<(), c_int> {
    validate_image_header(image, b"png_image_begin_read_from_file")?;
    if file_name.is_null() {
        return Err(image_error(
            image,
            b"png_image_begin_read_from_file: invalid argument\0",
        ));
    }
    Ok(())
}

fn validate_begin_read_from_stdio_args(image: png_imagep, file: *mut FILE) -> Result<(), c_int> {
    validate_image_header(image, b"png_image_begin_read_from_stdio")?;
    if file.is_null() {
        return Err(image_error(
            image,
            b"png_image_begin_read_from_stdio: invalid argument\0",
        ));
    }
    Ok(())
}

fn validate_begin_read_from_memory_args(
    image: png_imagep,
    memory: png_const_voidp,
    size: usize,
) -> Result<(), c_int> {
    validate_image_header(image, b"png_image_begin_read_from_memory")?;
    if memory.is_null() || size == 0 {
        return Err(image_error(
            image,
            b"png_image_begin_read_from_memory: invalid argument\0",
        ));
    }
    Ok(())
}

fn validate_write_args(
    image: png_imagep,
    buffer: png_const_voidp,
    row_stride: png_int_32,
) -> Result<(), c_int> {
    validate_image_header(image, b"png_image_write")?;

    let image_ref = unsafe { &*image };
    if image_ref.width == 0 || image_ref.height == 0 || buffer.is_null() {
        return Err(image_error(image, b"png_image_write: invalid argument\0"));
    }
    if row_stride == i32::MIN {
        return Err(image_error(
            image,
            b"png_image_write: row_stride too large\0",
        ));
    }

    Ok(())
}

fn validate_write_to_file_args(
    image: png_imagep,
    file_name: png_const_charp,
    buffer: png_const_voidp,
    row_stride: png_int_32,
) -> Result<(), c_int> {
    validate_write_args(image, buffer, row_stride)?;
    if file_name.is_null() {
        return Err(image_error(
            image,
            b"png_image_write_to_file: invalid argument\0",
        ));
    }
    Ok(())
}

fn validate_write_to_stdio_args(
    image: png_imagep,
    file: *mut FILE,
    buffer: png_const_voidp,
    row_stride: png_int_32,
) -> Result<(), c_int> {
    validate_write_args(image, buffer, row_stride)?;
    if file.is_null() {
        return Err(image_error(
            image,
            b"png_image_write_to_stdio: invalid argument\0",
        ));
    }
    Ok(())
}

fn validate_write_to_memory_args(
    image: png_imagep,
    memory_bytes: *mut png_alloc_size_t,
    buffer: png_const_voidp,
    row_stride: png_int_32,
) -> Result<(), c_int> {
    validate_write_args(image, buffer, row_stride)?;
    if memory_bytes.is_null() {
        return Err(image_error(
            image,
            b"png_image_write_to_memory: invalid argument\0",
        ));
    }
    Ok(())
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
    crate::abi_guard_no_png!({
        if let Err(result) = validate_begin_read_from_file_args(image, file_name) {
            return result;
        }

        unsafe { image_begin_read_from_file(image, file_name) }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_begin_read_from_stdio(
    image: png_imagep,
    file: *mut FILE,
) -> c_int {
    crate::abi_guard_no_png!({
        if let Err(result) = validate_begin_read_from_stdio_args(image, file) {
            return result;
        }

        unsafe { image_begin_read_from_stdio(image, file) }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_begin_read_from_memory(
    image: png_imagep,
    memory: png_const_voidp,
    size: usize,
) -> c_int {
    crate::abi_guard_no_png!(unsafe {
        if let Err(result) = validate_begin_read_from_memory_args(image, memory, size) {
            return result;
        }

        image_begin_read_from_memory(image, memory, size)
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

        unsafe { image_finish_read(image, background, buffer, row_stride, colormap) }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_free(image: png_imagep) {
    crate::abi_guard_no_png!(unsafe { image_free(image) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_write_to_file(
    image: png_imagep,
    file_name: png_const_charp,
    convert_to_8bit: c_int,
    buffer: png_const_voidp,
    row_stride: png_int_32,
    colormap: png_const_voidp,
) -> c_int {
    crate::abi_guard_no_png!(unsafe {
        if let Err(result) = validate_write_to_file_args(image, file_name, buffer, row_stride) {
            return result;
        }

        image_write_to_file(
            image,
            file_name,
            convert_to_8bit,
            buffer,
            row_stride,
            colormap,
        )
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_write_to_stdio(
    image: png_imagep,
    file: *mut FILE,
    convert_to_8bit: c_int,
    buffer: png_const_voidp,
    row_stride: png_int_32,
    colormap: png_const_voidp,
) -> c_int {
    crate::abi_guard_no_png!(unsafe {
        if let Err(result) = validate_write_to_stdio_args(image, file, buffer, row_stride) {
            return result;
        }

        image_write_to_stdio(image, file, convert_to_8bit, buffer, row_stride, colormap)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_write_to_memory(
    image: png_imagep,
    memory: png_voidp,
    memory_bytes: *mut png_alloc_size_t,
    convert_to_8bit: c_int,
    buffer: png_const_voidp,
    row_stride: png_int_32,
    colormap: png_const_voidp,
) -> c_int {
    crate::abi_guard_no_png!(unsafe {
        if let Err(result) = validate_write_to_memory_args(image, memory_bytes, buffer, row_stride)
        {
            return result;
        }

        image_write_to_memory(
            image,
            memory,
            memory_bytes,
            convert_to_8bit,
            buffer,
            row_stride,
            colormap,
        )
    })
}
