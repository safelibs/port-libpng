use crate::chunks::{read_core, read_info_core};
use crate::read::png_destroy_read_struct;
use crate::read_transform::{png_set_expand, png_set_gray_to_rgb, png_set_scale_16};
use crate::types::*;
use core::ffi::{c_char, c_int};
use core::{mem, ptr, slice};
use libc::FILE;

const PNG_IMAGE_VERSION: png_uint_32 = 1;
const PNG_IMAGE_WARNING: png_uint_32 = 1;
const PNG_IMAGE_ERROR: png_uint_32 = 2;
const PNG_IMAGE_FLAG_COLORSPACE_NOT_sRGB: png_uint_32 = 0x01;

const PNG_FORMAT_FLAG_ALPHA: png_uint_32 = 0x01;
const PNG_FORMAT_FLAG_COLOR: png_uint_32 = 0x02;
const PNG_FORMAT_FLAG_LINEAR: png_uint_32 = 0x04;
const PNG_FORMAT_FLAG_COLORMAP: png_uint_32 = 0x08;
const PNG_FORMAT_FLAG_BGR: png_uint_32 = 0x10;
const PNG_FORMAT_FLAG_AFIRST: png_uint_32 = 0x20;

const PNG_COLOR_MASK_PALETTE: png_byte = 1;
const PNG_COLOR_MASK_COLOR: png_byte = 2;
const PNG_COLOR_MASK_ALPHA: png_byte = 4;

const PNG_COLORSPACE_HAVE_ENDPOINTS: png_uint_16 = 0x0002;
const PNG_COLORSPACE_ENDPOINTS_MATCH_sRGB: png_uint_16 = 0x0040;
const PNG_COLORSPACE_INVALID: png_uint_16 = 0x8000;

#[derive(Debug)]
struct MemoryReader {
    memory: png_const_bytep,
    size: usize,
    offset: usize,
}

#[derive(Debug)]
struct SimplifiedReadControl {
    png_ptr: png_structp,
    info_ptr: png_infop,
    file: *mut FILE,
    owned_file: bool,
    memory: Option<MemoryReader>,
}

unsafe extern "C" {
    fn png_create_read_struct(
        user_png_ver: png_const_charp,
        error_ptr: png_voidp,
        error_fn: png_error_ptr,
        warn_fn: png_error_ptr,
    ) -> png_structp;
    fn png_create_info_struct(png_ptr: png_const_structrp) -> png_infop;
    fn png_init_io(png_ptr: png_structrp, fp: png_FILE_p);
    fn png_set_read_fn(png_ptr: png_structrp, io_ptr: png_voidp, read_data_fn: png_rw_ptr);
    fn png_set_benign_errors(png_ptr: png_structrp, allowed: c_int);
    fn png_get_error_ptr(png_ptr: png_const_structrp) -> png_voidp;
    fn png_get_io_ptr(png_ptr: png_const_structrp) -> png_voidp;
    fn png_longjmp(png_ptr: png_const_structrp, value: c_int) -> !;
    fn png_error(png_ptr: png_structrp, message: png_const_charp) -> !;

    fn png_safe_call_read_info(png_ptr: png_structrp, info_ptr: png_inforp) -> c_int;
    fn png_safe_call_read_update_info(png_ptr: png_structrp, info_ptr: png_inforp) -> c_int;
    fn png_safe_call_read_image(png_ptr: png_structrp, image: png_bytepp) -> c_int;
    fn png_safe_call_read_end(png_ptr: png_structrp, info_ptr: png_inforp) -> c_int;
}

fn supported_output_format(format: png_uint_32) -> bool {
    (format & PNG_FORMAT_FLAG_COLOR) != 0
        && (format & (PNG_FORMAT_FLAG_LINEAR | PNG_FORMAT_FLAG_COLORMAP)) == 0
}

fn output_channels(format: png_uint_32) -> usize {
    if (format & PNG_FORMAT_FLAG_ALPHA) != 0 {
        4
    } else {
        3
    }
}

fn pixel_row_stride(width: png_uint_32, format: png_uint_32) -> Option<png_uint_32> {
    width.checked_mul(output_channels(format) as png_uint_32)
}

unsafe fn reset_image(image: png_imagep) {
    ptr::write_bytes(image.cast::<u8>(), 0, mem::size_of::<png_image>());
    (*image).version = PNG_IMAGE_VERSION;
}

