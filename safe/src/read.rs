use crate::chunks::{
    ancillary_error_is_fatal, call_app_error, call_benign_error, call_error, call_warning,
    chunk_is_ancillary, dispatch_user_chunk_callback, keep_for_chunk, keep_requests_storage,
    read_core, read_info_core, reserve_chunk_cache_slot, set_read_phase, validate_parser_chunk,
    write_core,
};
use crate::common::{PNG_FLAG_ROW_INIT, PNG_HAVE_PNG_SIGNATURE};
use crate::interlace;
use crate::read_util::{
    checked_rowbytes_for_width, chunk_name_u32, is_known_chunk_name, png_crc32,
    update_phase_from_row_state, validate_chunk_name, ReadPhase,
    PNG_HANDLE_CHUNK_AS_DEFAULT, PNG_HANDLE_CHUNK_IF_SAFE, PNG_IDAT, PNG_IEND, PNG_IHDR,
    PNG_PLTE, PNG_SIGNATURE,
};
use crate::state;
use crate::types::*;
use crate::zlib;
use core::ffi::c_void;
use core::ptr;

const PNG_INTERLACE_TRANSFORM: png_uint_32 = 0x0002;
const PNG_HAVE_IHDR: png_uint_32 = 0x01;
const PNG_HAVE_PLTE: png_uint_32 = 0x02;
const PNG_HAVE_IDAT: png_uint_32 = 0x04;
const PNG_AFTER_IDAT: png_uint_32 = 0x08;
const PNG_HAVE_CHUNK_AFTER_IDAT: png_uint_32 = 0x2000;
const PNG_FLAG_ZSTREAM_ENDED: png_uint_32 = 0x0008;
const PNG_FLAG_CRC_ANCILLARY_USE: png_uint_32 = 0x0100;
const PNG_FLAG_CRC_ANCILLARY_NOWARN: png_uint_32 = 0x0200;
const PNG_FLAG_CRC_CRITICAL_USE: png_uint_32 = 0x0400;
const PNG_FLAG_CRC_CRITICAL_IGNORE: png_uint_32 = 0x0800;

const PNG_TEXT_COMPRESSION_NONE: i32 = -1;
const PNG_TEXT_COMPRESSION_ZTXT: i32 = 0;
const PNG_ITXT_COMPRESSION_NONE: i32 = 1;
const PNG_ITXT_COMPRESSION_ZTXT: i32 = 2;

unsafe extern "C" {
    fn png_safe_call_read_data(
        png_ptr: png_structrp,
        buffer: png_bytep,
        size: usize,
    ) -> core::ffi::c_int;
    fn png_safe_prepare_idat(png_ptr: png_structrp, length: png_uint_32) -> core::ffi::c_int;
    fn png_safe_complete_idat(png_ptr: png_structrp) -> core::ffi::c_int;
    fn png_safe_call_read_row(
        png_ptr: png_structrp,
        row: png_bytep,
        display_row: png_bytep,
    ) -> core::ffi::c_int;
    fn png_safe_call_read_start_row(png_ptr: png_structrp) -> core::ffi::c_int;
    fn png_safe_call_read_transform_info(
        png_ptr: png_structrp,
        info_ptr: png_inforp,
    ) -> core::ffi::c_int;

    fn png_safe_set_IHDR(
        png_ptr: png_structrp,
        info_ptr: png_inforp,
        width: png_uint_32,
        height: png_uint_32,
        bit_depth: core::ffi::c_int,
        color_type: core::ffi::c_int,
        interlace_type: core::ffi::c_int,
        compression_type: core::ffi::c_int,
        filter_type: core::ffi::c_int,
    ) -> core::ffi::c_int;
    fn png_safe_set_PLTE(
        png_ptr: png_structrp,
        info_ptr: png_inforp,
        palette: png_colorp,
        num_palette: core::ffi::c_int,
    ) -> core::ffi::c_int;
    fn png_safe_set_tRNS(
        png_ptr: png_structrp,
        info_ptr: png_inforp,
        trans_alpha: png_bytep,
        num_trans: core::ffi::c_int,
        trans_color: png_color_16p,
    ) -> core::ffi::c_int;
    fn png_safe_set_bKGD(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        background: png_color_16p,
    ) -> core::ffi::c_int;
    fn png_safe_set_cHRM_fixed(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        white_x: png_fixed_point,
        white_y: png_fixed_point,
        red_x: png_fixed_point,
        red_y: png_fixed_point,
        green_x: png_fixed_point,
        green_y: png_fixed_point,
        blue_x: png_fixed_point,
        blue_y: png_fixed_point,
    ) -> core::ffi::c_int;
    fn png_safe_set_eXIf_1(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        num_exif: png_uint_32,
        exif: png_bytep,
    ) -> core::ffi::c_int;
    fn png_safe_set_gAMA_fixed(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        file_gamma: png_fixed_point,
    ) -> core::ffi::c_int;
    fn png_safe_set_hIST(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        hist: png_const_uint_16p,
    ) -> core::ffi::c_int;
    fn png_safe_set_oFFs(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        offset_x: png_int_32,
        offset_y: png_int_32,
        unit_type: core::ffi::c_int,
    ) -> core::ffi::c_int;
    fn png_safe_set_pCAL(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        purpose: png_charp,
        x0: png_int_32,
        x1: png_int_32,
        type_: core::ffi::c_int,
        nparams: core::ffi::c_int,
        units: png_charp,
        params: png_charpp,
    ) -> core::ffi::c_int;
    fn png_safe_set_pHYs(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        res_x: png_uint_32,
        res_y: png_uint_32,
        unit_type: core::ffi::c_int,
    ) -> core::ffi::c_int;
    fn png_safe_set_sBIT(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        sig_bit: png_color_8p,
    ) -> core::ffi::c_int;
    fn png_safe_set_sCAL_s(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        unit: core::ffi::c_int,
        swidth: png_const_charp,
        sheight: png_const_charp,
    ) -> core::ffi::c_int;
    fn png_safe_set_sPLT(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        entries: png_sPLT_tp,
        num_entries: core::ffi::c_int,
    ) -> core::ffi::c_int;
    fn png_safe_set_sRGB(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        srgb_intent: core::ffi::c_int,
    ) -> core::ffi::c_int;
    fn png_safe_set_iCCP(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        name: png_const_charp,
        compression_type: core::ffi::c_int,
        profile: png_const_bytep,
        proflen: png_uint_32,
    ) -> core::ffi::c_int;
    fn png_safe_set_text(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        text_ptr: png_textp,
        num_text: core::ffi::c_int,
    ) -> core::ffi::c_int;
    fn png_safe_set_tIME(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        mod_time: png_timep,
    ) -> core::ffi::c_int;
    fn png_safe_set_unknown_chunks(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        unknowns: png_unknown_chunkp,
        num_unknowns: core::ffi::c_int,
    ) -> core::ffi::c_int;

    fn png_safe_parse_snapshot_capture(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> *mut c_void;
    fn png_safe_parse_snapshot_restore(
        png_ptr: png_structrp,
        info_ptr: png_inforp,
        snapshot: *const c_void,
    );
    fn png_safe_parse_snapshot_free(snapshot: *mut c_void);
}

struct ParseSnapshot {
    native: *mut c_void,
    png_state: Option<state::PngStructState>,
    info_state: Option<state::PngInfoState>,
}

unsafe fn snapshot_parse_state(png_ptr: png_structrp, info_ptr: png_inforp) -> ParseSnapshot {
    let native = unsafe { png_safe_parse_snapshot_capture(png_ptr, info_ptr) };
    if native.is_null() {
        let _ = unsafe { call_error(png_ptr, b"insufficient memory for parse snapshot\0") };
        unsafe { crate::error::png_longjmp(png_ptr, 1) }
    }

    ParseSnapshot {
        native,
        png_state: state::get_png(png_ptr),
        info_state: state::get_info(info_ptr),
    }
}

unsafe fn free_parse_snapshot(snapshot: &ParseSnapshot) {
    if !snapshot.native.is_null() {
        unsafe { png_safe_parse_snapshot_free(snapshot.native) };
    }
}

unsafe fn rollback_parse_state(png_ptr: png_structrp, info_ptr: png_inforp, snapshot: &ParseSnapshot) {
    unsafe { png_safe_parse_snapshot_restore(png_ptr, info_ptr, snapshot.native) };
    if let Some(png_state) = snapshot.png_state.clone() {
        state::register_png(png_ptr, png_state);
    }
    if let Some(info_state) = snapshot.info_state {
        state::register_info(info_ptr, info_state);
    }
    unsafe { free_parse_snapshot(snapshot) };
}

unsafe fn rollback_and_rethrow(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
) -> ! {
    unsafe { rollback_parse_state(png_ptr, info_ptr, snapshot) };
    crate::error::png_longjmp(png_ptr, 1)
}

unsafe fn error_and_rethrow(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    message: &'static [u8],
) -> ! {
    let _ = unsafe { call_error(png_ptr, message) };
    unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) }
}

