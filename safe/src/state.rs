use crate::common::{
    PNG_FLAG_APP_WARNINGS_WARN, PNG_FLAG_BENIGN_ERRORS_WARN, PNG_IS_READ_STRUCT,
    PNG_USER_CHUNK_CACHE_MAX, PNG_USER_CHUNK_MALLOC_MAX, PNG_USER_HEIGHT_MAX, PNG_USER_WIDTH_MAX,
};
use crate::types::*;
use core::ffi::{c_char, c_int, c_void};
use core::mem;
use core::ptr;

unsafe extern "C" {
    pub fn png_safe_longjmp_state_size() -> usize;
    pub fn png_safe_longjmp_state_set(storage: *mut c_void) -> c_int;
    pub fn png_safe_longjmp_state_invoke(
        storage: *mut c_void,
        callback: Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
        context: *mut c_void,
    ) -> c_int;
    pub fn png_safe_longjmp_state_jump(storage: *mut c_void, value: c_int) -> !;
    pub fn png_safe_longjmp_state_buf(storage: *mut c_void) -> *mut c_void;
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PngStructState {
    pub jmp_buf_ptr: *mut JmpBuf,
    pub longjmp_fn: png_longjmp_ptr,
    pub jmp_buf_size: usize,
    pub longjmp_storage: *mut c_void,
    pub longjmp_storage_size: usize,
    pub error_fn: png_error_ptr,
    pub warning_fn: png_error_ptr,
    pub error_ptr: png_voidp,
    pub write_data_fn: png_rw_ptr,
    pub read_data_fn: png_rw_ptr,
    pub io_ptr: png_voidp,
    pub output_flush_fn: png_flush_ptr,
    pub read_user_transform_fn: png_user_transform_ptr,
    pub write_user_transform_fn: png_user_transform_ptr,
    pub user_transform_ptr: png_voidp,
    pub user_transform_depth: png_byte,
    pub user_transform_channels: png_byte,
    pub sig_bytes: png_byte,
    pub reserved0: png_byte,
    pub mode: png_uint_32,
    pub flags: png_uint_32,
    pub transformations: png_uint_32,
    pub width: png_uint_32,
    pub height: png_uint_32,
    pub num_rows: png_uint_32,
    pub rowbytes: usize,
    pub row_number: png_uint_32,
    pub chunk_name: png_uint_32,
    pub write_row_fn: png_write_status_ptr,
    pub read_row_fn: png_read_status_ptr,
    pub info_fn: png_progressive_info_ptr,
    pub row_fn: png_progressive_row_ptr,
    pub end_fn: png_progressive_end_ptr,
    pub options: png_uint_32,
    pub time_buffer: [c_char; 29],
    pub free_me: png_uint_32,
    pub user_chunk_ptr: png_voidp,
    pub read_user_chunk_fn: png_user_chunk_ptr,
    pub num_palette_max: c_int,
    pub num_trans: png_uint_16,
    pub mem_ptr: png_voidp,
    pub malloc_fn: png_malloc_ptr,
    pub free_fn: png_free_ptr,
    pub user_width_max: png_uint_32,
    pub user_height_max: png_uint_32,
    pub user_chunk_cache_max: png_uint_32,
    pub user_chunk_malloc_max: png_alloc_size_t,
    pub io_state: png_uint_32,
    pub zbuffer_size: png_uint,
}

impl PngStructState {
    pub fn zeroed() -> Self {
        unsafe { mem::zeroed() }
    }

    pub fn defaults(
        is_read: bool,
        error_ptr: png_voidp,
        error_fn: png_error_ptr,
        warning_fn: png_error_ptr,
        mem_ptr: png_voidp,
        malloc_fn: png_malloc_ptr,
        free_fn: png_free_ptr,
    ) -> Self {
        let mut state = Self::zeroed();
        state.error_ptr = error_ptr;
        state.error_fn = error_fn;
        state.warning_fn = warning_fn;
        state.mem_ptr = mem_ptr;
        state.malloc_fn = malloc_fn;
        state.free_fn = free_fn;
        state.user_width_max = PNG_USER_WIDTH_MAX;
        state.user_height_max = PNG_USER_HEIGHT_MAX;
        state.user_chunk_cache_max = PNG_USER_CHUNK_CACHE_MAX;
        state.user_chunk_malloc_max = PNG_USER_CHUNK_MALLOC_MAX;
        state.num_palette_max = 0;
        state.zbuffer_size = crate::common::PNG_ZBUF_SIZE;
        state.flags |= PNG_FLAG_BENIGN_ERRORS_WARN | PNG_FLAG_APP_WARNINGS_WARN;
        if is_read {
            state.mode = PNG_IS_READ_STRUCT;
        }
        state
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PngInfoState {
    pub width: png_uint_32,
    pub height: png_uint_32,
    pub valid: png_uint_32,
    pub rowbytes: usize,
    pub palette: png_colorp,
    pub num_palette: png_uint_16,
    pub num_trans: png_uint_16,
    pub bit_depth: png_byte,
    pub color_type: png_byte,
    pub compression_type: png_byte,
    pub filter_type: png_byte,
    pub interlace_type: png_byte,
    pub channels: png_byte,
    pub pixel_depth: png_byte,
    pub spare_byte: png_byte,
    pub signature: [png_byte; 8],
    pub iccp_name: png_charp,
    pub iccp_profile: png_bytep,
    pub iccp_proflen: png_uint_32,
    pub num_text: c_int,
    pub max_text: c_int,
    pub text: png_textp,
    pub mod_time: png_time,
    pub sig_bit: png_color_8,
    pub trans_alpha: png_bytep,
    pub trans_color: png_color_16,
    pub background: png_color_16,
    pub x_offset: png_int_32,
    pub y_offset: png_int_32,
    pub offset_unit_type: png_byte,
    pub x_pixels_per_unit: png_uint_32,
    pub y_pixels_per_unit: png_uint_32,
    pub phys_unit_type: png_byte,
    pub num_exif: c_int,
    pub exif: png_bytep,
    pub eXIf_buf: png_bytep,
    pub hist: png_uint_16p,
    pub pcal_purpose: png_charp,
    pub pcal_X0: png_int_32,
    pub pcal_X1: png_int_32,
    pub pcal_units: png_charp,
    pub pcal_params: png_charpp,
    pub pcal_type: png_byte,
    pub pcal_nparams: png_byte,
    pub free_me: png_uint_32,
    pub unknown_chunks: png_unknown_chunkp,
    pub unknown_chunks_num: c_int,
    pub splt_palettes: png_sPLT_tp,
    pub splt_palettes_num: c_int,
    pub scal_unit: png_byte,
    pub scal_s_width: png_charp,
    pub scal_s_height: png_charp,
    pub row_pointers: png_bytepp,
}

impl PngInfoState {
    pub fn zeroed() -> Self {
        unsafe { mem::zeroed() }
    }
}

pub fn png_ptr_state<'a>(png_ptr: png_structrp) -> Option<&'a mut PngStructState> {
    unsafe { png_ptr.cast::<PngStructState>().as_mut() }
}

pub fn png_ptr_state_const<'a>(png_ptr: png_const_structrp) -> Option<&'a PngStructState> {
    unsafe { png_ptr.cast::<PngStructState>().as_ref() }
}

pub fn info_ptr_state<'a>(info_ptr: png_inforp) -> Option<&'a mut PngInfoState> {
    unsafe { info_ptr.cast::<PngInfoState>().as_mut() }
}

pub fn info_ptr_state_const<'a>(info_ptr: png_const_inforp) -> Option<&'a PngInfoState> {
    unsafe { info_ptr.cast::<PngInfoState>().as_ref() }
}

pub fn alloc_longjmp_storage() -> (*mut c_void, usize, *mut JmpBuf) {
    let size = unsafe { png_safe_longjmp_state_size() };
    let storage = unsafe { libc::malloc(size) };
    if storage.is_null() {
        return (ptr::null_mut(), size, ptr::null_mut());
    }

    unsafe {
        libc::memset(storage, 0, size);
    }
    let jmp_buf_ptr = unsafe { png_safe_longjmp_state_buf(storage) }.cast::<JmpBuf>();
    (storage, size, jmp_buf_ptr)
}