unsafe fn write_message_ptr(image: png_imagep, message: png_const_charp) {
    if image.is_null() {
        return;
    }

    let dst = &mut (*image).message;
    let limit = dst.len().saturating_sub(1);
    let mut len = if message.is_null() {
        0
    } else {
        libc::strlen(message.cast())
    };
    if len > limit {
        len = limit;
    }

    if len != 0 {
        ptr::copy_nonoverlapping(message, dst.as_mut_ptr(), len);
    }
    dst[len] = 0;
}

unsafe fn write_message_bytes(image: png_imagep, message: &[u8]) {
    if image.is_null() {
        return;
    }

    let dst = &mut (*image).message;
    let limit = dst.len().saturating_sub(1);
    let mut len = message
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(message.len());
    if len > limit {
        len = limit;
    }

    if len != 0 {
        ptr::copy_nonoverlapping(message.as_ptr().cast::<c_char>(), dst.as_mut_ptr(), len);
    }
    dst[len] = 0;
}

unsafe fn image_free_internal(image: png_imagep) {
    if image.is_null() || (*image).opaque.is_null() {
        return;
    }

    let control = Box::from_raw((*image).opaque.cast::<SimplifiedReadControl>());
    (*image).opaque = ptr::null_mut();

    if control.owned_file && !control.file.is_null() {
        let _ = libc::fclose(control.file);
    }

    let mut png_ptr = control.png_ptr;
    let mut info_ptr = control.info_ptr;
    unsafe {
        png_destroy_read_struct(&mut png_ptr, &mut info_ptr, ptr::null_mut());
    }
}

unsafe fn image_error_ptr(image: png_imagep, message: png_const_charp) -> c_int {
    if !image.is_null() {
        (*image).warning_or_error |= PNG_IMAGE_ERROR;
        write_message_ptr(image, message);
        image_free_internal(image);
    }
    0
}

unsafe fn image_error_bytes(image: png_imagep, message: &[u8]) -> c_int {
    if !image.is_null() {
        (*image).warning_or_error |= PNG_IMAGE_ERROR;
        write_message_bytes(image, message);
        image_free_internal(image);
    }
    0
}

unsafe extern "C" fn simplified_error(png_ptr: png_structp, message: png_const_charp) {
    let image = unsafe { png_get_error_ptr(png_ptr) }.cast::<png_image>();
    if !image.is_null() {
        unsafe {
            (*image).warning_or_error |= PNG_IMAGE_ERROR;
            write_message_ptr(image, message);
        }
    }
    unsafe {
        png_longjmp(png_ptr, 1);
    }
}

unsafe extern "C" fn simplified_warning(png_ptr: png_structp, message: png_const_charp) {
    let image = unsafe { png_get_error_ptr(png_ptr) }.cast::<png_image>();
    if image.is_null() {
        return;
    }

    if ((*image).warning_or_error & 0x03) == 0 {
        unsafe {
            write_message_ptr(image, message);
        }
    }
    unsafe {
        (*image).warning_or_error |= PNG_IMAGE_WARNING;
    }
}

unsafe extern "C" fn simplified_memory_read(
    png_ptr: png_structp,
    out: png_bytep,
    length: usize,
) {
    let reader = unsafe { png_get_io_ptr(png_ptr) }.cast::<MemoryReader>();
    if reader.is_null() {
        unsafe {
            png_error(png_ptr, b"invalid memory read\0".as_ptr().cast());
        }
    }

    let reader = unsafe { &mut *reader };
    let Some(end) = reader.offset.checked_add(length) else {
        unsafe {
            png_error(png_ptr, b"read beyond end of data\0".as_ptr().cast());
        }
    };

    if end > reader.size {
        unsafe {
            png_error(png_ptr, b"read beyond end of data\0".as_ptr().cast());
        }
    }

    unsafe {
        ptr::copy_nonoverlapping(reader.memory.add(reader.offset), out, length);
    }
    reader.offset = end;
}

unsafe fn create_control(image: png_imagep) -> Result<Box<SimplifiedReadControl>, ()> {
    let png_ptr = unsafe {
        png_create_read_struct(
            b"1.6.43\0".as_ptr().cast(),
            image.cast(),
            Some(simplified_error),
            Some(simplified_warning),
        )
    };
    if png_ptr.is_null() {
        return Err(());
    }

    let info_ptr = unsafe { png_create_info_struct(png_ptr) };
    if info_ptr.is_null() {
        let mut png_ptr = png_ptr;
        unsafe {
            png_destroy_read_struct(&mut png_ptr, ptr::null_mut(), ptr::null_mut());
        }
        return Err(());
    }

    Ok(Box::new(SimplifiedReadControl {
        png_ptr,
        info_ptr,
        file: ptr::null_mut(),
        owned_file: false,
        memory: None,
    }))
}

