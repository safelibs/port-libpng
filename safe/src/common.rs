use crate::types::*;
use core::ffi::{c_char, c_int};
use core::ptr;

pub const PNG_LIBPNG_VER: png_uint_32 = 10643;
pub const PNG_UINT_31_MAX: png_uint_32 = 0x7fff_ffff;
pub const PNG_USER_WIDTH_MAX: png_uint_32 = 1_000_000;
pub const PNG_USER_HEIGHT_MAX: png_uint_32 = 1_000_000;
pub const PNG_USER_CHUNK_CACHE_MAX: png_uint_32 = 1_000;
pub const PNG_USER_CHUNK_MALLOC_MAX: png_alloc_size_t = 8_000_000;
pub const PNG_ZBUF_SIZE: png_uint = 8192;

pub const PNG_STRUCT_PNG: png_uint_32 = 0x0001;
pub const PNG_STRUCT_INFO: png_uint_32 = 0x0002;

pub const PNG_HAVE_PNG_SIGNATURE: png_uint_32 = 0x1000;
pub const PNG_IS_READ_STRUCT: png_uint_32 = 0x8000;

pub const PNG_USER_TRANSFORM: png_uint_32 = 0x100000;

pub const PNG_FLAG_ROW_INIT: png_uint_32 = 0x0040;
pub const PNG_FLAG_LIBRARY_MISMATCH: png_uint_32 = 0x20000;
pub const PNG_FLAG_BENIGN_ERRORS_WARN: png_uint_32 = 0x100000;
pub const PNG_FLAG_APP_WARNINGS_WARN: png_uint_32 = 0x200000;
pub const PNG_FLAG_APP_ERRORS_WARN: png_uint_32 = 0x400000;

pub const PNG_DESTROY_WILL_FREE_DATA: c_int = 1;
pub const PNG_USER_WILL_FREE_DATA: c_int = 2;

pub const PNG_FREE_HIST: png_uint_32 = 0x0008;
pub const PNG_FREE_ICCP: png_uint_32 = 0x0010;
pub const PNG_FREE_SPLT: png_uint_32 = 0x0020;
pub const PNG_FREE_ROWS: png_uint_32 = 0x0040;
pub const PNG_FREE_PCAL: png_uint_32 = 0x0080;
pub const PNG_FREE_SCAL: png_uint_32 = 0x0100;
pub const PNG_FREE_UNKN: png_uint_32 = 0x0200;
pub const PNG_FREE_PLTE: png_uint_32 = 0x1000;
pub const PNG_FREE_TRNS: png_uint_32 = 0x2000;
pub const PNG_FREE_TEXT: png_uint_32 = 0x4000;
pub const PNG_FREE_EXIF: png_uint_32 = 0x8000;
pub const PNG_FREE_ALL: png_uint_32 = 0xffff;
pub const PNG_FREE_MUL: png_uint_32 = 0x4220;

pub const PNG_INFO_gAMA: png_uint_32 = 0x0001;
pub const PNG_INFO_sBIT: png_uint_32 = 0x0002;
pub const PNG_INFO_cHRM: png_uint_32 = 0x0004;
pub const PNG_INFO_PLTE: png_uint_32 = 0x0008;
pub const PNG_INFO_tRNS: png_uint_32 = 0x0010;
pub const PNG_INFO_bKGD: png_uint_32 = 0x0020;
pub const PNG_INFO_hIST: png_uint_32 = 0x0040;
pub const PNG_INFO_pHYs: png_uint_32 = 0x0080;
pub const PNG_INFO_oFFs: png_uint_32 = 0x0100;
pub const PNG_INFO_tIME: png_uint_32 = 0x0200;
pub const PNG_INFO_pCAL: png_uint_32 = 0x0400;
pub const PNG_INFO_sRGB: png_uint_32 = 0x0800;
pub const PNG_INFO_iCCP: png_uint_32 = 0x1000;
pub const PNG_INFO_sPLT: png_uint_32 = 0x2000;
pub const PNG_INFO_sCAL: png_uint_32 = 0x4000;
pub const PNG_INFO_IDAT: png_uint_32 = 0x8000;
pub const PNG_INFO_eXIf: png_uint_32 = 0x10000;

