use crate::chunks::{read_core, read_info_core};
use crate::read::png_destroy_read_struct;
use crate::read_transform::{
    png_set_expand, png_set_expand_16, png_set_gray_to_rgb, png_set_scale_16,
};
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
const PNG_FORMAT_FLAG_ASSOCIATED_ALPHA: png_uint_32 = 0x40;

const PNG_COLOR_MASK_PALETTE: png_byte = 1;
const PNG_COLOR_MASK_COLOR: png_byte = 2;
const PNG_COLOR_MASK_ALPHA: png_byte = 4;

const PNG_COLORSPACE_HAVE_ENDPOINTS: png_uint_16 = 0x0002;
const PNG_COLORSPACE_ENDPOINTS_MATCH_sRGB: png_uint_16 = 0x0040;
const PNG_COLORSPACE_INVALID: png_uint_16 = 0x8000;

const MAX_SAMPLE_BYTES: usize = 8;
const RGB_TO_GRAY_RED_COEFF: u32 = 6968;
const RGB_TO_GRAY_GREEN_COEFF: u32 = 23_434;
const RGB_TO_GRAY_BLUE_COEFF: u32 = 2366;

#[derive(Clone, Copy, Debug)]
struct FormatSpec {
    format: png_uint_32,
}

#[derive(Clone, Copy, Debug)]
struct Pixel16 {
    red: u16,
    green: u16,
    blue: u16,
    alpha: u16,
}

impl FormatSpec {
    fn parse(format: png_uint_32) -> Option<Self> {
        let supported = PNG_FORMAT_FLAG_ALPHA
            | PNG_FORMAT_FLAG_COLOR
            | PNG_FORMAT_FLAG_LINEAR
            | PNG_FORMAT_FLAG_COLORMAP
            | PNG_FORMAT_FLAG_BGR
            | PNG_FORMAT_FLAG_AFIRST
            | PNG_FORMAT_FLAG_ASSOCIATED_ALPHA;
        if (format & !supported) != 0 {
            return None;
        }

        Some(Self { format })
    }

    fn alpha(self) -> bool {
        (self.format & PNG_FORMAT_FLAG_ALPHA) != 0
    }

    fn color(self) -> bool {
        (self.format & PNG_FORMAT_FLAG_COLOR) != 0
    }

    fn linear(self) -> bool {
        (self.format & PNG_FORMAT_FLAG_LINEAR) != 0
    }

    fn colormap(self) -> bool {
        (self.format & PNG_FORMAT_FLAG_COLORMAP) != 0
    }

    fn bgr(self) -> bool {
        self.color() && (self.format & PNG_FORMAT_FLAG_BGR) != 0
    }

    fn afirst(self) -> bool {
        self.alpha() && (self.format & PNG_FORMAT_FLAG_AFIRST) != 0
    }

    fn associated_alpha(self) -> bool {
        self.alpha() && (self.format & PNG_FORMAT_FLAG_ASSOCIATED_ALPHA) != 0
    }

    fn sample_channels(self) -> usize {
        usize::try_from((self.format & (PNG_FORMAT_FLAG_COLOR | PNG_FORMAT_FLAG_ALPHA)) + 1)
            .unwrap_or(0)
    }

    fn sample_component_size(self) -> usize {
        usize::try_from(((self.format & PNG_FORMAT_FLAG_LINEAR) >> 2) + 1).unwrap_or(0)
    }

    fn sample_size(self) -> usize {
        self.sample_channels() * self.sample_component_size()
    }

    fn pixel_channels(self) -> usize {
        if self.colormap() {
            1
        } else {
            self.sample_channels()
        }
    }

    fn pixel_component_size(self) -> usize {
        if self.colormap() {
            1
        } else {
            self.sample_component_size()
        }
    }

    fn pixel_row_stride(self, width: png_uint_32) -> Option<png_uint_32> {
        width.checked_mul(self.pixel_channels() as png_uint_32)
    }

    fn row_bytes(self, stride_units: usize) -> Option<usize> {
        stride_units.checked_mul(self.pixel_component_size())
    }
}

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

fn expand_u8(value: u8) -> u16 {
    u16::from(value) * 257
}

fn reduce_u16(value: u16) -> u8 {
    ((u32::from(value) + 128) / 257) as u8
}

fn premultiply_u16(channel: u16, alpha: u16) -> u16 {
    (((u32::from(channel) * u32::from(alpha)) + 32_767) / 65_535) as u16
}

fn composite_u16(channel: u16, alpha: u16, background: u16) -> u16 {
    (((u32::from(channel) * u32::from(alpha))
        + (u32::from(background) * u32::from(65_535 - alpha))
        + 32_767)
        / 65_535) as u16
}