unsafe fn read_exact_or_rethrow(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    buffer: &mut [u8],
) {
    if buffer.is_empty() {
        return;
    }

    if unsafe { png_safe_call_read_data(png_ptr, buffer.as_mut_ptr(), buffer.len()) } == 0 {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }
}

fn read_be_u32(bytes: &[u8]) -> png_uint_32 {
    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn read_be_u16(bytes: &[u8]) -> png_uint_16 {
    u16::from_be_bytes([bytes[0], bytes[1]])
}

fn read_be_i32(bytes: &[u8]) -> png_int_32 {
    i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn nul_terminated(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len() + 1);
    out.extend_from_slice(bytes);
    out.push(0);
    out
}

fn split_first_nul(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    let pos = bytes.iter().position(|byte| *byte == 0)?;
    Some((&bytes[..pos], &bytes[pos + 1..]))
}

unsafe fn apply_crc_policy_or_rethrow(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    chunk_name: [png_byte; 4],
    data: &[png_byte],
    actual_crc: png_uint_32,
) -> bool {
    let computed = png_crc32(chunk_name, data);
    if computed == actual_crc {
        return true;
    }

    let flags = read_core(png_ptr).flags;
    if chunk_is_ancillary(chunk_name) {
        let use_data = (flags & PNG_FLAG_CRC_ANCILLARY_USE) != 0;
        let warn = (flags & PNG_FLAG_CRC_ANCILLARY_NOWARN) == 0;

        if warn {
            let _ = call_warning(png_ptr, b"CRC error\0");
        } else if !use_data {
            error_and_rethrow(png_ptr, info_ptr, snapshot, b"CRC error\0");
        }

        return use_data;
    }

    if (flags & PNG_FLAG_CRC_CRITICAL_IGNORE) != 0 {
        return true;
    }

    if (flags & PNG_FLAG_CRC_CRITICAL_USE) != 0 {
        let _ = call_warning(png_ptr, b"CRC error\0");
        return true;
    }

    error_and_rethrow(png_ptr, info_ptr, snapshot, b"CRC error\0");
}

unsafe fn read_chunk_data_or_discard(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    chunk_name: [png_byte; 4],
    length: png_uint_32,
) -> Option<Vec<u8>> {
    let size = usize::try_from(length).ok()?;
    let mut data = vec![0; size];
    unsafe { read_exact_or_rethrow(png_ptr, info_ptr, snapshot, &mut data) };

    let mut crc = [0u8; 4];
    unsafe { read_exact_or_rethrow(png_ptr, info_ptr, snapshot, &mut crc) };

    let use_data = unsafe {
        apply_crc_policy_or_rethrow(png_ptr, info_ptr, snapshot, chunk_name, &data, read_be_u32(&crc))
    };
    use_data.then_some(data)
}

unsafe fn read_signature_or_rethrow(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
) {
    let sig_bytes = state::get_png(png_ptr)
        .map(|png_state| png_state.sig_bytes.clamp(0, 8) as usize)
        .unwrap_or(0);
    let mut signature = PNG_SIGNATURE;
    if sig_bytes < PNG_SIGNATURE.len() {
        unsafe {
            read_exact_or_rethrow(
                png_ptr,
                info_ptr,
                snapshot,
                &mut signature[sig_bytes..PNG_SIGNATURE.len()],
            );
        }
    }

    if signature != PNG_SIGNATURE {
        unsafe { error_and_rethrow(png_ptr, info_ptr, snapshot, b"Not a PNG file\0") };
    }

    state::update_png(png_ptr, |png_state| {
        png_state.sig_bytes = PNG_SIGNATURE.len() as i32;
        png_state.read_phase = ReadPhase::ChunkHeader;
    });

    let mut core = read_core(png_ptr);
    core.mode |= PNG_HAVE_PNG_SIGNATURE;
    write_core(png_ptr, &core);
}

unsafe fn read_chunk_header_or_rethrow(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
) -> (png_uint_32, [png_byte; 4]) {
    let mut header = [0u8; 8];
    unsafe { read_exact_or_rethrow(png_ptr, info_ptr, snapshot, &mut header) };

    let length = read_be_u32(&header[..4]);
    let name = [header[4], header[5], header[6], header[7]];
    if !validate_chunk_name(name) {
        unsafe { error_and_rethrow(png_ptr, info_ptr, snapshot, b"invalid chunk type\0") };
    }

    let chunk_name = chunk_name_u32(name);
    if let Err(message) = validate_parser_chunk(png_ptr, chunk_name, length) {
        unsafe { error_and_rethrow(png_ptr, info_ptr, snapshot, message) };
    }

    let mut core = read_core(png_ptr);
    core.chunk_name = chunk_name;
    write_core(png_ptr, &core);
    set_read_phase(png_ptr, ReadPhase::ChunkPayload);
    (length, name)
}

unsafe fn ancillary_benign_or_rethrow(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    message: &'static [u8],
) {
    if ancillary_error_is_fatal(png_ptr) {
        unsafe { error_and_rethrow(png_ptr, info_ptr, snapshot, message) };
    }

    let _ = unsafe { call_benign_error(png_ptr, message) };
}

unsafe fn reserve_text_slot_or_handle(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
) -> bool {
    match reserve_chunk_cache_slot(png_ptr, b"no space in chunk cache\0") {
        Ok(()) => true,
        Err(message) => {
            unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, message) };
            false
        }
    }
}