unsafe fn fill_header(image: png_imagep, control: &SimplifiedReadControl) {
    let core = unsafe { read_core(control.png_ptr) };
    let info = unsafe { read_info_core(control.info_ptr) };

    (*image).width = core.width;
    (*image).height = core.height;

    let mut format = 0u32;
    if (core.color_type & PNG_COLOR_MASK_COLOR) != 0 {
        format |= PNG_FORMAT_FLAG_COLOR;
    }
    if (core.color_type & PNG_COLOR_MASK_ALPHA) != 0 || info.num_trans > 0 {
        format |= PNG_FORMAT_FLAG_ALPHA;
    }
    if core.bit_depth == 16 {
        format |= PNG_FORMAT_FLAG_LINEAR;
    }
    if (core.color_type & PNG_COLOR_MASK_PALETTE) != 0 {
        format |= PNG_FORMAT_FLAG_COLORMAP;
    }
    (*image).format = format;

    (*image).flags = 0;
    if (format & PNG_FORMAT_FLAG_COLOR) != 0 {
        let endpoint_flags = core.colorspace.flags
            & (PNG_COLORSPACE_HAVE_ENDPOINTS
                | PNG_COLORSPACE_ENDPOINTS_MATCH_sRGB
                | PNG_COLORSPACE_INVALID);
        if endpoint_flags == PNG_COLORSPACE_HAVE_ENDPOINTS {
            (*image).flags |= PNG_IMAGE_FLAG_COLORSPACE_NOT_sRGB;
        }
    }

    (*image).colormap_entries = match core.color_type {
        0 => {
            let entries = 1u32.checked_shl(core.bit_depth.into()).unwrap_or(0);
            entries.min(256)
        }
        3 => u32::from(info.num_palette).min(256),
        _ => 256,
    };
}

unsafe fn begin_read_with_control(image: png_imagep, mut control: Box<SimplifiedReadControl>) -> c_int {
    unsafe {
        png_set_benign_errors(control.png_ptr, 1);
    }

    let control_ptr = control.as_mut() as *mut SimplifiedReadControl;
    unsafe {
        (*image).opaque = control_ptr.cast();
    }
    let _ = Box::into_raw(control);

    let control = unsafe { &mut *control_ptr };
    if unsafe { png_safe_call_read_info(control.png_ptr, control.info_ptr) } == 0 {
        unsafe {
            image_free_internal(image);
        }
        return 0;
    }

    unsafe {
        fill_header(image, control);
    }
    1
}

fn absolute_row_stride(row_stride: png_int_32) -> png_uint_32 {
    if row_stride < 0 {
        row_stride.unsigned_abs()
    } else {
        row_stride as png_uint_32
    }
}

fn premultiply(channel: u8, alpha: u8) -> u8 {
    (((u32::from(channel) * u32::from(alpha)) + 127) / 255) as u8
}

fn composite(channel: u8, alpha: u8, background: u8) -> u8 {
    (((u32::from(channel) * u32::from(alpha))
        + (u32::from(background) * u32::from(255 - alpha))
        + 127)
        / 255) as u8
}

fn write_output_pixel(format: png_uint_32, dst: &mut [u8], rgb: [u8; 3], alpha: u8) {
    let channels = output_channels(format);
    let bgr = (format & PNG_FORMAT_FLAG_BGR) != 0;
    let afirst = (format & PNG_FORMAT_FLAG_AFIRST) != 0;
    let ordered = if bgr {
        [rgb[2], rgb[1], rgb[0]]
    } else {
        rgb
    };

    if channels == 4 {
        if afirst {
            dst[0] = alpha;
            dst[1] = ordered[0];
            dst[2] = ordered[1];
            dst[3] = ordered[2];
        } else {
            dst[0] = ordered[0];
            dst[1] = ordered[1];
            dst[2] = ordered[2];
            dst[3] = alpha;
        }
    } else {
        dst[0] = ordered[0];
        dst[1] = ordered[1];
        dst[2] = ordered[2];
    }
}