fn grayscale_u16(pixel: Pixel16) -> u16 {
    let gray = u64::from(pixel.red) * u64::from(RGB_TO_GRAY_RED_COEFF)
        + u64::from(pixel.green) * u64::from(RGB_TO_GRAY_GREEN_COEFF)
        + u64::from(pixel.blue) * u64::from(RGB_TO_GRAY_BLUE_COEFF)
        + 16_384;
    (gray >> 15) as u16
}

fn read_decoded_pixel(
    row: &[u8],
    x: usize,
    decoded_channels: usize,
    component_size: usize,
) -> Option<Pixel16> {
    let pixel_bytes = decoded_channels.checked_mul(component_size)?;
    let start = x.checked_mul(pixel_bytes)?;
    let src = row.get(start..start.checked_add(pixel_bytes)?)?;

    let read_component = |component_index: usize| -> Option<u16> {
        let offset = component_index.checked_mul(component_size)?;
        let bytes = src.get(offset..offset.checked_add(component_size)?)?;
        Some(if component_size == 2 {
            u16::from_be_bytes([bytes[0], bytes[1]])
        } else {
            expand_u8(bytes[0])
        })
    };

    let red = read_component(0)?;
    let green = read_component(1)?;
    let blue = read_component(2)?;
    let alpha = if decoded_channels == 4 {
        read_component(3)?
    } else {
        65_535
    };

    Some(Pixel16 {
        red,
        green,
        blue,
        alpha,
    })
}

fn encode_component(dst: &mut [u8], offset: usize, value: u16, linear: bool) {
    if linear {
        dst[offset..offset + 2].copy_from_slice(&value.to_ne_bytes());
    } else {
        dst[offset] = reduce_u16(value);
    }
}

fn write_sample_bytes(
    spec: FormatSpec,
    pixel: Pixel16,
    background: Option<[u16; 3]>,
    dst: &mut [u8],
) {
    let mut red = pixel.red;
    let mut green = pixel.green;
    let mut blue = pixel.blue;
    let alpha = pixel.alpha;

    if spec.alpha() {
        if spec.associated_alpha() {
            red = premultiply_u16(red, alpha);
            green = premultiply_u16(green, alpha);
            blue = premultiply_u16(blue, alpha);
        }
    } else if alpha < 65_535 {
        let background = if spec.linear() {
            [0, 0, 0]
        } else {
            background.unwrap_or([0, 0, 0])
        };
        red = composite_u16(red, alpha, background[0]);
        green = composite_u16(green, alpha, background[1]);
        blue = composite_u16(blue, alpha, background[2]);
    }

    if spec.color() {
        let ordered = if spec.bgr() {
            [blue, green, red]
        } else {
            [red, green, blue]
        };
        let component_size = spec.sample_component_size();

        if spec.alpha() {
            if spec.afirst() {
                encode_component(dst, 0, alpha, spec.linear());
                encode_component(dst, component_size, ordered[0], spec.linear());
                encode_component(dst, component_size * 2, ordered[1], spec.linear());
                encode_component(dst, component_size * 3, ordered[2], spec.linear());
            } else {
                encode_component(dst, 0, ordered[0], spec.linear());
                encode_component(dst, component_size, ordered[1], spec.linear());
                encode_component(dst, component_size * 2, ordered[2], spec.linear());
                encode_component(dst, component_size * 3, alpha, spec.linear());
            }
        } else {
            encode_component(dst, 0, ordered[0], spec.linear());
            encode_component(dst, component_size, ordered[1], spec.linear());
            encode_component(dst, component_size * 2, ordered[2], spec.linear());
        }
    } else {
        let gray = grayscale_u16(Pixel16 {
            red,
            green,
            blue,
            alpha,
        });
        let component_size = spec.sample_component_size();

        if spec.alpha() {
            if spec.afirst() {
                encode_component(dst, 0, alpha, spec.linear());
                encode_component(dst, component_size, gray, spec.linear());
            } else {
                encode_component(dst, 0, gray, spec.linear());
                encode_component(dst, component_size, alpha, spec.linear());
            }
        } else {
            encode_component(dst, 0, gray, spec.linear());
        }
    }
}