unsafe fn store_unknown_chunk_or_rethrow(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    name: [png_byte; 4],
    mut data: Vec<u8>,
) {
    if info_ptr.is_null() {
        return;
    }

    if !unsafe { reserve_text_slot_or_handle(png_ptr, info_ptr, snapshot) } {
        return;
    }

    let core = read_core(png_ptr);
    let mut chunk = png_unknown_chunk::default();
    chunk.name[..4].copy_from_slice(&name);
    chunk.size = data.len();
    chunk.location = (core.mode & 0xff) as png_byte;
    chunk.data = if data.is_empty() {
        ptr::null_mut()
    } else {
        data.as_mut_ptr()
    };

    if unsafe { png_safe_set_unknown_chunks(png_ptr, info_ptr, &mut chunk, 1) } == 0 {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }
}

unsafe fn handle_unknown_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    name: [png_byte; 4],
    data: Vec<u8>,
    mut keep: core::ffi::c_int,
    known_chunk: bool,
) {
    let mut callback_data = data;
    let mut callback_chunk = png_unknown_chunk::default();
    callback_chunk.name[..4].copy_from_slice(&name);
    callback_chunk.size = callback_data.len();
    callback_chunk.location = (read_core(png_ptr).mode & 0xff) as png_byte;
    callback_chunk.data = if callback_data.is_empty() {
        ptr::null_mut()
    } else {
        callback_data.as_mut_ptr()
    };

    if let Some(result) = dispatch_user_chunk_callback(png_ptr, &mut callback_chunk) {
        if result < 0 {
            unsafe { error_and_rethrow(png_ptr, info_ptr, snapshot, b"error in user chunk\0") };
        } else if result > 0 {
            return;
        } else if keep < PNG_HANDLE_CHUNK_IF_SAFE {
            keep = PNG_HANDLE_CHUNK_IF_SAFE;
        }
    }

    if keep_requests_storage(keep, name) {
        unsafe { store_unknown_chunk_or_rethrow(png_ptr, info_ptr, snapshot, name, callback_data) };
        return;
    }

    if !known_chunk && !chunk_is_ancillary(name) {
        unsafe { error_and_rethrow(png_ptr, info_ptr, snapshot, b"unknown critical chunk\0") };
    }
}

unsafe fn parse_ihdr_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    data: &[u8],
) {
    if data.len() != 13 {
        unsafe { error_and_rethrow(png_ptr, info_ptr, snapshot, b"IHDR must be 13 bytes\0") };
    }

    let width = read_be_u32(&data[0..4]);
    let height = read_be_u32(&data[4..8]);
    let bit_depth = i32::from(data[8]);
    let color_type = i32::from(data[9]);
    let compression_type = i32::from(data[10]);
    let filter_type = i32::from(data[11]);
    let interlace_type = i32::from(data[12]);

    if unsafe {
        png_safe_set_IHDR(
            png_ptr,
            info_ptr,
            width,
            height,
            bit_depth,
            color_type,
            interlace_type,
            compression_type,
            filter_type,
        )
    } == 0
    {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }

    let channels = match color_type {
        0 | 3 => 1u8,
        2 => 3u8,
        4 => 2u8,
        6 => 4u8,
        _ => unsafe {
            error_and_rethrow(png_ptr, info_ptr, snapshot, b"invalid IHDR color type\0")
        },
    };
    let pixel_depth = usize::try_from(bit_depth)
        .ok()
        .and_then(|depth| depth.checked_mul(usize::from(channels)))
        .and_then(|depth| u8::try_from(depth).ok())
        .unwrap_or_else(|| unsafe {
            error_and_rethrow(png_ptr, info_ptr, snapshot, b"invalid IHDR bit depth\0")
        });
    let rowbytes = checked_rowbytes_for_width(
        usize::try_from(width).unwrap_or_else(|_| unsafe {
            error_and_rethrow(png_ptr, info_ptr, snapshot, b"image width is too large\0")
        }),
        usize::from(pixel_depth),
    )
    .unwrap_or_else(|| unsafe {
        error_and_rethrow(png_ptr, info_ptr, snapshot, b"image row is too large\0")
    });

    let mut core = read_core(png_ptr);
    core.mode |= PNG_HAVE_IHDR;
    core.width = width;
    core.height = height;
    core.interlaced = interlace_type as png_byte;
    core.color_type = color_type as png_byte;
    core.bit_depth = bit_depth as png_byte;
    core.pixel_depth = pixel_depth;
    core.channels = channels;
    core.compression_type = compression_type as png_byte;
    core.filter_type = filter_type as png_byte;
    core.rowbytes = rowbytes;
    write_core(png_ptr, &core);
}

