use crate::common::WriteZlibSettings;
use crate::read_util::checked_decompressed_len;
use crate::state;
use crate::types::*;
use core::ffi::{c_char, c_int, c_uint, c_ulong, c_void};

const Z_NO_FLUSH: c_int = 0;
const Z_FINISH: c_int = 4;
const Z_OK: c_int = 0;
const Z_STREAM_END: c_int = 1;
const Z_BUF_ERROR: c_int = -5;

#[repr(C)]
struct ZStream {
    next_in: *mut u8,
    avail_in: c_uint,
    total_in: c_ulong,
    next_out: *mut u8,
    avail_out: c_uint,
    total_out: c_ulong,
    msg: *const c_char,
    state: *mut c_void,
    zalloc: Option<unsafe extern "C" fn(*mut c_void, c_uint, c_uint) -> *mut c_void>,
    zfree: Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>,
    opaque: *mut c_void,
    data_type: c_int,
    adler: c_ulong,
    reserved: c_ulong,
}

unsafe extern "C" {
    fn zlibVersion() -> *const c_char;
    fn inflateInit_(stream: *mut ZStream, version: *const c_char, stream_size: c_int) -> c_int;
    fn inflate(stream: *mut ZStream, flush: c_int) -> c_int;
    fn inflateEnd(stream: *mut ZStream) -> c_int;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct AncillaryInflateState {
    pub declared_bytes: usize,
    pub requested_bytes: usize,
    pub malloc_limit: png_alloc_size_t,
}

impl AncillaryInflateState {
    pub(crate) fn validate(self) -> Result<(), &'static [u8]> {
        validate_ancillary_allocation_limit(
            self.declared_bytes,
            self.requested_bytes,
            self.malloc_limit,
        )
    }
}

pub(crate) fn validate_ancillary_allocation_limit(
    declared_bytes: usize,
    requested_bytes: usize,
    malloc_limit: png_alloc_size_t,
) -> Result<(), &'static [u8]> {
    let Some(total) = checked_decompressed_len(declared_bytes, requested_bytes) else {
        return Err(&b"ancillary size overflow\0"[..]);
    };

    if malloc_limit != 0 && total > malloc_limit {
        return Err(&b"chunk data is too large\0"[..]);
    }

    Ok(())
}

fn inflate_limit(png_ptr: png_const_structrp) -> png_alloc_size_t {
    state::get_png(png_ptr.cast_mut())
        .map(|png_state| png_state.user_chunk_malloc_max)
        .unwrap_or(0)
}

pub(crate) fn write_zlib_settings(png_ptr: png_const_structrp) -> Option<WriteZlibSettings> {
    state::get_png(png_ptr.cast_mut()).map(|png_state| png_state.write_zlib)
}

pub(crate) fn update_write_zlib_settings(
    png_ptr: png_structrp,
    update: impl FnOnce(&mut WriteZlibSettings),
) {
    state::update_png(png_ptr, |png_state| {
        update(&mut png_state.write_zlib);
    });
}

fn push_output_with_limit(
    output: &mut Vec<u8>,
    bytes: &[u8],
    limit: png_alloc_size_t,
) -> Result<(), &'static [u8]> {
    let next_len = output
        .len()
        .checked_add(bytes.len())
        .ok_or(b"ancillary size overflow\0".as_slice())?;
    if limit != 0 && next_len > limit {
        return Err(b"chunk data is too large\0".as_slice());
    }

    output.extend_from_slice(bytes);
    Ok(())
}

pub(crate) fn inflate_ancillary_zlib(
    png_ptr: png_const_structrp,
    input: &[u8],
    declared_bytes: Option<usize>,
    terminate: bool,
) -> Result<Vec<u8>, &'static [u8]> {
    let limit = inflate_limit(png_ptr);
    let mut stream = unsafe { core::mem::zeroed::<ZStream>() };

    let version = unsafe { zlibVersion() };
    let init = unsafe {
        inflateInit_(
            &mut stream,
            version,
            i32::try_from(core::mem::size_of::<ZStream>()).unwrap_or(i32::MAX),
        )
    };
    if init != Z_OK {
        return Err(b"bad compressed data\0".as_slice());
    }

    stream.next_in = input.as_ptr().cast_mut();
    stream.avail_in = c_uint::try_from(input.len()).unwrap_or(c_uint::MAX);

    let mut output = Vec::new();
    let mut chunk = [0u8; 4096];
    let mut result = Err(b"bad compressed data\0".as_slice());

    loop {
        stream.next_out = chunk.as_mut_ptr();
        stream.avail_out = c_uint::try_from(chunk.len()).unwrap_or(c_uint::MAX);

        let ret = unsafe {
            inflate(
                &mut stream,
                if stream.avail_in == 0 {
                    Z_FINISH
                } else {
                    Z_NO_FLUSH
                },
            )
        };
        let produced = chunk.len() - usize::try_from(stream.avail_out).unwrap_or(0);
        if produced != 0 {
            if let Err(message) = push_output_with_limit(&mut output, &chunk[..produced], limit) {
                unsafe {
                    let _ = inflateEnd(&mut stream);
                }
                return Err(message);
            }
        }

        if ret == Z_STREAM_END {
            result = Ok(());
            break;
        }

        if ret == Z_OK || (ret == Z_BUF_ERROR && stream.avail_in != 0) {
            continue;
        }

        break;
    }

    unsafe {
        let _ = inflateEnd(&mut stream);
    }

    result?;
    if let Some(declared) = declared_bytes {
        validate_ancillary_allocation_limit(declared, output.len(), limit)?;
    }

    if terminate {
        if limit != 0 && output.len().saturating_add(1) > limit {
            return Err(b"chunk data is too large\0".as_slice());
        }
        output.push(0);
    }

    Ok(output)
}
