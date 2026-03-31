#![allow(dead_code)]

use core::ffi::{c_char, c_double, c_int, c_uchar, c_uint, c_void};
use libc::{FILE, time_t, tm};

pub type png_voidp = *mut c_void;
pub type png_const_voidp = *const c_void;
pub type png_charp = *mut c_char;
pub type png_const_charp = *const c_char;
pub type png_bytep = *mut png_byte;
pub type png_const_bytep = *const png_byte;
pub type png_bytepp = *mut png_bytep;
pub type png_charpp = *mut png_charp;
pub type png_uint_16p = *mut png_uint_16;
pub type png_const_uint_16p = *const png_uint_16;

pub type png_byte = u8;
pub type png_uint_16 = u16;
pub type png_uint_32 = u32;
pub type png_int_32 = i32;
pub type png_fixed_point = png_int_32;
pub type png_size_t = usize;
pub type png_alloc_size_t = usize;

#[repr(C)]
pub struct png_struct {
    _private: [u8; 0],
}

#[repr(C)]
pub struct png_info {
    _private: [u8; 0],
}

#[repr(C)]
pub struct JmpBuf {
    _private: [u8; 0],
}

pub type png_jmpbufp = *mut JmpBuf;

pub type png_structp = *mut png_struct;
pub type png_const_structp = *const png_struct;
pub type png_structpp = *mut png_structp;
pub type png_structrp = png_structp;
pub type png_const_structrp = png_const_structp;

pub type png_infop = *mut png_info;
pub type png_const_infop = *const png_info;
pub type png_infopp = *mut png_infop;
pub type png_inforp = png_infop;
pub type png_const_inforp = png_const_infop;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct png_color {
    pub red: png_byte,
    pub green: png_byte,
    pub blue: png_byte,
}

pub type png_colorp = *mut png_color;
pub type png_const_colorp = *const png_color;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct png_color_16 {
    pub index: png_byte,
    pub red: png_uint_16,
    pub green: png_uint_16,
    pub blue: png_uint_16,
    pub gray: png_uint_16,
}

pub type png_color_16p = *mut png_color_16;
pub type png_const_color_16p = *const png_color_16;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct png_color_8 {
    pub red: png_byte,
    pub green: png_byte,
    pub blue: png_byte,
    pub gray: png_byte,
    pub alpha: png_byte,
}

