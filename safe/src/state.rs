use crate::common::{
    PNG_IS_READ_STRUCT, PNG_USER_CHUNK_CACHE_MAX, PNG_USER_CHUNK_MALLOC_MAX,
    PNG_USER_HEIGHT_MAX, PNG_USER_WIDTH_MAX, WriteZlibSettings,
};
use crate::read_util::{
    PNG_HANDLE_CHUNK_AS_DEFAULT, ProgressiveReadState, ReadPhase, UnknownChunkSetting,
};
use crate::types::*;
use core::ffi::{c_char, c_int};
use core::num::NonZeroUsize;
use core::ptr;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, OnceLock};

#[derive(Clone)]
pub(crate) struct PngStructState {
    pub core: png_safe_read_core,
    pub is_read_struct: bool,
    pub error_ptr: png_voidp,
    pub error_fn: png_error_ptr,
    pub warning_fn: png_error_ptr,
    pub mem_ptr: png_voidp,
    pub malloc_fn: png_malloc_ptr,
    pub free_fn: png_free_ptr,
    pub io_ptr: png_voidp,
    pub read_data_fn: png_rw_ptr,
    pub write_data_fn: png_rw_ptr,
    pub output_flush_fn: png_flush_ptr,
    pub read_row_fn: png_read_status_ptr,
    pub write_row_fn: png_write_status_ptr,
    pub flush_rows: c_int,
    pub progressive_ptr: png_voidp,
    pub progressive_info_fn: png_progressive_info_ptr,
    pub progressive_row_fn: png_progressive_row_ptr,
    pub progressive_end_fn: png_progressive_end_ptr,
    pub user_chunk_ptr: png_voidp,
    pub read_user_chunk_fn: png_user_chunk_ptr,
    pub read_user_transform_fn: png_user_transform_ptr,
    pub write_user_transform_fn: png_user_transform_ptr,
    pub user_transform_ptr: png_voidp,
    pub user_transform_depth: c_int,
    pub user_transform_channels: c_int,
    pub user_width_max: png_uint_32,
    pub user_height_max: png_uint_32,
    pub user_chunk_cache_max: png_uint_32,
    pub user_chunk_malloc_max: png_alloc_size_t,
    pub benign_errors: c_int,
    pub check_for_invalid_index: c_int,
    pub palette_max: c_int,
    pub options: png_uint_32,
    pub sig_bytes: c_int,
    pub time_buffer: [c_char; 29],
    pub write_zlib: WriteZlibSettings,
    pub mng_features_permitted: png_uint_32,
    pub longjmp_fn: png_longjmp_ptr,
    pub jmp_buf_ptr: *mut JmpBuf,
    pub jmp_buf_size: usize,
    pub read_phase: ReadPhase,
    pub progressive_state: ProgressiveReadState,
    pub unknown_default_keep: c_int,
    pub unknown_chunk_list: Vec<UnknownChunkSetting>,
    pub pending_chunk_header: [png_byte; 8],
    pub has_pending_chunk_header: bool,
    pub captured_input: Vec<png_byte>,
    pub passthrough_written: bool,
    pub write_session: Option<WriteSessionState>,
}

unsafe impl Send for PngStructState {}

impl PngStructState {
    pub(crate) fn new_read(
        error_ptr: png_voidp,
        error_fn: png_error_ptr,
        warning_fn: png_error_ptr,
        mem_ptr: png_voidp,
        malloc_fn: png_malloc_ptr,
        free_fn: png_free_ptr,
    ) -> Self {
        let mut core = png_safe_read_core::default();
        core.mode = PNG_IS_READ_STRUCT;

        Self {
            core,
            is_read_struct: true,
            error_ptr,
            error_fn,
            warning_fn,
            mem_ptr,
            malloc_fn,
            free_fn,
            io_ptr: ptr::null_mut(),
            read_data_fn: None,
            write_data_fn: None,
            output_flush_fn: None,
            read_row_fn: None,
            write_row_fn: None,
            flush_rows: 0,
            progressive_ptr: ptr::null_mut(),
            progressive_info_fn: None,
            progressive_row_fn: None,
            progressive_end_fn: None,
            user_chunk_ptr: ptr::null_mut(),
            read_user_chunk_fn: None,
            read_user_transform_fn: None,
            write_user_transform_fn: None,
            user_transform_ptr: ptr::null_mut(),
            user_transform_depth: 0,
            user_transform_channels: 0,
            user_width_max: PNG_USER_WIDTH_MAX,
            user_height_max: PNG_USER_HEIGHT_MAX,
            user_chunk_cache_max: PNG_USER_CHUNK_CACHE_MAX,
            user_chunk_malloc_max: PNG_USER_CHUNK_MALLOC_MAX,
            benign_errors: 1,
            check_for_invalid_index: 1,
            palette_max: 0,
            options: 0,
            sig_bytes: 0,
            time_buffer: [0; 29],
            write_zlib: WriteZlibSettings::default(),
            mng_features_permitted: 0,
            longjmp_fn: None,
            jmp_buf_ptr: ptr::null_mut(),
            jmp_buf_size: 0,
            read_phase: ReadPhase::Signature,
            progressive_state: ProgressiveReadState::default(),
            unknown_default_keep: PNG_HANDLE_CHUNK_AS_DEFAULT,
            unknown_chunk_list: Vec::new(),
            pending_chunk_header: [0; 8],
            has_pending_chunk_header: false,
            captured_input: Vec::new(),
            passthrough_written: false,
            write_session: None,
        }
    }