pub const PNG_IO_READING: png_uint_32 = 0x0001;
pub const PNG_IO_WRITING: png_uint_32 = 0x0002;
pub const PNG_IO_SIGNATURE: png_uint_32 = 0x0010;
pub const PNG_IO_CHUNK_HDR: png_uint_32 = 0x0020;
pub const PNG_IO_CHUNK_DATA: png_uint_32 = 0x0040;
pub const PNG_IO_CHUNK_CRC: png_uint_32 = 0x0080;

pub const PNG_OPTION_NEXT: c_int = 14;
pub const PNG_OPTION_UNSET: c_int = 0;
pub const PNG_OPTION_INVALID: c_int = 1;
pub const PNG_OPTION_OFF: c_int = 2;
pub const PNG_OPTION_ON: c_int = 3;

pub const PNG_Z_DEFAULT_COMPRESSION: c_int = -1;
pub const PNG_Z_DEFAULT_STRATEGY: c_int = 1;
pub const PNG_TEXT_Z_DEFAULT_COMPRESSION: c_int = -1;
pub const PNG_TEXT_Z_DEFAULT_STRATEGY: c_int = 0;
pub const PNG_Z_DEFAULT_MEM_LEVEL: c_int = 8;
pub const PNG_Z_DEFAULT_WINDOW_BITS: c_int = 15;
pub const PNG_Z_DEFAULT_METHOD: c_int = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WriteZlibSettings {
    pub buffer_size: usize,
    pub level: c_int,
    pub mem_level: c_int,
    pub method: c_int,
    pub strategy: c_int,
    pub window_bits: c_int,
    pub text_level: c_int,
    pub text_mem_level: c_int,
    pub text_method: c_int,
    pub text_strategy: c_int,
    pub text_window_bits: c_int,
}

impl Default for WriteZlibSettings {
    fn default() -> Self {
        Self {
            buffer_size: PNG_ZBUF_SIZE as usize,
            level: PNG_Z_DEFAULT_COMPRESSION,
            mem_level: PNG_Z_DEFAULT_MEM_LEVEL,
            method: PNG_Z_DEFAULT_METHOD,
            strategy: PNG_Z_DEFAULT_STRATEGY,
            window_bits: PNG_Z_DEFAULT_WINDOW_BITS,
            text_level: PNG_TEXT_Z_DEFAULT_COMPRESSION,
            text_mem_level: PNG_Z_DEFAULT_MEM_LEVEL,
            text_method: PNG_Z_DEFAULT_METHOD,
            text_strategy: PNG_TEXT_Z_DEFAULT_STRATEGY,
            text_window_bits: PNG_Z_DEFAULT_WINDOW_BITS,
        }
    }
}

pub static PNG_LIBPNG_VER_STRING: &[u8] = b"1.6.43\0";
pub static PNG_HEADER_VERSION_STRING: &[u8] = b" libpng version 1.6.43\n\n\0";
pub static PNG_COPYRIGHT_STRING: &[u8] = b"libpng version 1.6.43\nCopyright (c) 2018-2024 Cosmin Truta\nCopyright (c) 1998-2002,2004,2006-2018 Glenn Randers-Pehrson\nCopyright (c) 1996-1997 Andreas Dilger\nCopyright (c) 1995-1996 Guy Eric Schalnat, Group 42, Inc.\n\0";

#[macro_export]
macro_rules! abi_guard {
    ($png_ptr:expr, $body:expr) => {{
        let png_ptr = $png_ptr;
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $body)) {
            Ok(value) => value,
            Err(_) => {
                if !png_ptr.is_null() {
                    unsafe { crate::error::panic_to_png_error(png_ptr) }
                }
                std::process::abort();
            }
        }
    }};
}

#[macro_export]
macro_rules! abi_guard_no_png {
    ($body:expr) => {{
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $body)) {
            Ok(value) => value,
            Err(_) => Default::default(),
        }
    }};
}

