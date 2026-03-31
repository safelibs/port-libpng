use crate::common::PNG_FLAG_ROW_INIT;
use crate::types::*;
use core::ffi::c_int;

pub(crate) type KeepSymbol = core::sync::atomic::AtomicPtr<()>;

pub(crate) const PNG_HAVE_IEND: png_uint_32 = 0x10;

pub(crate) const PNG_HANDLE_CHUNK_AS_DEFAULT: c_int = 0;
pub(crate) const PNG_HANDLE_CHUNK_NEVER: c_int = 1;
pub(crate) const PNG_HANDLE_CHUNK_IF_SAFE: c_int = 2;
pub(crate) const PNG_HANDLE_CHUNK_ALWAYS: c_int = 3;
pub(crate) const PNG_HANDLE_CHUNK_LAST: c_int = 4;

pub(crate) const PNG_SIGNATURE: [png_byte; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
pub(crate) const PNG_IDAT: png_uint_32 = u32::from_be_bytes(*b"IDAT");
pub(crate) const PNG_IEND: png_uint_32 = u32::from_be_bytes(*b"IEND");
pub(crate) const PNG_IHDR: png_uint_32 = u32::from_be_bytes(*b"IHDR");
pub(crate) const PNG_PLTE: png_uint_32 = u32::from_be_bytes(*b"PLTE");

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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ProgressiveReadState {
    pub last_pause_bytes: usize,
    pub last_skip_bytes: png_uint_32,
    pub paused_with_save: bool,
    pub buffered: Vec<png_byte>,
    pub decode_offset: usize,
    pub current_input_start: usize,
    pub current_input_size: usize,
    pub info_emitted: bool,
    pub end_emitted: bool,
    pub decoded: bool,
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

pub(crate) fn chunk_name_u32(name: [png_byte; 4]) -> png_uint_32 {
    u32::from_be_bytes(name)
}

pub(crate) fn validate_chunk_name(name: [png_byte; 4]) -> bool {
    name.into_iter().all(|byte| byte.is_ascii_alphabetic())
}

pub(crate) fn known_chunk_names() -> &'static [[png_byte; 4]] {
    static KNOWN_CHUNKS: [[png_byte; 4]; 22] = [
        *b"IHDR", *b"PLTE", *b"IDAT", *b"IEND", *b"bKGD", *b"cHRM", *b"eXIf", *b"gAMA",
        *b"hIST", *b"iCCP", *b"iTXt", *b"oFFs", *b"pCAL", *b"pHYs", *b"sBIT", *b"sCAL",
        *b"sPLT", *b"sRGB", *b"tEXt", *b"tIME", *b"tRNS", *b"zTXt",
    ];

    &KNOWN_CHUNKS
}

pub(crate) fn is_known_chunk_name(name: [png_byte; 4]) -> bool {
    known_chunk_names().contains(&name)
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

pub(crate) fn crc32_update(mut crc: u32, bytes: &[u8]) -> u32 {
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }

    crc
}

pub(crate) fn png_crc32(name: [png_byte; 4], data: &[u8]) -> u32 {
    let crc = crc32_update(!0, &name);
    !crc32_update(crc, data)
}

pub(crate) fn update_phase_from_row_state(png_ptr: png_structrp, core: &png_safe_read_core) -> ReadPhase {
    if (core.flags & PNG_FLAG_ROW_INIT) != 0 {
        ReadPhase::ImageRows
    } else if (core.mode & PNG_HAVE_IEND) != 0 {
        ReadPhase::Terminal
    } else if core.idat_size != 0 || core.chunk_name == PNG_IDAT {
        ReadPhase::IdatStream
    } else {
        let _ = png_ptr;
        ReadPhase::ChunkHeader
    }
}