    pub(crate) fn new_write(
        error_ptr: png_voidp,
        error_fn: png_error_ptr,
        warning_fn: png_error_ptr,
        mem_ptr: png_voidp,
        malloc_fn: png_malloc_ptr,
        free_fn: png_free_ptr,
    ) -> Self {
        let mut state = Self::new_read(
            error_ptr,
            error_fn,
            warning_fn,
            mem_ptr,
            malloc_fn,
            free_fn,
        );
        state.is_read_struct = false;
        state.core.mode &= !PNG_IS_READ_STRUCT;
        state.benign_errors = 0;

        state
    }
}

#[derive(Clone, Default)]
pub(crate) struct WriteSessionState {
    pub rowbytes: usize,
    pub image_data: Vec<png_byte>,
    pub seen_rows: Vec<bool>,
    pub header_text_count: usize,
    pub total_row_writes: u64,
}

#[derive(Clone, Default)]
pub(crate) struct OwnedTextChunk {
    pub compression: c_int,
    pub keyword: String,
    pub text: String,
    pub language_tag: String,
    pub translated_keyword: String,
}

#[derive(Clone, Default)]
pub(crate) struct OwnedUnknownChunk {
    pub name: [png_byte; 5],
    pub data: Vec<png_byte>,
    pub location: png_byte,
}

#[derive(Default)]
pub(crate) struct PngInfoState {
    pub core: png_safe_info_core,
    pub palette: Vec<png_color>,
    pub trans_alpha: Vec<png_byte>,
    pub hist: Vec<png_uint_16>,
    pub text_chunks: Vec<OwnedTextChunk>,
    pub exif: Vec<png_byte>,
    pub iccp_name: Vec<u8>,
    pub iccp_profile: Vec<png_byte>,
    pub phys: Option<(png_uint_32, png_uint_32, c_int)>,
    pub offs: Option<(png_int_32, png_int_32, c_int)>,
    pub time: Option<png_time>,
    pub scal_unit: c_int,
    pub scal_width: Vec<u8>,
    pub scal_height: Vec<u8>,
    pub unknown_chunks: Vec<OwnedUnknownChunk>,
    pub unknown_chunk_cache: Vec<png_unknown_chunk>,
}

unsafe impl Send for PngInfoState {}

impl Clone for PngInfoState {
    fn clone(&self) -> Self {
        Self {
            core: self.core,
            palette: self.palette.clone(),
            trans_alpha: self.trans_alpha.clone(),
            hist: self.hist.clone(),
            text_chunks: self.text_chunks.clone(),
            exif: self.exif.clone(),
            iccp_name: self.iccp_name.clone(),
            iccp_profile: self.iccp_profile.clone(),
            phys: self.phys,
            offs: self.offs,
            time: self.time,
            scal_unit: self.scal_unit,
            scal_width: self.scal_width.clone(),
            scal_height: self.scal_height.clone(),
            unknown_chunks: self.unknown_chunks.clone(),
            unknown_chunk_cache: Vec::new(),
        }
    }
}

// Rust owns the runtime registry behind the exported opaque handle values.
// The registry is keyed by the ABI pointer value, but the authoritative state
// now lives here rather than in an upstream sidecar runtime.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct PngHandleKey(NonZeroUsize);

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct InfoHandleKey(NonZeroUsize);

fn png_struct_states() -> &'static Mutex<HashMap<PngHandleKey, PngStructState>> {
    static STATES: OnceLock<Mutex<HashMap<PngHandleKey, PngStructState>>> = OnceLock::new();
    STATES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn png_info_states() -> &'static Mutex<HashMap<InfoHandleKey, PngInfoState>> {
    static STATES: OnceLock<Mutex<HashMap<InfoHandleKey, PngInfoState>>> = OnceLock::new();
    STATES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn latest_passthrough_bytes() -> &'static Mutex<Vec<png_byte>> {
    static BYTES: OnceLock<Mutex<Vec<png_byte>>> = OnceLock::new();
    BYTES.get_or_init(|| Mutex::new(Vec::new()))
}