unsafe fn parse_plte_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    data: &[u8],
) {
    if data.is_empty() || data.len() % 3 != 0 || data.len() / 3 > 256 {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid palette\0") };
        return;
    }

    let mut palette = Vec::with_capacity(data.len() / 3);
    for rgb in data.chunks_exact(3) {
        palette.push(png_color {
            red: rgb[0],
            green: rgb[1],
            blue: rgb[2],
        });
    }

    if unsafe {
        png_safe_set_PLTE(
            png_ptr,
            info_ptr,
            palette.as_mut_ptr(),
            i32::try_from(palette.len()).unwrap_or(i32::MAX),
        )
    } == 0
    {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }

    let mut core = read_core(png_ptr);
    core.mode |= PNG_HAVE_PLTE;
    write_core(png_ptr, &core);
}

unsafe fn parse_chrm_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    data: &[u8],
) {
    if data.len() != 32 {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid cHRM\0") };
        return;
    }

    let values = [
        read_be_i32(&data[0..4]),
        read_be_i32(&data[4..8]),
        read_be_i32(&data[8..12]),
        read_be_i32(&data[12..16]),
        read_be_i32(&data[16..20]),
        read_be_i32(&data[20..24]),
        read_be_i32(&data[24..28]),
        read_be_i32(&data[28..32]),
    ];

    if unsafe {
        png_safe_set_cHRM_fixed(
            png_ptr,
            info_ptr,
            values[0],
            values[1],
            values[2],
            values[3],
            values[4],
            values[5],
            values[6],
            values[7],
        )
    } == 0
    {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }
}

unsafe fn parse_gama_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    data: &[u8],
) {
    if data.len() != 4 {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid gAMA\0") };
        return;
    }

    if unsafe { png_safe_set_gAMA_fixed(png_ptr, info_ptr, read_be_i32(data)) } == 0 {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }
}

unsafe fn parse_srgb_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    data: &[u8],
) {
    if data.len() != 1 {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid sRGB\0") };
        return;
    }

    if unsafe { png_safe_set_sRGB(png_ptr, info_ptr, i32::from(data[0])) } == 0 {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }
}

unsafe fn parse_sbit_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    data: &[u8],
) {
    let core = read_core(png_ptr);
    let expected = match core.color_type {
        0 => 1,
        2 | 3 => 3,
        4 => 2,
        6 => 4,
        _ => 0,
    };

    if expected == 0 || data.len() != expected {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid sBIT\0") };
        return;
    }

    let mut sig_bit = png_color_8::default();
    match expected {
        1 => sig_bit.gray = data[0],
        2 => {
            sig_bit.gray = data[0];
            sig_bit.alpha = data[1];
        }
        3 => {
            sig_bit.red = data[0];
            sig_bit.green = data[1];
            sig_bit.blue = data[2];
        }
        4 => {
            sig_bit.red = data[0];
            sig_bit.green = data[1];
            sig_bit.blue = data[2];
            sig_bit.alpha = data[3];
        }
        _ => {}
    }

    if unsafe { png_safe_set_sBIT(png_ptr, info_ptr, &mut sig_bit) } == 0 {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }
}

unsafe fn parse_bkgd_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    data: &[u8],
) {
    let core = read_core(png_ptr);
    let mut background = png_color_16::default();

    match core.color_type {
        3 => {
            if data.len() != 1 {
                unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid bKGD\0") };
                return;
            }
            background.index = data[0];
        }
        0 | 4 => {
            if data.len() != 2 {
                unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid bKGD\0") };
                return;
            }
            background.gray = read_be_u16(data);
        }
        2 | 6 => {
            if data.len() != 6 {
                unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid bKGD\0") };
                return;
            }
            background.red = read_be_u16(&data[0..2]);
            background.green = read_be_u16(&data[2..4]);
            background.blue = read_be_u16(&data[4..6]);
        }
        _ => {
            unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid bKGD\0") };
            return;
        }
    }

    if unsafe { png_safe_set_bKGD(png_ptr, info_ptr, &mut background) } == 0 {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }
}

unsafe fn parse_trns_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    data: &[u8],
) {
    let core = read_core(png_ptr);
    let mut trans_color = png_color_16::default();

    match core.color_type {
        3 => {
            let mut alpha = data.to_vec();
            if unsafe {
                png_safe_set_tRNS(
                    png_ptr,
                    info_ptr,
                    alpha.as_mut_ptr(),
                    i32::try_from(alpha.len()).unwrap_or(i32::MAX),
                    ptr::null_mut(),
                )
            } == 0
            {
                unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
            }
        }
        0 => {
            if data.len() != 2 {
                unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid tRNS\0") };
                return;
            }
            trans_color.gray = read_be_u16(data);
            if unsafe { png_safe_set_tRNS(png_ptr, info_ptr, ptr::null_mut(), 0, &mut trans_color) } == 0 {
                unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
            }
        }
        2 => {
            if data.len() != 6 {
                unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid tRNS\0") };
                return;
            }
            trans_color.red = read_be_u16(&data[0..2]);
            trans_color.green = read_be_u16(&data[2..4]);
            trans_color.blue = read_be_u16(&data[4..6]);
            if unsafe { png_safe_set_tRNS(png_ptr, info_ptr, ptr::null_mut(), 0, &mut trans_color) } == 0 {
                unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
            }
        }
        _ => {
            unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid tRNS\0") };
        }
    }
}

unsafe fn parse_phys_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    data: &[u8],
) {
    if data.len() != 9 {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid pHYs\0") };
        return;
    }

    if unsafe {
        png_safe_set_pHYs(
            png_ptr,
            info_ptr,
            read_be_u32(&data[0..4]),
            read_be_u32(&data[4..8]),
            i32::from(data[8]),
        )
    } == 0
    {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }
}

unsafe fn parse_offs_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    data: &[u8],
) {
    if data.len() != 9 {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid oFFs\0") };
        return;
    }

    if unsafe {
        png_safe_set_oFFs(
            png_ptr,
            info_ptr,
            read_be_i32(&data[0..4]),
            read_be_i32(&data[4..8]),
            i32::from(data[8]),
        )
    } == 0
    {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }
}