pub type png_color_8p = *mut png_color_8;
pub type png_const_color_8p = *const png_color_8;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct png_xy {
    pub redx: png_fixed_point,
    pub redy: png_fixed_point,
    pub greenx: png_fixed_point,
    pub greeny: png_fixed_point,
    pub bluex: png_fixed_point,
    pub bluey: png_fixed_point,
    pub whitex: png_fixed_point,
    pub whitey: png_fixed_point,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct png_XYZ {
    pub red_X: png_fixed_point,
    pub red_Y: png_fixed_point,
    pub red_Z: png_fixed_point,
    pub green_X: png_fixed_point,
    pub green_Y: png_fixed_point,
    pub green_Z: png_fixed_point,
    pub blue_X: png_fixed_point,
    pub blue_Y: png_fixed_point,
    pub blue_Z: png_fixed_point,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct png_colorspace {
    pub gamma: png_fixed_point,
    pub end_points_xy: png_xy,
    pub end_points_XYZ: png_XYZ,
    pub rendering_intent: png_uint_16,
    pub flags: png_uint_16,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct png_sPLT_entry {
    pub red: png_uint_16,
    pub green: png_uint_16,
    pub blue: png_uint_16,
    pub alpha: png_uint_16,
    pub frequency: png_uint_16,
}

pub type png_sPLT_entryp = *mut png_sPLT_entry;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct png_sPLT_t {
    pub name: png_charp,
    pub depth: png_byte,
    pub entries: png_sPLT_entryp,
    pub nentries: png_int_32,
}

pub type png_sPLT_tp = *mut png_sPLT_t;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct png_text {
    pub compression: c_int,
    pub key: png_charp,
    pub text: png_charp,
    pub text_length: usize,
    pub itxt_length: usize,
    pub lang: png_charp,
    pub lang_key: png_charp,
}

pub type png_textp = *mut png_text;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct png_time {
    pub year: png_uint_16,
    pub month: png_byte,
    pub day: png_byte,
    pub hour: png_byte,
    pub minute: png_byte,
    pub second: png_byte,
}

pub type png_timep = *mut png_time;
pub type png_const_timep = *const png_time;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct png_unknown_chunk {
    pub name: [png_byte; 5],
    pub data: png_bytep,
    pub size: usize,
    pub location: png_byte,
}

impl Default for png_unknown_chunk {
    fn default() -> Self {
        Self {
            name: [0; 5],
            data: core::ptr::null_mut(),
            size: 0,
            location: 0,
        }
    }
}

pub type png_unknown_chunkp = *mut png_unknown_chunk;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct png_row_info {
    pub width: png_uint_32,
    pub rowbytes: usize,
    pub color_type: png_byte,
    pub bit_depth: png_byte,
    pub channels: png_byte,
    pub pixel_depth: png_byte,
}

pub type png_row_infop = *mut png_row_info;

pub type png_error_ptr = Option<unsafe extern "C" fn(png_structp, png_const_charp)>;
pub type png_rw_ptr = Option<unsafe extern "C" fn(png_structp, png_bytep, usize)>;
pub type png_flush_ptr = Option<unsafe extern "C" fn(png_structp)>;
pub type png_read_status_ptr = Option<unsafe extern "C" fn(png_structp, png_uint_32, c_int)>;
pub type png_write_status_ptr = Option<unsafe extern "C" fn(png_structp, png_uint_32, c_int)>;
pub type png_progressive_info_ptr = Option<unsafe extern "C" fn(png_structp, png_infop)>;
pub type png_progressive_end_ptr = Option<unsafe extern "C" fn(png_structp, png_infop)>;
pub type png_progressive_row_ptr =
    Option<unsafe extern "C" fn(png_structp, png_bytep, png_uint_32, c_int)>;
pub type png_user_transform_ptr =
    Option<unsafe extern "C" fn(png_structp, png_row_infop, png_bytep)>;
pub type png_user_chunk_ptr =
    Option<unsafe extern "C" fn(png_structp, png_unknown_chunkp) -> c_int>;
pub type png_longjmp_ptr = Option<unsafe extern "C" fn(png_jmpbufp, c_int)>;
pub type png_malloc_ptr = Option<unsafe extern "C" fn(png_structp, png_alloc_size_t) -> png_voidp>;
pub type png_free_ptr = Option<unsafe extern "C" fn(png_structp, png_voidp)>;

pub type png_FILE_p = *mut FILE;
pub type png_tm = tm;
pub type png_time_t = time_t;
pub type png_uchar = c_uchar;
pub type png_uint = c_uint;
pub type png_controlp = *mut c_void;
pub type png_doublep = *mut c_double;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct png_safe_read_core {
    pub mode: png_uint_32,
    pub flags: png_uint_32,
    pub transformations: png_uint_32,
    pub width: png_uint_32,
    pub height: png_uint_32,
    pub num_rows: png_uint_32,
    pub row_number: png_uint_32,
    pub chunk_name: png_uint_32,
    pub idat_size: png_uint_32,
    pub rowbytes: usize,
    pub info_rowbytes: usize,
    pub save_buffer_size: usize,
    pub buffer_size: usize,
    pub current_buffer_size: usize,
    pub interlaced: png_byte,
    pub color_type: png_byte,
    pub bit_depth: png_byte,
    pub pixel_depth: png_byte,
    pub transformed_pixel_depth: png_byte,
    pub channels: png_byte,
    pub compression_type: png_byte,
    pub filter_type: png_byte,
    pub background_gamma_type: png_byte,
    pub background_gamma: png_fixed_point,
    pub screen_gamma: png_fixed_point,
    pub background: png_color_16,
    pub shift: png_color_8,
    pub colorspace: png_colorspace,
    pub rgb_to_gray_status: png_byte,
    pub rgb_to_gray_coefficients_set: png_byte,
    pub rgb_to_gray_red_coeff: png_uint_16,
    pub rgb_to_gray_green_coeff: png_uint_16,
    pub pass: c_int,
    pub process_mode: c_int,
    pub num_palette_max: c_int,
    pub unknown_default: c_int,
    pub num_chunk_list: png_uint_32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct png_safe_info_core {
    pub width: png_uint_32,
    pub height: png_uint_32,
    pub valid: png_uint_32,
    pub rowbytes: usize,
    pub num_palette: png_uint_16,
    pub num_trans: png_uint_16,
    pub bit_depth: png_byte,
    pub color_type: png_byte,
    pub compression_type: png_byte,
    pub filter_type: png_byte,
    pub interlace_type: png_byte,
    pub channels: png_byte,
    pub pixel_depth: png_byte,
    pub background: png_color_16,
    pub sig_bit: png_color_8,
    pub trans_color: png_color_16,
    pub colorspace: png_colorspace,
    pub row_pointers: png_bytepp,
    pub free_me: png_uint_32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct png_image {
    pub opaque: png_controlp,
    pub version: png_uint_32,
    pub width: png_uint_32,
    pub height: png_uint_32,
    pub format: png_uint_32,
    pub flags: png_uint_32,
    pub colormap_entries: png_uint_32,
    pub warning_or_error: png_uint_32,
    pub message: [c_char; 64],
}

pub type png_imagep = *mut png_image;
pub type png_const_imagep = *const png_image;
