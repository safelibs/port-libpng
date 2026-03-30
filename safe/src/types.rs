use core::ffi::{c_char, c_int, c_uchar, c_uint, c_void};
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
pub type png_longjmp_ptr = Option<unsafe extern "C" fn(*mut JmpBuf, c_int)>;
pub type png_malloc_ptr = Option<unsafe extern "C" fn(png_structp, png_alloc_size_t) -> png_voidp>;
pub type png_free_ptr = Option<unsafe extern "C" fn(png_structp, png_voidp)>;

pub type png_FILE_p = *mut FILE;
pub type png_tm = tm;
pub type png_time_t = time_t;
pub type png_uchar = c_uchar;
pub type png_uint = c_uint;