unsafe fn parse_time_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    data: &[u8],
) {
    if data.len() != 7 {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid tIME\0") };
        return;
    }

    let mut mod_time = png_time {
        year: read_be_u16(&data[0..2]),
        month: data[2],
        day: data[3],
        hour: data[4],
        minute: data[5],
        second: data[6],
    };
    if unsafe { png_safe_set_tIME(png_ptr, info_ptr, &mut mod_time) } == 0 {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }
}

unsafe fn parse_text_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    data: &[u8],
) {
    if unsafe { !reserve_text_slot_or_handle(png_ptr, info_ptr, snapshot) } {
        return;
    }

    let Some((keyword, text)) = split_first_nul(data) else {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"bad keyword\0") };
        return;
    };
    if keyword.is_empty() || keyword.len() > 79 {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"bad keyword\0") };
        return;
    }

    let mut key = nul_terminated(keyword);
    let mut value = nul_terminated(text);
    let mut text_info = png_text {
        compression: PNG_TEXT_COMPRESSION_NONE,
        key: key.as_mut_ptr().cast(),
        text: value.as_mut_ptr().cast(),
        text_length: text.len(),
        itxt_length: 0,
        lang: ptr::null_mut(),
        lang_key: ptr::null_mut(),
    };

    if unsafe { png_safe_set_text(png_ptr, info_ptr, &mut text_info, 1) } == 0 {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }
}

unsafe fn parse_ztxt_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    data: &[u8],
) {
    if unsafe { !reserve_text_slot_or_handle(png_ptr, info_ptr, snapshot) } {
        return;
    }

    let Some((keyword, rest)) = split_first_nul(data) else {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"bad keyword\0") };
        return;
    };
    if keyword.is_empty() || keyword.len() > 79 || rest.len() < 2 || rest[0] != 0 {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid zTXt\0") };
        return;
    }

    let inflated = match zlib::inflate_ancillary_zlib(
        png_ptr,
        &rest[1..],
        Some(data.len()),
        true,
    ) {
        Ok(bytes) => bytes,
        Err(message) => {
            unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, message) };
            return;
        }
    };

    let mut key = nul_terminated(keyword);
    let mut value = inflated;
    let text_len = value.len().saturating_sub(1);
    let mut text_info = png_text {
        compression: PNG_TEXT_COMPRESSION_ZTXT,
        key: key.as_mut_ptr().cast(),
        text: value.as_mut_ptr().cast(),
        text_length: text_len,
        itxt_length: 0,
        lang: ptr::null_mut(),
        lang_key: ptr::null_mut(),
    };

    if unsafe { png_safe_set_text(png_ptr, info_ptr, &mut text_info, 1) } == 0 {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }
}

unsafe fn parse_itxt_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    data: &[u8],
) {
    if unsafe { !reserve_text_slot_or_handle(png_ptr, info_ptr, snapshot) } {
        return;
    }

    let Some((keyword, rest)) = split_first_nul(data) else {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"bad keyword\0") };
        return;
    };
    if keyword.is_empty() || keyword.len() > 79 || rest.len() < 3 {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid iTXt\0") };
        return;
    }

    let compression_flag = rest[0];
    let compression_method = rest[1];
    let Some((language_tag, rest)) = split_first_nul(&rest[2..]) else {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid iTXt\0") };
        return;
    };
    let Some((translated_keyword, text_bytes)) = split_first_nul(rest) else {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid iTXt\0") };
        return;
    };

    let mut text = if compression_flag != 0 {
        if compression_method != 0 {
            unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid iTXt\0") };
            return;
        }

        match zlib::inflate_ancillary_zlib(png_ptr, text_bytes, Some(data.len()), false) {
            Ok(bytes) => bytes,
            Err(message) => {
                unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, message) };
                return;
            }
        }
    } else {
        text_bytes.to_vec()
    };
    text.push(0);

    let mut key = nul_terminated(keyword);
    let mut lang = nul_terminated(language_tag);
    let mut lang_key = nul_terminated(translated_keyword);
    let text_len = text.len().saturating_sub(1);
    let mut text_info = png_text {
        compression: if compression_flag != 0 {
            PNG_ITXT_COMPRESSION_ZTXT
        } else {
            PNG_ITXT_COMPRESSION_NONE
        },
        key: key.as_mut_ptr().cast(),
        text: text.as_mut_ptr().cast(),
        text_length: 0,
        itxt_length: text_len,
        lang: lang.as_mut_ptr().cast(),
        lang_key: lang_key.as_mut_ptr().cast(),
    };

    if unsafe { png_safe_set_text(png_ptr, info_ptr, &mut text_info, 1) } == 0 {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }
}

unsafe fn parse_iccp_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    data: &[u8],
) {
    let Some((keyword, rest)) = split_first_nul(data) else {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"bad iCCP profile\0") };
        return;
    };
    if keyword.is_empty() || keyword.len() > 79 || rest.len() < 2 || rest[0] != 0 {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"bad iCCP profile\0") };
        return;
    }

    let profile = match zlib::inflate_ancillary_zlib(png_ptr, &rest[1..], Some(data.len()), false) {
        Ok(bytes) => bytes,
        Err(message) => {
            unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, message) };
            return;
        }
    };
    if profile.len() < 132 {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"bad iCCP profile\0") };
        return;
    }

    let name = nul_terminated(keyword);
    if unsafe {
        png_safe_set_iCCP(
            png_ptr,
            info_ptr,
            name.as_ptr().cast(),
            0,
            profile.as_ptr(),
            u32::try_from(profile.len()).unwrap_or(u32::MAX),
        )
    } == 0
    {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }
}

unsafe fn parse_hist_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    data: &[u8],
) {
    let info = read_info_core(info_ptr);
    let expected = usize::from(info.num_palette).checked_mul(2).unwrap_or(usize::MAX);
    if expected == 0 || data.len() != expected {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid hIST\0") };
        return;
    }

    let mut hist = Vec::with_capacity(data.len() / 2);
    for value in data.chunks_exact(2) {
        hist.push(read_be_u16(value));
    }
    if unsafe { png_safe_set_hIST(png_ptr, info_ptr, hist.as_ptr()) } == 0 {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }
}

