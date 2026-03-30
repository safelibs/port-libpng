use crate::types::*;
use core::ffi::c_int;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct OutputInfo {
    pub info_ptr: usize,
    pub width: png_uint_32,
    pub height: png_uint_32,
    pub bit_depth: png_byte,
    pub channels: png_byte,
    pub rowbytes: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct TransformState {
    pub expand: bool,
    pub expand_16: bool,
    pub palette_to_rgb: bool,
    pub trns_to_alpha: bool,
    pub gray_to_rgb: bool,
    pub scale_16: bool,
    pub strip_16: bool,
    pub quantize: bool,
    pub shift: bool,
    pub swap_alpha: bool,
    pub invert_alpha: bool,
    pub invert_mono: bool,
    pub bgr: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ColorspaceState {
    pub last_xyz_sums: [i64; 3],
    pub degenerate_xyz: bool,
    pub rgb_to_gray_coefficients: Option<(png_uint_16, png_uint_16)>,
    pub background_requested: bool,
    pub alpha_mode_requested: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ReadState {
    pub output: OutputInfo,
    pub transforms: TransformState,
    pub colorspace: ColorspaceState,
    pub invalid_index_enabled: Option<bool>,
    pub palette_max: c_int,
    pub interlace_passes: c_int,
}

impl Default for ReadState {
    fn default() -> Self {
        Self {
            output: OutputInfo::default(),
            transforms: TransformState::default(),
            colorspace: ColorspaceState::default(),
            invalid_index_enabled: None,
            palette_max: -1,
            interlace_passes: 1,
        }
    }
}

fn read_states() -> &'static Mutex<HashMap<usize, ReadState>> {
    static READ_STATES: OnceLock<Mutex<HashMap<usize, ReadState>>> = OnceLock::new();
    READ_STATES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn update_read_state(png_ptr: png_structrp, update: impl FnOnce(&mut ReadState)) {
    if png_ptr.is_null() {
        return;
    }

    if let Ok(mut states) = read_states().lock() {
        update(states.entry(png_ptr as usize).or_default());
    }
}

pub(crate) fn with_transform_state(
    png_ptr: png_structrp,
    update: impl FnOnce(&mut TransformState),
) {
    update_read_state(png_ptr, |state| update(&mut state.transforms));
}

pub(crate) fn with_colorspace_state(
    png_ptr: png_structrp,
    update: impl FnOnce(&mut ColorspaceState),
) {
    update_read_state(png_ptr, |state| update(&mut state.colorspace));
}

pub(crate) fn read_state_snapshot(png_ptr: png_const_structrp) -> Option<ReadState> {
    if png_ptr.is_null() {
        return None;
    }

    read_states()
        .lock()
        .ok()
        .and_then(|states| states.get(&(png_ptr as usize)).copied())
}

pub(crate) fn output_info_for(png_ptr: png_const_structrp) -> Option<OutputInfo> {
    read_state_snapshot(png_ptr).map(|state| state.output)
}

pub(crate) fn clear_read_state(png_ptr: png_const_structrp) {
    if png_ptr.is_null() {
        return;
    }

    if let Ok(mut states) = read_states().lock() {
        states.remove(&(png_ptr as usize));
    }
}

unsafe extern "C" {
    fn png_get_image_width(png_ptr: png_const_structrp, info_ptr: png_const_inforp)
    -> png_uint_32;
    fn png_get_image_height(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_uint_32;
    fn png_get_bit_depth(png_ptr: png_const_structrp, info_ptr: png_const_inforp) -> png_byte;
    fn png_get_channels(png_ptr: png_const_structrp, info_ptr: png_const_inforp) -> png_byte;
    fn png_get_rowbytes(png_ptr: png_const_structrp, info_ptr: png_const_inforp) -> usize;

    fn upstream_png_set_check_for_invalid_index(png_ptr: png_structrp, allowed: c_int);
    fn upstream_png_get_palette_max(png_ptr: png_const_structp, info_ptr: png_const_infop)
    -> c_int;
}

pub(crate) unsafe fn refresh_output_info(
    png_ptr: png_structrp,
    info_ptr: png_inforp,
) -> Option<OutputInfo> {
    if png_ptr.is_null() || info_ptr.is_null() {
        return None;
    }

    let output = OutputInfo {
        info_ptr: info_ptr as usize,
        width: unsafe { png_get_image_width(png_ptr, info_ptr) },
        height: unsafe { png_get_image_height(png_ptr, info_ptr) },
        bit_depth: unsafe { png_get_bit_depth(png_ptr, info_ptr) },
        channels: unsafe { png_get_channels(png_ptr, info_ptr) },
        rowbytes: unsafe { png_get_rowbytes(png_ptr, info_ptr) },
    };
    update_read_state(png_ptr, |state| state.output = output);
    Some(output)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_check_for_invalid_index(png_ptr: png_structrp, allowed: c_int) {
    update_read_state(png_ptr, |state| {
        state.invalid_index_enabled = Some(allowed > 0);
        state.palette_max = if allowed > 0 { 0 } else { -1 };
    });
    unsafe {
        upstream_png_set_check_for_invalid_index(png_ptr, allowed);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_palette_max(
    png_ptr: png_const_structp,
    info_ptr: png_const_infop,
) -> c_int {
    let palette_max = unsafe { upstream_png_get_palette_max(png_ptr, info_ptr) };
    update_read_state(png_ptr.cast_mut(), |state| state.palette_max = palette_max);
    palette_max
}