unsafe fn finish_read_impl(
    image: png_imagep,
    control: &mut SimplifiedReadControl,
    background: png_const_colorp,
    buffer: png_voidp,
    row_stride: png_int_32,
) -> bool {
    let format = (*image).format;
    if !supported_output_format(format) {
        unsafe {
            image_error_bytes(image, b"png_image_finish_read: invalid argument\0");
        }
        return false;
    }

    let source = unsafe { read_core(control.png_ptr) };
    let source_info = unsafe { read_info_core(control.info_ptr) };
    let source_has_alpha =
        (source.color_type & PNG_COLOR_MASK_ALPHA) != 0 || source_info.num_trans > 0;

    unsafe {
        png_set_expand(control.png_ptr);
        png_set_gray_to_rgb(control.png_ptr);
        if source.bit_depth == 16 {
            png_set_scale_16(control.png_ptr);
        }
    }

    if unsafe { png_safe_call_read_update_info(control.png_ptr, control.info_ptr) } == 0 {
        return false;
    }

    let updated = unsafe { read_core(control.png_ptr) };
    let updated_info = unsafe { read_info_core(control.info_ptr) };
    let height = match usize::try_from(updated.height) {
        Ok(value) => value,
        Err(_) => {
            unsafe {
                image_error_bytes(image, b"png_image_finish_read: image too large\0");
            }
            return false;
        }
    };
    let rowbytes = if updated.info_rowbytes != 0 {
        updated.info_rowbytes
    } else if updated_info.rowbytes != 0 {
        updated_info.rowbytes
    } else {
        updated.rowbytes
    };
    let total_bytes = match rowbytes.checked_mul(height) {
        Some(value) => value,
        None => {
            unsafe {
                image_error_bytes(image, b"png_image_finish_read: image too large\0");
            }
            return false;
        }
    };

    let mut decoded = vec![0u8; total_bytes];
    let mut row_ptrs = Vec::<png_bytep>::with_capacity(height);
    for y in 0..height {
        row_ptrs.push(unsafe { decoded.as_mut_ptr().add(y * rowbytes) });
    }

    if unsafe { png_safe_call_read_image(control.png_ptr, row_ptrs.as_mut_ptr()) } == 0 {
        return false;
    }
    if unsafe { png_safe_call_read_end(control.png_ptr, ptr::null_mut()) } == 0 {
        return false;
    }

    let decoded_channels = usize::from(if updated_info.channels != 0 {
        updated_info.channels
    } else {
        updated.channels
    });
    if decoded_channels != 3 && decoded_channels != 4 {
        unsafe {
            image_error_bytes(image, b"png_image_finish_read: invalid argument\0");
        }
        return false;
    }

    let output_channels = output_channels(format);
    let stride = absolute_row_stride(row_stride) as usize;
    let buffer = buffer.cast::<u8>();
    let bg = if background.is_null() {
        [0u8, 0u8, 0u8]
    } else {
        unsafe { [(*background).red, (*background).green, (*background).blue] }
    };
    let want_alpha = (format & PNG_FORMAT_FLAG_ALPHA) != 0;
    let source_rows = decoded.as_slice();

    for y in 0..height {
        let src_row = &source_rows[y * rowbytes..(y + 1) * rowbytes];
        let dst_y = if row_stride < 0 { height - 1 - y } else { y };
        let dst_row = unsafe { slice::from_raw_parts_mut(buffer.add(dst_y * stride), stride) };

        for x in 0..((*image).width as usize) {
            let src = &src_row[x * decoded_channels..(x + 1) * decoded_channels];
            let alpha = if decoded_channels == 4 { src[3] } else { 255 };
            let rgb = if want_alpha {
                [premultiply(src[0], alpha), premultiply(src[1], alpha), premultiply(src[2], alpha)]
            } else if decoded_channels == 4 && source_has_alpha {
                [
                    composite(src[0], alpha, bg[0]),
                    composite(src[1], alpha, bg[1]),
                    composite(src[2], alpha, bg[2]),
                ]
            } else {
                [src[0], src[1], src[2]]
            };

            let dst = &mut dst_row[x * output_channels..(x + 1) * output_channels];
            write_output_pixel(format, dst, rgb, alpha);
        }
    }

    true
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_begin_read_from_file(
    image: png_imagep,
    file_name: png_const_charp,
) -> c_int {
    if image.is_null() {
        return 0;
    }
    if unsafe { (*image).version } != PNG_IMAGE_VERSION {
        return unsafe {
            image_error_bytes(
                image,
                b"png_image_begin_read_from_file: incorrect PNG_IMAGE_VERSION\0",
            )
        };
    }
    if file_name.is_null() {
        return unsafe {
            image_error_bytes(image, b"png_image_begin_read_from_file: invalid argument\0")
        };
    }
    if unsafe { !(*image).opaque.is_null() } {
        return unsafe { image_error_bytes(image, b"png_image_read: opaque pointer not NULL\0") };
    }

    let file = unsafe { libc::fopen(file_name.cast(), b"rb\0".as_ptr().cast()) };
    if file.is_null() {
        let errno = unsafe { *libc::__errno_location() };
        return unsafe { image_error_ptr(image, libc::strerror(errno)) };
    }

    unsafe {
        reset_image(image);
    }

    let mut control = match unsafe { create_control(image) } {
        Ok(control) => control,
        Err(_) => {
            let _ = unsafe { libc::fclose(file) };
            return unsafe { image_error_bytes(image, b"png_image_read: out of memory\0") };
        }
    };

    control.file = file;
    control.owned_file = true;
    unsafe {
        png_init_io(control.png_ptr, file);
        begin_read_with_control(image, control)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_begin_read_from_stdio(
    image: png_imagep,
    file: *mut FILE,
) -> c_int {
    if image.is_null() {
        return 0;
    }
    if unsafe { (*image).version } != PNG_IMAGE_VERSION {
        return unsafe {
            image_error_bytes(
                image,
                b"png_image_begin_read_from_stdio: incorrect PNG_IMAGE_VERSION\0",
            )
        };
    }
    if file.is_null() {
        return unsafe {
            image_error_bytes(image, b"png_image_begin_read_from_stdio: invalid argument\0")
        };
    }
    if unsafe { !(*image).opaque.is_null() } {
        return unsafe { image_error_bytes(image, b"png_image_read: opaque pointer not NULL\0") };
    }

    unsafe {
        reset_image(image);
    }

    let mut control = match unsafe { create_control(image) } {
        Ok(control) => control,
        Err(_) => return unsafe { image_error_bytes(image, b"png_image_read: out of memory\0") },
    };

    control.file = file;
    unsafe {
        png_init_io(control.png_ptr, file);
        begin_read_with_control(image, control)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_begin_read_from_memory(
    image: png_imagep,
    memory: png_const_voidp,
    size: usize,
) -> c_int {
    if image.is_null() {
        return 0;
    }
    if unsafe { (*image).version } != PNG_IMAGE_VERSION {
        return unsafe {
            image_error_bytes(
                image,
                b"png_image_begin_read_from_memory: incorrect PNG_IMAGE_VERSION\0",
            )
        };
    }
    if memory.is_null() || size == 0 {
        return unsafe {
            image_error_bytes(image, b"png_image_begin_read_from_memory: invalid argument\0")
        };
    }
    if unsafe { !(*image).opaque.is_null() } {
        return unsafe { image_error_bytes(image, b"png_image_read: opaque pointer not NULL\0") };
    }

    unsafe {
        reset_image(image);
    }

    let mut control = match unsafe { create_control(image) } {
        Ok(control) => control,
        Err(_) => return unsafe { image_error_bytes(image, b"png_image_read: out of memory\0") },
    };

    control.memory = Some(MemoryReader {
        memory: memory.cast(),
        size,
        offset: 0,
    });
    let reader = control.memory.as_mut().unwrap() as *mut MemoryReader;
    unsafe {
        png_set_read_fn(control.png_ptr, reader.cast(), Some(simplified_memory_read));
        begin_read_with_control(image, control)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_finish_read(
    image: png_imagep,
    background: png_const_colorp,
    buffer: png_voidp,
    mut row_stride: png_int_32,
    colormap: png_voidp,
) -> c_int {
    if image.is_null() {
        return 0;
    }
    if unsafe { (*image).version } != PNG_IMAGE_VERSION {
        return unsafe {
            image_error_bytes(
                image,
                b"png_image_finish_read: damaged PNG_IMAGE_VERSION\0",
            )
        };
    }

    let format = unsafe { (*image).format };
    let Some(png_row_stride) = pixel_row_stride(unsafe { (*image).width }, format) else {
        return unsafe {
            image_error_bytes(image, b"png_image_finish_read: row_stride too large\0")
        };
    };

    if row_stride == 0 {
        row_stride = png_row_stride as png_int_32;
    }

    let check = absolute_row_stride(row_stride);
    if unsafe { (*image).opaque.is_null() } || buffer.is_null() || check < png_row_stride {
        return unsafe { image_error_bytes(image, b"png_image_finish_read: invalid argument\0") };
    }

    if !colormap.is_null() || (format & PNG_FORMAT_FLAG_COLORMAP) != 0 {
        return unsafe { image_error_bytes(image, b"png_image_finish_read: invalid argument\0") };
    }

    if unsafe { (*image).height } > 0xffff_ffffu32 / check.max(1) {
        return unsafe { image_error_bytes(image, b"png_image_finish_read: image too large\0") };
    }

    let control = unsafe { &mut *(*image).opaque.cast::<SimplifiedReadControl>() };
    let result = unsafe { finish_read_impl(image, control, background, buffer, row_stride) };
    unsafe {
        image_free_internal(image);
    }
    if result { 1 } else { 0 }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_free(image: png_imagep) {
    unsafe {
        image_free_internal(image);
    }
}