unsafe fn parse_splt_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    data: &[u8],
) {
    if unsafe { !reserve_text_slot_or_handle(png_ptr, info_ptr, snapshot) } {
        return;
    }

    let Some((name, rest)) = split_first_nul(data) else {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid sPLT\0") };
        return;
    };
    if name.is_empty() || rest.is_empty() {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid sPLT\0") };
        return;
    }

    let depth = rest[0];
    let entry_bytes = &rest[1..];
    let entry_size = match depth {
        8 => 6,
        16 => 10,
        _ => {
            unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid sPLT\0") };
            return;
        }
    };
    if entry_bytes.len() % entry_size != 0 {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid sPLT\0") };
        return;
    }

    let mut entries = Vec::with_capacity(entry_bytes.len() / entry_size);
    for entry in entry_bytes.chunks_exact(entry_size) {
        let mut item = png_sPLT_entry::default();
        if depth == 8 {
            item.red = u16::from(entry[0]);
            item.green = u16::from(entry[1]);
            item.blue = u16::from(entry[2]);
            item.alpha = u16::from(entry[3]);
            item.frequency = read_be_u16(&entry[4..6]);
        } else {
            item.red = read_be_u16(&entry[0..2]);
            item.green = read_be_u16(&entry[2..4]);
            item.blue = read_be_u16(&entry[4..6]);
            item.alpha = read_be_u16(&entry[6..8]);
            item.frequency = read_be_u16(&entry[8..10]);
        }
        entries.push(item);
    }

    let mut palette_name = nul_terminated(name);
    let mut splt = png_sPLT_t {
        name: palette_name.as_mut_ptr().cast(),
        depth,
        entries: entries.as_mut_ptr(),
        nentries: i32::try_from(entries.len()).unwrap_or(i32::MAX),
    };

    if unsafe { png_safe_set_sPLT(png_ptr, info_ptr, &mut splt, 1) } == 0 {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }
}

unsafe fn parse_exif_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    data: &[u8],
) {
    if unsafe {
        png_safe_set_eXIf_1(
            png_ptr,
            info_ptr,
            u32::try_from(data.len()).unwrap_or(u32::MAX),
            data.as_ptr().cast_mut(),
        )
    } == 0
    {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }
}

unsafe fn parse_pcal_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    data: &[u8],
) {
    let Some((purpose, rest)) = split_first_nul(data) else {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid pCAL\0") };
        return;
    };
    if rest.len() < 12 {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid pCAL\0") };
        return;
    }

    let x0 = read_be_i32(&rest[0..4]);
    let x1 = read_be_i32(&rest[4..8]);
    let type_ = i32::from(rest[8]);
    let nparams = i32::from(rest[9]);
    match (type_, nparams) {
        (0, 2) | (1, 3) | (2, 3) | (3, 4) => {}
        (0..=3, _) => {
            unsafe {
                ancillary_benign_or_rethrow(
                    png_ptr,
                    info_ptr,
                    snapshot,
                    b"invalid parameter count\0",
                )
            };
            return;
        }
        _ => {
            let _ = unsafe { call_benign_error(png_ptr, b"unrecognized equation type\0") };
        }
    }

    // Upstream null-terminates the chunk buffer in-place, so the units string
    // and final parameter may legally end at the chunk boundary.
    let mut strings = rest[10..].to_vec();
    strings.push(0);
    let Some((units, mut params_blob)) = split_first_nul(&strings) else {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid pCAL\0") };
        return;
    };
    let mut params_storage: Vec<Vec<u8>> = Vec::new();
    for _ in 0..nparams.max(0) {
        let Some((param, tail)) = split_first_nul(params_blob) else {
            unsafe {
                ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid data\0")
            };
            return;
        };
        params_storage.push(nul_terminated(param));
        params_blob = tail;
    }

    let mut param_ptrs: Vec<png_charp> = params_storage
        .iter_mut()
        .map(|param| param.as_mut_ptr().cast())
        .collect();
    let mut purpose = nul_terminated(purpose);
    let mut units = nul_terminated(units);
    if unsafe {
        png_safe_set_pCAL(
            png_ptr,
            info_ptr,
            purpose.as_mut_ptr().cast(),
            x0,
            x1,
            type_,
            nparams,
            units.as_mut_ptr().cast(),
            if param_ptrs.is_empty() {
                ptr::null_mut()
            } else {
                param_ptrs.as_mut_ptr()
            },
        )
    } == 0
    {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }
}

unsafe fn parse_scal_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    data: &[u8],
) {
    if data.len() < 3 {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid sCAL\0") };
        return;
    }

    let unit = i32::from(data[0]);
    let Some((width, rest)) = split_first_nul(&data[1..]) else {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid sCAL\0") };
        return;
    };
    if width.is_empty() || rest.is_empty() {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid sCAL\0") };
        return;
    }
    let height = if rest[rest.len() - 1] == 0 {
        &rest[..rest.len() - 1]
    } else {
        rest
    };
    if height.is_empty() {
        unsafe { ancillary_benign_or_rethrow(png_ptr, info_ptr, snapshot, b"invalid sCAL\0") };
        return;
    }

    let width = nul_terminated(width);
    let height = nul_terminated(height);
    if unsafe { png_safe_set_sCAL_s(png_ptr, info_ptr, unit, width.as_ptr().cast(), height.as_ptr().cast()) } == 0 {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }
}