unsafe fn finish_read_impl(
    image: png_imagep,
    control: &mut SimplifiedReadControl,
    background: png_const_colorp,
    buffer: png_voidp,
    colormap: png_voidp,
    row_stride: png_int_32,
) -> bool {
    let Some(spec) = FormatSpec::parse(unsafe { (*image).format }) else {
        unsafe {
            image_error_bytes(image, b"png_image_finish_read: invalid argument\0");
        }
        return false;
    };

    let source = unsafe { read_core(control.png_ptr) };
    let source_info = unsafe { read_info_core(control.info_ptr) };
    let source_has_alpha =
        (source.color_type & PNG_COLOR_MASK_ALPHA) != 0 || source_info.num_trans > 0;

    if spec.colormap()
        && !spec.linear()
        && !spec.alpha()
        && source_has_alpha
        && background.is_null()
    {
        unsafe {
            image_error_bytes(image, b"png_image_finish_read: invalid argument\0");
        }
        return false;
    }

    unsafe {
        if spec.linear() {
            png_set_expand_16(control.png_ptr);
        } else {
            png_set_expand(control.png_ptr);
        }
        png_set_gray_to_rgb(control.png_ptr);
        if !spec.linear() && source.bit_depth == 16 {
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
    let decoded_bit_depth = if updated_info.bit_depth != 0 {
        updated_info.bit_depth
    } else {
        updated.bit_depth
    };
    let decoded_component_size = if decoded_bit_depth == 16 {
        2
    } else {
        1
    };
    if decoded_channels != 3 && decoded_channels != 4 {
        unsafe {
            image_error_bytes(image, b"png_image_finish_read: invalid argument\0");
        }
        return false;
    }

    let stride_units = absolute_row_stride(row_stride) as usize;
    let Some(stride_bytes) = spec.row_bytes(stride_units) else {
        unsafe {
            image_error_bytes(image, b"png_image_finish_read: image too large\0");
        }
        return false;
    };
    let buffer = buffer.cast::<u8>();
    let background_16 = if background.is_null() {
        None
    } else {
        Some(unsafe {
            [
                expand_u8((*background).red),
                expand_u8((*background).green),
                expand_u8((*background).blue),
            ]
        })
    };
    let source_rows = decoded.as_slice();
    let width = unsafe { (*image).width as usize };
    let entry_size = spec.sample_size();

    let mut colormap_entries = Vec::<Vec<u8>>::new();
    let colormap_capacity = if spec.colormap() {
        usize::try_from(unsafe { (*image).colormap_entries })
            .unwrap_or(0)
            .min(256)
    } else {
        0
    };

    for y in 0..height {
        let src_row = &source_rows[y * rowbytes..(y + 1) * rowbytes];
        let dst_y = if row_stride < 0 { height - 1 - y } else { y };
        let dst_row =
            unsafe { slice::from_raw_parts_mut(buffer.add(dst_y * stride_bytes), stride_bytes) };

        for x in 0..width {
            let Some(pixel) = read_decoded_pixel(src_row, x, decoded_channels, decoded_component_size)
            else {
                unsafe {
                    image_error_bytes(image, b"png_image_finish_read: invalid argument\0");
                }
                return false;
            };

            if spec.colormap() {
                let mut entry = [0u8; MAX_SAMPLE_BYTES];
                write_sample_bytes(spec, pixel, background_16, &mut entry[..entry_size]);
                let entry_slice = &entry[..entry_size];
                let entry_index = if let Some(index) = colormap_entries
                    .iter()
                    .position(|existing| existing.as_slice() == entry_slice)
                {
                    index
                } else {
                    if colormap_entries.len() >= colormap_capacity {
                        unsafe {
                            image_error_bytes(image, b"png_image_finish_read: invalid argument\0");
                        }
                        return false;
                    }
                    colormap_entries.push(entry_slice.to_vec());
                    colormap_entries.len() - 1
                };
                dst_row[x] = entry_index as u8;
            } else {
                let pixel_size = spec.sample_size();
                let start = x * pixel_size;
                let end = start + pixel_size;
                write_sample_bytes(spec, pixel, background_16, &mut dst_row[start..end]);
            }
        }
    }

    if spec.colormap() {
        let colormap = colormap.cast::<u8>();
        if colormap.is_null() {
            unsafe {
                image_error_bytes(image, b"png_image_finish_read[color-map]: no color-map\0");
            }
            return false;
        }

        for (index, entry) in colormap_entries.iter().enumerate() {
            unsafe {
                ptr::copy_nonoverlapping(
                    entry.as_ptr(),
                    colormap.add(index * entry_size),
                    entry_size,
                );
            }
        }
        unsafe {
            (*image).colormap_entries = colormap_entries.len() as png_uint_32;
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
    let Some(spec) = FormatSpec::parse(format) else {
        return unsafe { image_error_bytes(image, b"png_image_finish_read: invalid argument\0") };
    };
    let Some(png_row_stride) = spec.pixel_row_stride(unsafe { (*image).width }) else {
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

    if spec.colormap() && (unsafe { (*image).colormap_entries } == 0 || colormap.is_null()) {
        return unsafe {
            image_error_bytes(image, b"png_image_finish_read[color-map]: no color-map\0")
        };
    }

    let Some(component_check) = usize::try_from(check)
        .ok()
        .and_then(|stride| spec.row_bytes(stride))
    else {
        return unsafe { image_error_bytes(image, b"png_image_finish_read: image too large\0") };
    };

    if unsafe { (*image).height } > 0xffff_ffffu32 / (component_check.max(1) as png_uint_32) {
        return unsafe { image_error_bytes(image, b"png_image_finish_read: image too large\0") };
    }

    let control = unsafe { &mut *(*image).opaque.cast::<SimplifiedReadControl>() };
    let result = unsafe { finish_read_impl(image, control, background, buffer, colormap, row_stride) };
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