fn lock_recover<T>(mutex: &'static Mutex<T>) -> MutexGuard<'static, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn png_key<T>(ptr: *mut T) -> Option<PngHandleKey> {
    NonZeroUsize::new(ptr as usize).map(PngHandleKey)
}

fn info_key<T>(ptr: *mut T) -> Option<InfoHandleKey> {
    NonZeroUsize::new(ptr as usize).map(InfoHandleKey)
}

pub(crate) fn register_png(png_ptr: png_structrp, state: PngStructState) {
    if let Some(key) = png_key(png_ptr) {
        lock_recover(png_struct_states()).insert(key, state);
    }
}

pub(crate) fn get_png(png_ptr: png_structrp) -> Option<PngStructState> {
    let key = png_key(png_ptr)?;
    lock_recover(png_struct_states()).get(&key).cloned()
}

pub(crate) fn update_png(png_ptr: png_structrp, update: impl FnOnce(&mut PngStructState)) {
    let Some(key) = png_key(png_ptr) else {
        return;
    };

    if let Some(state) = lock_recover(png_struct_states()).get_mut(&key) {
        update(state);
    }
}

pub(crate) fn with_png<R>(png_ptr: png_structrp, read: impl FnOnce(&PngStructState) -> R) -> Option<R> {
    let key = png_key(png_ptr)?;
    let states = lock_recover(png_struct_states());
    states.get(&key).map(read)
}

pub(crate) fn with_png_mut<R>(
    png_ptr: png_structrp,
    update: impl FnOnce(&mut PngStructState) -> R,
) -> Option<R> {
    let key = png_key(png_ptr)?;
    let mut states = lock_recover(png_struct_states());
    states.get_mut(&key).map(update)
}

pub(crate) fn remove_png(png_ptr: png_structrp) -> Option<PngStructState> {
    let key = png_key(png_ptr)?;
    lock_recover(png_struct_states()).remove(&key)
}

pub(crate) fn register_info(info_ptr: png_infop, state: PngInfoState) {
    if let Some(key) = info_key(info_ptr) {
        lock_recover(png_info_states()).insert(key, state);
    }
}

pub(crate) fn register_default_info(info_ptr: png_infop) {
    register_info(info_ptr, PngInfoState::default());
}

pub(crate) fn get_info(info_ptr: png_infop) -> Option<PngInfoState> {
    let key = info_key(info_ptr)?;
    lock_recover(png_info_states()).get(&key).cloned()
}

pub(crate) fn update_info(info_ptr: png_infop, update: impl FnOnce(&mut PngInfoState)) {
    let Some(key) = info_key(info_ptr) else {
        return;
    };

    if let Some(state) = lock_recover(png_info_states()).get_mut(&key) {
        update(state);
    }
}

pub(crate) fn with_info<R>(info_ptr: png_infop, read: impl FnOnce(&PngInfoState) -> R) -> Option<R> {
    let key = info_key(info_ptr)?;
    let states = lock_recover(png_info_states());
    states.get(&key).map(read)
}

pub(crate) fn with_info_mut<R>(
    info_ptr: png_infop,
    update: impl FnOnce(&mut PngInfoState) -> R,
) -> Option<R> {
    let key = info_key(info_ptr)?;
    let mut states = lock_recover(png_info_states());
    states.get_mut(&key).map(update)
}

pub(crate) fn remove_info(info_ptr: png_infop) -> Option<PngInfoState> {
    let key = info_key(info_ptr)?;
    lock_recover(png_info_states()).remove(&key)
}

pub(crate) fn move_info(old_info_ptr: png_infop, new_info_ptr: png_infop) {
    let Some(old_key) = info_key(old_info_ptr) else {
        if !new_info_ptr.is_null() {
            register_default_info(new_info_ptr);
        }
        return;
    };

    let mut states = lock_recover(png_info_states());
    let state = states.remove(&old_key);

    if let Some(new_key) = info_key(new_info_ptr) {
        states.insert(new_key, state.unwrap_or_default());
    }
}

pub(crate) fn append_captured_read_data(png_ptr: png_structrp, bytes: &[png_byte]) {
    if bytes.is_empty() {
        return;
    }

    update_png(png_ptr, |state| {
        state.captured_input.extend_from_slice(bytes);
        let mut latest = lock_recover(latest_passthrough_bytes());
        latest.clear();
        latest.extend_from_slice(&state.captured_input);
    });
}

pub(crate) fn latest_captured_read_data() -> Option<Vec<png_byte>> {
    let latest = lock_recover(latest_passthrough_bytes());
    if latest.is_empty() {
        None
    } else {
        Some(latest.clone())
    }
}