unsafe fn parse_known_chunk(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
    name: [png_byte; 4],
    data: &[u8],
) {
    match name {
        [b'I', b'H', b'D', b'R'] => unsafe { parse_ihdr_chunk(png_ptr, info_ptr, snapshot, data) },
        [b'P', b'L', b'T', b'E'] => unsafe { parse_plte_chunk(png_ptr, info_ptr, snapshot, data) },
        [b'c', b'H', b'R', b'M'] => unsafe { parse_chrm_chunk(png_ptr, info_ptr, snapshot, data) },
        [b'g', b'A', b'M', b'A'] => unsafe { parse_gama_chunk(png_ptr, info_ptr, snapshot, data) },
        [b's', b'R', b'G', b'B'] => unsafe { parse_srgb_chunk(png_ptr, info_ptr, snapshot, data) },
        [b's', b'B', b'I', b'T'] => unsafe { parse_sbit_chunk(png_ptr, info_ptr, snapshot, data) },
        [b'b', b'K', b'G', b'D'] => unsafe { parse_bkgd_chunk(png_ptr, info_ptr, snapshot, data) },
        [b't', b'R', b'N', b'S'] => unsafe { parse_trns_chunk(png_ptr, info_ptr, snapshot, data) },
        [b'p', b'H', b'Y', b's'] => unsafe { parse_phys_chunk(png_ptr, info_ptr, snapshot, data) },
        [b'o', b'F', b'F', b's'] => unsafe { parse_offs_chunk(png_ptr, info_ptr, snapshot, data) },
        [b't', b'I', b'M', b'E'] => unsafe { parse_time_chunk(png_ptr, info_ptr, snapshot, data) },
        [b't', b'E', b'X', b't'] => unsafe { parse_text_chunk(png_ptr, info_ptr, snapshot, data) },
        [b'z', b'T', b'X', b't'] => unsafe { parse_ztxt_chunk(png_ptr, info_ptr, snapshot, data) },
        [b'i', b'T', b'X', b't'] => unsafe { parse_itxt_chunk(png_ptr, info_ptr, snapshot, data) },
        [b'i', b'C', b'C', b'P'] => unsafe { parse_iccp_chunk(png_ptr, info_ptr, snapshot, data) },
        [b'h', b'I', b'S', b'T'] => unsafe { parse_hist_chunk(png_ptr, info_ptr, snapshot, data) },
        [b's', b'P', b'L', b'T'] => unsafe { parse_splt_chunk(png_ptr, info_ptr, snapshot, data) },
        [b'e', b'X', b'I', b'f'] => unsafe { parse_exif_chunk(png_ptr, info_ptr, snapshot, data) },
        [b'p', b'C', b'A', b'L'] => unsafe { parse_pcal_chunk(png_ptr, info_ptr, snapshot, data) },
        [b's', b'C', b'A', b'L'] => unsafe { parse_scal_chunk(png_ptr, info_ptr, snapshot, data) },
        [b'I', b'E', b'N', b'D'] => {
            let mut core = read_core(png_ptr);
            core.mode |= crate::read_util::PNG_HAVE_IEND;
            write_core(png_ptr, &core);
        }
        _ => {}
    }
}

unsafe fn read_info_loop(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
) {
    loop {
        let (length, name) = unsafe { read_chunk_header_or_rethrow(png_ptr, info_ptr, snapshot) };
        let chunk_name = chunk_name_u32(name);
        let keep = keep_for_chunk(png_ptr, name);
        let known_chunk = is_known_chunk_name(name);
        let mut core = read_core(png_ptr);

        if chunk_name == PNG_IDAT {
            if (core.mode & PNG_HAVE_IHDR) == 0 {
                unsafe { error_and_rethrow(png_ptr, info_ptr, snapshot, b"Missing IHDR before IDAT\0") };
            } else if core.color_type == 3 && (core.mode & PNG_HAVE_PLTE) == 0 {
                unsafe { error_and_rethrow(png_ptr, info_ptr, snapshot, b"Missing PLTE before IDAT\0") };
            }
            core.mode |= PNG_HAVE_IDAT;
            write_core(png_ptr, &core);
        } else if (core.mode & PNG_HAVE_IDAT) != 0 {
            core.mode |= PNG_HAVE_CHUNK_AFTER_IDAT | PNG_AFTER_IDAT;
            write_core(png_ptr, &core);
        }

        if keep != PNG_HANDLE_CHUNK_AS_DEFAULT || !known_chunk {
            if let Some(data) = unsafe { read_chunk_data_or_discard(png_ptr, info_ptr, snapshot, name, length) } {
                unsafe { handle_unknown_chunk(png_ptr, info_ptr, snapshot, name, data, keep, known_chunk) };
            }

            if chunk_name == PNG_PLTE {
                let mut updated = read_core(png_ptr);
                updated.mode |= PNG_HAVE_PLTE;
                write_core(png_ptr, &updated);
            } else if chunk_name == PNG_IDAT {
                let mut updated = read_core(png_ptr);
                updated.idat_size = 0;
                write_core(png_ptr, &updated);
                set_read_phase(png_ptr, ReadPhase::ChunkHeader);
                break;
            } else if chunk_name == PNG_IEND {
                set_read_phase(png_ptr, ReadPhase::Terminal);
                break;
            }

            set_read_phase(png_ptr, ReadPhase::ChunkHeader);
            continue;
        }

        if chunk_name == PNG_IDAT {
            let mut updated = read_core(png_ptr);
            updated.idat_size = length;
            write_core(png_ptr, &updated);
            if unsafe { png_safe_prepare_idat(png_ptr, length) } == 0 {
                unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
            }
            set_read_phase(png_ptr, ReadPhase::IdatStream);
            break;
        }

        if let Some(data) = unsafe { read_chunk_data_or_discard(png_ptr, info_ptr, snapshot, name, length) } {
            unsafe { parse_known_chunk(png_ptr, info_ptr, snapshot, name, &data) };
        }

        if chunk_name == PNG_IEND {
            set_read_phase(png_ptr, ReadPhase::Terminal);
            break;
        }

        set_read_phase(png_ptr, ReadPhase::ChunkHeader);
    }
}