pub fn month_name(month: png_byte) -> Option<&'static [u8; 3]> {
    match month {
        1 => Some(b"Jan"),
        2 => Some(b"Feb"),
        3 => Some(b"Mar"),
        4 => Some(b"Apr"),
        5 => Some(b"May"),
        6 => Some(b"Jun"),
        7 => Some(b"Jul"),
        8 => Some(b"Aug"),
        9 => Some(b"Sep"),
        10 => Some(b"Oct"),
        11 => Some(b"Nov"),
        12 => Some(b"Dec"),
        _ => None,
    }
}

pub fn chunk_name_byte(chunk_name: png_uint_32, shift: u32) -> u8 {
    ((chunk_name >> shift) & 0xff) as u8
}

pub fn is_alpha_chunk_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphabetic()
}

pub fn build_chunk_message(chunk_name: png_uint_32, message: png_const_charp, out: &mut [c_char]) {
    let mut index = 0usize;
    for shift in [24u32, 16, 8, 0] {
        let byte = chunk_name_byte(chunk_name, shift);
        if is_alpha_chunk_name_byte(byte) {
            if index + 1 < out.len() {
                out[index] = byte as c_char;
                index += 1;
            }
        } else if index + 4 < out.len() {
            out[index] = b'[' as c_char;
            out[index + 1] = b"0123456789ABCDEF"[(byte >> 4) as usize] as c_char;
            out[index + 2] = b"0123456789ABCDEF"[(byte & 0x0f) as usize] as c_char;
            out[index + 3] = b']' as c_char;
            index += 4;
        }
    }

    if message.is_null() {
        out[index.min(out.len() - 1)] = 0;
        return;
    }

    if index + 2 < out.len() {
        out[index] = b':' as c_char;
        out[index + 1] = b' ' as c_char;
        index += 2;
    }

    unsafe {
        let mut src = message;
        while *src != 0 && index + 1 < out.len() {
            out[index] = *src;
            src = src.add(1);
            index += 1;
        }
        out[index.min(out.len() - 1)] = 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_sig_cmp(
    sig: png_const_bytep,
    start: usize,
    mut num_to_check: usize,
) -> c_int {
    crate::abi_guard_no_png!({
        static PNG_SIGNATURE: [png_byte; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

        if sig.is_null() || start > 7 || num_to_check < 1 {
            return -1;
        }

        if num_to_check > 8 {
            num_to_check = 8;
        }
        if start + num_to_check > 8 {
            num_to_check = 8 - start;
        }

        libc::memcmp(
            sig.add(start).cast(),
            PNG_SIGNATURE.as_ptr().add(start).cast(),
            num_to_check,
        ) as c_int
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_access_version_number() -> png_uint_32 {
    crate::abi_guard_no_png!(PNG_LIBPNG_VER)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_libpng_ver(_png_ptr: png_const_structrp) -> png_const_charp {
    crate::abi_guard_no_png!(PNG_LIBPNG_VER_STRING.as_ptr().cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_header_ver(_png_ptr: png_const_structrp) -> png_const_charp {
    crate::abi_guard_no_png!(PNG_LIBPNG_VER_STRING.as_ptr().cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_header_version(_png_ptr: png_const_structrp) -> png_const_charp {
    crate::abi_guard_no_png!(PNG_HEADER_VERSION_STRING.as_ptr().cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_copyright(_png_ptr: png_const_structrp) -> png_const_charp {
    crate::abi_guard_no_png!(PNG_COPYRIGHT_STRING.as_ptr().cast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_build_grayscale_palette(bit_depth: c_int, palette: png_colorp) {
    crate::abi_guard_no_png!({
        if palette.is_null() {
            return;
        }

        let (num_palette, color_inc) = match bit_depth {
            1 => (2usize, 0xffu32),
            2 => (4usize, 0x55u32),
            4 => (16usize, 0x11u32),
            8 => (256usize, 1u32),
            _ => (0usize, 0u32),
        };

        let mut value = 0u32;
        for index in 0..num_palette {
            let entry = palette.add(index);
            (*entry).red = (value & 0xff) as png_byte;
            (*entry).green = (value & 0xff) as png_byte;
            (*entry).blue = (value & 0xff) as png_byte;
            value += color_inc;
        }
    });
}

pub(crate) unsafe fn png_get_uint_32_internal(buf: png_const_bytep) -> png_uint_32 {
    crate::abi_guard_no_png!({
        if buf.is_null() {
            return 0;
        }

        ((png_uint_32::from(*buf)) << 24)
            | ((png_uint_32::from(*buf.add(1))) << 16)
            | ((png_uint_32::from(*buf.add(2))) << 8)
            | png_uint_32::from(*buf.add(3))
    })
}

unsafe fn png_save_uint_32_impl(buf: png_bytep, value: png_uint_32) {
    if buf.is_null() {
        return;
    }
    *buf = ((value >> 24) & 0xff) as png_byte;
    *buf.add(1) = ((value >> 16) & 0xff) as png_byte;
    *buf.add(2) = ((value >> 8) & 0xff) as png_byte;
    *buf.add(3) = (value & 0xff) as png_byte;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_save_uint_32(buf: png_bytep, value: png_uint_32) {
    crate::abi_guard_no_png!({
        png_save_uint_32_impl(buf, value);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_save_uint_16(buf: png_bytep, value: core::ffi::c_uint) {
    crate::abi_guard_no_png!({
        if buf.is_null() {
            return;
        }
        *buf = ((value >> 8) & 0xff) as png_byte;
        *buf.add(1) = (value & 0xff) as png_byte;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_save_int_32(buf: png_bytep, value: png_int_32) {
    crate::abi_guard_no_png!(png_save_uint_32_impl(buf, value as png_uint_32))
}

fn rfc1123_string(ptime: &png_time) -> Option<[u8; 29]> {
    if ptime.year > 9999
        || ptime.month == 0
        || ptime.month > 12
        || ptime.day == 0
        || ptime.day > 31
        || ptime.hour > 23
        || ptime.minute > 59
        || ptime.second > 60
    {
        return None;
    }

    let month = month_name(ptime.month)?;
    let string = format!(
        "{} {} {} {:02}:{:02}:{:02} +0000",
        ptime.day,
        core::str::from_utf8(month).ok()?,
        ptime.year,
        ptime.hour,
        ptime.minute,
        ptime.second
    );
    if string.len() >= 29 {
        return None;
    }

    let mut out = [0u8; 29];
    out[..string.len()].copy_from_slice(string.as_bytes());
    Some(out)
}

unsafe fn png_convert_from_struct_tm_impl(ptime: png_timep, ttime: *const libc::tm) {
    if ptime.is_null() || ttime.is_null() {
        return;
    }

    (*ptime).year = (1900 + (*ttime).tm_year) as png_uint_16;
    (*ptime).month = ((*ttime).tm_mon + 1) as png_byte;
    (*ptime).day = (*ttime).tm_mday as png_byte;
    (*ptime).hour = (*ttime).tm_hour as png_byte;
    (*ptime).minute = (*ttime).tm_min as png_byte;
    (*ptime).second = (*ttime).tm_sec as png_byte;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_convert_to_rfc1123_buffer(
    out: *mut c_char,
    ptime: png_const_timep,
) -> c_int {
    crate::abi_guard_no_png!({
        if out.is_null() || ptime.is_null() {
            return 0;
        }

        let Some(formatted) = rfc1123_string(&*ptime) else {
            return 0;
        };

        ptr::copy_nonoverlapping(formatted.as_ptr().cast::<c_char>(), out, formatted.len());
        1
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_convert_from_struct_tm(ptime: png_timep, ttime: *const libc::tm) {
    crate::abi_guard_no_png!(png_convert_from_struct_tm_impl(ptime, ttime));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_convert_from_time_t(ptime: png_timep, ttime: libc::time_t) {
    crate::abi_guard_no_png!({
        if ptime.is_null() {
            return;
        }

        let mut out: libc::tm = core::mem::zeroed();
        if libc::gmtime_r(&ttime, &mut out).is_null() {
            core::ptr::write_bytes(ptime, 0, 1);
            return;
        }

        png_convert_from_struct_tm_impl(ptime, &out);
    })
}
