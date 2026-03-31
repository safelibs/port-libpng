use crate::common::{PNG_FLAG_ROW_INIT, PNG_HAVE_PNG_SIGNATURE};
use crate::types::*;
use core::ffi::c_int;

pub(crate) type KeepSymbol = core::sync::atomic::AtomicPtr<()>;

pub(crate) const PNG_HAVE_IEND: png_uint_32 = 0x10;
pub(crate) const PNG_HAVE_CHUNK_HEADER: png_uint_32 = 0x100;

pub(crate) const PNG_HANDLE_CHUNK_AS_DEFAULT: c_int = 0;
pub(crate) const PNG_HANDLE_CHUNK_NEVER: c_int = 1;
pub(crate) const PNG_HANDLE_CHUNK_IF_SAFE: c_int = 2;
pub(crate) const PNG_HANDLE_CHUNK_ALWAYS: c_int = 3;
pub(crate) const PNG_HANDLE_CHUNK_LAST: c_int = 4;

const PNG_READ_SIG_MODE: c_int = 0;
const PNG_READ_CHUNK_MODE: c_int = 1;
const PNG_READ_IDAT_MODE: c_int = 2;
const PNG_READ_TEXT_MODE: c_int = 4;
const PNG_READ_ZTXT_MODE: c_int = 5;
const PNG_READ_DONE_MODE: c_int = 6;
const PNG_READ_ITXT_MODE: c_int = 7;
const PNG_ERROR_MODE: c_int = 8;

const PNG_IDAT: png_uint_32 = 0x4944_4154;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum ReadPhase {
    #[default]
    Signature,
    ChunkHeader,
    ChunkPayload,
    IdatStream,
    ImageRows,
    Terminal,
}

impl ReadPhase {
    pub(crate) fn from_core(core: &png_safe_read_core) -> Self {
        if (core.mode & PNG_HAVE_IEND) != 0 || core.process_mode == PNG_READ_DONE_MODE {
            return Self::Terminal;
        }

        if (core.flags & PNG_FLAG_ROW_INIT) != 0 {
            return Self::ImageRows;
        }

        match core.process_mode {
            PNG_READ_SIG_MODE => Self::Signature,
            PNG_READ_CHUNK_MODE => {
                if (core.mode & PNG_HAVE_CHUNK_HEADER) != 0 {
                    Self::ChunkPayload
                } else {
                    Self::ChunkHeader
                }
            }
            PNG_READ_IDAT_MODE => Self::IdatStream,
            PNG_READ_TEXT_MODE | PNG_READ_ZTXT_MODE | PNG_READ_ITXT_MODE => Self::ChunkPayload,
            PNG_ERROR_MODE => Self::Terminal,
            _ => {
                if (core.mode & PNG_HAVE_PNG_SIGNATURE) == 0 {
                    Self::Signature
                } else if core.chunk_name == PNG_IDAT || core.idat_size != 0 {
                    Self::IdatStream
                } else {
                    Self::ChunkHeader
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ProgressiveReadState {
    pub last_pause_bytes: usize,
    pub last_skip_bytes: png_uint_32,
    pub paused_with_save: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnknownChunkSetting {
    pub name: [png_byte; 4],
    pub keep: png_byte,
}

impl UnknownChunkSetting {
    pub(crate) const fn new(name: [png_byte; 4], keep: png_byte) -> Self {
        Self { name, keep }
    }
}

pub(crate) fn checked_chunk_length(length: png_uint_32) -> Option<usize> {
    usize::try_from(length).ok()
}

pub(crate) fn checked_row_factor(rowbytes: usize, rows: png_uint_32) -> Option<usize> {
    rowbytes.checked_mul(usize::try_from(rows).ok()?)
}

pub(crate) fn checked_rowbytes_for_width(width: usize, pixel_depth: usize) -> Option<usize> {
    if pixel_depth == 0 {
        return None;
    }

    width
        .checked_mul(pixel_depth)?
        .checked_add(7)?
        .checked_div(8)
}

pub(crate) fn checked_decompressed_len(
    declared: png_alloc_size_t,
    extra: usize,
) -> Option<png_alloc_size_t> {
    declared.checked_add(extra)
}

pub(crate) fn infer_pixel_depth(core: &png_safe_read_core, rowbytes: usize) -> Option<usize> {
    const CANDIDATES: [usize; 9] = [1, 2, 4, 8, 16, 24, 32, 48, 64];

    let width = usize::try_from(core.width).ok()?;
    let transformed = usize::from(core.transformed_pixel_depth);
    if transformed != 0 {
        return Some(transformed);
    }

    let derived = usize::from(core.channels).checked_mul(usize::from(core.bit_depth))?;
    if derived != 0 && checked_rowbytes_for_width(width, derived) == Some(rowbytes) {
        return Some(derived);
    }

    CANDIDATES
        .into_iter()
        .find(|candidate| checked_rowbytes_for_width(width, *candidate) == Some(rowbytes))
}

pub(crate) fn copy_chunk_name(chunk_list: png_const_bytep, index: usize) -> Option<[png_byte; 4]> {
    if chunk_list.is_null() {
        return None;
    }

    let base = unsafe { chunk_list.add(index.checked_mul(5)?) };
    Some(unsafe { [*base, *base.add(1), *base.add(2), *base.add(3)] })
}

pub(crate) fn known_chunks_to_ignore() -> &'static [[png_byte; 4]] {
    static KNOWN_CHUNKS: [[png_byte; 4]; 18] = [
        *b"bKGD", *b"cHRM", *b"eXIf", *b"gAMA", *b"hIST", *b"iCCP", *b"iTXt", *b"oFFs",
        *b"pCAL", *b"pHYs", *b"sBIT", *b"sCAL", *b"sPLT", *b"sTER", *b"sRGB", *b"tEXt",
        *b"tIME", *b"zTXt",
    ];

    &KNOWN_CHUNKS
}

pub(crate) fn ancillary_chunk(name: [png_byte; 4]) -> bool {
    (name[0] & 0x20) != 0
}

pub(crate) fn safe_to_copy(name: [png_byte; 4]) -> bool {
    (name[3] & 0x20) != 0
}
