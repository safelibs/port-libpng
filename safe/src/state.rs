use crate::common::{
    PNG_USER_CHUNK_CACHE_MAX, PNG_USER_CHUNK_MALLOC_MAX, PNG_USER_HEIGHT_MAX,
    PNG_USER_WIDTH_MAX,
};
use crate::types::*;
use core::ffi::c_int;
use core::ptr;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Copy)]
pub(crate) struct PngStructState {
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
    pub longjmp_fn: png_longjmp_ptr,
    pub jmp_buf_ptr: *mut JmpBuf,
    pub jmp_buf_size: usize,
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
        Self {
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
            longjmp_fn: None,
            jmp_buf_ptr: ptr::null_mut(),
            jmp_buf_size: 0,
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
        Self {
            is_read_struct: false,
            benign_errors: 0,
            ..Self::new_read(error_ptr, error_fn, warning_fn, mem_ptr, malloc_fn, free_fn)
        }
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct PngInfoState {
    pub free_me: png_uint_32,
    pub row_pointers: png_bytepp,
}

unsafe impl Send for PngInfoState {}

fn png_struct_states() -> &'static Mutex<HashMap<usize, PngStructState>> {
    static STATES: OnceLock<Mutex<HashMap<usize, PngStructState>>> = OnceLock::new();
    STATES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn png_info_states() -> &'static Mutex<HashMap<usize, PngInfoState>> {
    static STATES: OnceLock<Mutex<HashMap<usize, PngInfoState>>> = OnceLock::new();
    STATES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn png_key<T>(ptr: *mut T) -> Option<usize> {
    (!ptr.is_null()).then_some(ptr as usize)
}

fn info_key<T>(ptr: *mut T) -> Option<usize> {
    (!ptr.is_null()).then_some(ptr as usize)
}

pub(crate) fn register_png(png_ptr: png_structrp, state: PngStructState) {
    if let Some(key) = png_key(png_ptr) {
        png_struct_states().lock().unwrap().insert(key, state);
    }
}

pub(crate) fn get_png(png_ptr: png_structrp) -> Option<PngStructState> {
    let key = png_key(png_ptr)?;
    png_struct_states().lock().unwrap().get(&key).copied()
}

pub(crate) fn update_png(png_ptr: png_structrp, update: impl FnOnce(&mut PngStructState)) {
    let Some(key) = png_key(png_ptr) else {
        return;
    };

    if let Some(state) = png_struct_states().lock().unwrap().get_mut(&key) {
        update(state);
    }
}

pub(crate) fn remove_png(png_ptr: png_structrp) -> Option<PngStructState> {
    let key = png_key(png_ptr)?;
    png_struct_states().lock().unwrap().remove(&key)
}

pub(crate) fn register_info(info_ptr: png_infop, state: PngInfoState) {
    if let Some(key) = info_key(info_ptr) {
        png_info_states().lock().unwrap().insert(key, state);
    }
}

pub(crate) fn register_default_info(info_ptr: png_infop) {
    register_info(info_ptr, PngInfoState::default());
}

pub(crate) fn get_info(info_ptr: png_infop) -> Option<PngInfoState> {
    let key = info_key(info_ptr)?;
    png_info_states().lock().unwrap().get(&key).copied()
}

pub(crate) fn update_info(info_ptr: png_infop, update: impl FnOnce(&mut PngInfoState)) {
    let Some(key) = info_key(info_ptr) else {
        return;
    };

    if let Some(state) = png_info_states().lock().unwrap().get_mut(&key) {
        update(state);
    }
}

pub(crate) fn remove_info(info_ptr: png_infop) -> Option<PngInfoState> {
    let key = info_key(info_ptr)?;
    png_info_states().lock().unwrap().remove(&key)
}

pub(crate) fn move_info(old_info_ptr: png_infop, new_info_ptr: png_infop) {
    let Some(old_key) = info_key(old_info_ptr) else {
        if !new_info_ptr.is_null() {
            register_default_info(new_info_ptr);
        }
        return;
    };

    let mut states = png_info_states().lock().unwrap();
    let state = states.remove(&old_key);

    if let Some(new_key) = info_key(new_info_ptr) {
        states.insert(new_key, state.unwrap_or_default());
    }
}
