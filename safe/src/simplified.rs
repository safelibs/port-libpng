use crate::types::*;
use libc::FILE;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Copy, Debug, Default)]
struct SimplifiedReadState {
    source_kind: u8,
    begin_called: bool,
    finish_called: bool,
    last_result: bool,
    normalized_row_stride: usize,
    warning_or_error: png_uint_32,
    message_len: usize,
    opaque_live: bool,
}

fn image_states() -> &'static Mutex<HashMap<usize, SimplifiedReadState>> {
    static IMAGE_STATES: OnceLock<Mutex<HashMap<usize, SimplifiedReadState>>> = OnceLock::new();
    IMAGE_STATES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn update_image_state(image: png_imagep, update: impl FnOnce(&mut SimplifiedReadState)) {
    if image.is_null() {
        return;
    }

    if let Ok(mut states) = image_states().lock() {
        update(states.entry(image as usize).or_default());
    }
}

fn clear_image_state(image: png_const_imagep) {
    if image.is_null() {
        return;
    }

    if let Ok(mut states) = image_states().lock() {
        states.remove(&(image as usize));
    }
}

fn current_message_len(image: png_const_imagep) -> usize {
    if image.is_null() {
        return 0;
    }

    let message = unsafe { &(*image).message };
    message.iter().position(|byte| *byte == 0).unwrap_or(message.len())
}

fn finish_row_stride(row_stride: png_int_32) -> usize {
    if row_stride == 0 {
        0
    } else if row_stride < 0 {
        row_stride.unsigned_abs() as usize
    } else {
        row_stride as usize
    }
}

unsafe extern "C" {
    fn png_safe_call_image_begin_read_from_file(
        image: png_imagep,
        file_name: png_const_charp,
    ) -> core::ffi::c_int;
    fn png_safe_call_image_begin_read_from_stdio(
        image: png_imagep,
        file: *mut FILE,
    ) -> core::ffi::c_int;
    fn png_safe_call_image_begin_read_from_memory(
        image: png_imagep,
        memory: png_const_voidp,
        size: usize,
    ) -> core::ffi::c_int;
    fn png_safe_call_image_finish_read(
        image: png_imagep,
        background: png_const_colorp,
        buffer: png_voidp,
        row_stride: png_int_32,
        colormap: png_voidp,
    ) -> core::ffi::c_int;
    fn png_safe_call_image_free(image: png_imagep);
}

fn record_begin_result(image: png_imagep, source_kind: u8, result: core::ffi::c_int) {
    update_image_state(image, |state| {
        state.source_kind = source_kind;
        state.begin_called = true;
        state.finish_called = false;
        state.last_result = result != 0;
        state.warning_or_error = unsafe { (*image).warning_or_error };
        state.message_len = current_message_len(image);
        state.opaque_live = unsafe { !(*image).opaque.is_null() };
    });
}

fn record_finish_result(image: png_imagep, result: core::ffi::c_int, row_stride: png_int_32) {
    update_image_state(image, |state| {
        state.finish_called = true;
        state.last_result = result != 0;
        state.normalized_row_stride = finish_row_stride(row_stride);
        state.warning_or_error = unsafe { (*image).warning_or_error };
        state.message_len = current_message_len(image);
        state.opaque_live = unsafe { !(*image).opaque.is_null() };
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_begin_read_from_file(
    image: png_imagep,
    file_name: png_const_charp,
) -> core::ffi::c_int {
    clear_image_state(image);
    let result = unsafe { png_safe_call_image_begin_read_from_file(image, file_name) };
    record_begin_result(image, 1, result);
    result
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_begin_read_from_stdio(
    image: png_imagep,
    file: *mut FILE,
) -> core::ffi::c_int {
    clear_image_state(image);
    let result = unsafe { png_safe_call_image_begin_read_from_stdio(image, file) };
    record_begin_result(image, 2, result);
    result
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_begin_read_from_memory(
    image: png_imagep,
    memory: png_const_voidp,
    size: usize,
) -> core::ffi::c_int {
    clear_image_state(image);
    let result = unsafe { png_safe_call_image_begin_read_from_memory(image, memory, size) };
    record_begin_result(image, 3, result);
    result
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_finish_read(
    image: png_imagep,
    background: png_const_colorp,
    buffer: png_voidp,
    row_stride: png_int_32,
    colormap: png_voidp,
) -> core::ffi::c_int {
    let result =
        unsafe { png_safe_call_image_finish_read(image, background, buffer, row_stride, colormap) };
    record_finish_result(image, result, row_stride);
    result
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_image_free(image: png_imagep) {
    unsafe {
        png_safe_call_image_free(image);
    }
    clear_image_state(image);
}