unsafe fn read_end_loop(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
    snapshot: &ParseSnapshot,
) {
    if keep_for_chunk(png_ptr, *b"IDAT") == PNG_HANDLE_CHUNK_AS_DEFAULT
        && unsafe { png_safe_complete_idat(png_ptr) } == 0
    {
        unsafe { rollback_and_rethrow(png_ptr, info_ptr, snapshot) };
    }

    let core = read_core(png_ptr);
    let info = read_info_core(info_ptr);
    if core.color_type == 3 && core.num_palette_max >= i32::from(info.num_palette) {
        let _ = unsafe { call_benign_error(png_ptr, b"Read palette index exceeding num_palette\0") };
    }

    loop {
        let (length, name) = unsafe { read_chunk_header_or_rethrow(png_ptr, info_ptr, snapshot) };
        let chunk_name = chunk_name_u32(name);
        let known_chunk = is_known_chunk_name(name);
        let keep = keep_for_chunk(png_ptr, name);

        if chunk_name != PNG_IDAT {
            let mut updated = read_core(png_ptr);
            updated.mode |= PNG_HAVE_CHUNK_AFTER_IDAT;
            write_core(png_ptr, &updated);
        }

        if chunk_name == PNG_IDAT {
            if length > 0 && (read_core(png_ptr).flags & PNG_FLAG_ZSTREAM_ENDED) == 0 {
                let _ = unsafe { call_benign_error(png_ptr, b"Too many IDATs found\0") };
            }
        }

        if info_ptr.is_null() && chunk_name != PNG_IEND && chunk_name != PNG_IHDR {
            let _ = unsafe { read_chunk_data_or_discard(png_ptr, info_ptr, snapshot, name, length) };
            set_read_phase(png_ptr, ReadPhase::ChunkHeader);
            continue;
        }

        if keep != PNG_HANDLE_CHUNK_AS_DEFAULT || !known_chunk {
            if let Some(data) = unsafe { read_chunk_data_or_discard(png_ptr, info_ptr, snapshot, name, length) } {
                unsafe { handle_unknown_chunk(png_ptr, info_ptr, snapshot, name, data, keep, known_chunk) };
            }
        } else if let Some(data) = unsafe { read_chunk_data_or_discard(png_ptr, info_ptr, snapshot, name, length) } {
            unsafe { parse_known_chunk(png_ptr, info_ptr, snapshot, name, &data) };
        }

        if chunk_name == PNG_IEND {
            set_read_phase(png_ptr, ReadPhase::Terminal);
            break;
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_info(png_ptr: png_structrp, info_ptr: png_inforp) {
    crate::abi_guard!(png_ptr, unsafe {
        let snapshot = snapshot_parse_state(png_ptr, info_ptr);
        read_signature_or_rethrow(png_ptr, info_ptr, &snapshot);
        read_info_loop(png_ptr, info_ptr, &snapshot);
        free_parse_snapshot(&snapshot);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_update_info(png_ptr: png_structrp, info_ptr: png_inforp) {
    crate::abi_guard!(png_ptr, unsafe {
        let snapshot = snapshot_parse_state(png_ptr, info_ptr);
        if (read_core(png_ptr).flags & PNG_FLAG_ROW_INIT) == 0 {
            if png_safe_call_read_start_row(png_ptr) == 0 {
                rollback_and_rethrow(png_ptr, info_ptr, &snapshot);
            }
            if png_safe_call_read_transform_info(png_ptr, info_ptr) == 0 {
                rollback_and_rethrow(png_ptr, info_ptr, &snapshot);
            }
        } else {
            let _ = call_app_error(
                png_ptr,
                b"png_read_update_info/png_start_read_image: duplicate call\0",
            );
        }

        let core = read_core(png_ptr);
        set_read_phase(png_ptr, update_phase_from_row_state(png_ptr, &core));
        free_parse_snapshot(&snapshot);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_start_read_image(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, unsafe {
        let snapshot = snapshot_parse_state(png_ptr, ptr::null_mut());
        if (read_core(png_ptr).flags & PNG_FLAG_ROW_INIT) == 0 {
            if png_safe_call_read_start_row(png_ptr) == 0 {
                rollback_and_rethrow(png_ptr, ptr::null_mut(), &snapshot);
            }
        } else {
            let _ = call_app_error(
                png_ptr,
                b"png_start_read_image/png_read_update_info: duplicate call\0",
            );
        }

        let core = read_core(png_ptr);
        set_read_phase(png_ptr, update_phase_from_row_state(png_ptr, &core));
        free_parse_snapshot(&snapshot);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_row(
    png_ptr: png_structrp,
    row: png_bytep,
    display_row: png_bytep,
) {
    crate::abi_guard!(png_ptr, unsafe {
        let snapshot = snapshot_parse_state(png_ptr, ptr::null_mut());
        if png_safe_call_read_row(png_ptr, row, display_row) == 0 {
            rollback_and_rethrow(png_ptr, ptr::null_mut(), &snapshot);
        }
        if !(row.is_null() && display_row.is_null()) {
            interlace::sanitize_row_padding(png_ptr, row, display_row);
        }
        let core = read_core(png_ptr);
        set_read_phase(png_ptr, update_phase_from_row_state(png_ptr, &core));
        free_parse_snapshot(&snapshot);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_rows(
    png_ptr: png_structrp,
    row: png_bytepp,
    display_row: png_bytepp,
    num_rows: png_uint_32,
) {
    crate::abi_guard!(png_ptr, unsafe {
        if png_ptr.is_null() {
            return;
        }

        let mut rp = row;
        let mut dp = display_row;
        for _ in 0..num_rows {
            let rptr = if rp.is_null() {
                ptr::null_mut()
            } else {
                let value = *rp;
                rp = rp.add(1);
                value
            };
            let dptr = if dp.is_null() {
                ptr::null_mut()
            } else {
                let value = *dp;
                dp = dp.add(1);
                value
            };
            png_read_row(png_ptr, rptr, dptr);
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_image(png_ptr: png_structrp, image: png_bytepp) {
    crate::abi_guard!(png_ptr, unsafe {
        if png_ptr.is_null() || image.is_null() {
            return;
        }

        let mut core = read_core(png_ptr);
        let passes = if (core.flags & crate::common::PNG_FLAG_ROW_INIT) == 0 {
            let passes = crate::interlace::png_set_interlace_handling(png_ptr);
            png_start_read_image(png_ptr);
            passes
        } else {
            if core.interlaced != 0 && (core.transformations & PNG_INTERLACE_TRANSFORM) == 0 {
                let _ = call_warning(
                    png_ptr,
                    b"Interlace handling should be turned on when using png_read_image\0",
                );
                core.num_rows = core.height;
                crate::chunks::write_core(png_ptr, &core);
            }

            crate::interlace::png_set_interlace_handling(png_ptr)
        };

        let image_height = core.height;
        for _pass in 0..passes {
            let mut rows = image;
            for _ in 0..image_height {
                png_read_row(png_ptr, *rows, ptr::null_mut());
                rows = rows.add(1);
            }
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_read_end(png_ptr: png_structrp, info_ptr: png_inforp) {
    crate::abi_guard!(png_ptr, unsafe {
        let snapshot = snapshot_parse_state(png_ptr, info_ptr);
        read_end_loop(png_ptr, info_ptr, &snapshot);
        set_read_phase(png_ptr, ReadPhase::Terminal);
        free_parse_snapshot(&snapshot);
    });
}
