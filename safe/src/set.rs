use crate::chunks;
use crate::chunks::{read_core, read_info_core, write_core};
use crate::common::PNG_OPTION_INVALID;
use crate::read_util::PNG_HANDLE_CHUNK_LAST;
use crate::state;
use crate::types::*;
use core::ffi::c_int;

unsafe extern "C" {
    fn bridge_png_set_sig_bytes(png_ptr: png_structrp, num_bytes: c_int);
    fn bridge_png_set_rows(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        row_pointers: png_bytepp,
    );
    fn bridge_png_set_user_limits(
        png_ptr: png_structrp,
        user_width_max: png_uint_32,
        user_height_max: png_uint_32,
    );
    fn bridge_png_set_chunk_cache_max(png_ptr: png_structrp, user_chunk_cache_max: png_uint_32);
    fn bridge_png_set_chunk_malloc_max(
        png_ptr: png_structrp,
        user_chunk_malloc_max: png_alloc_size_t,
    );
    fn bridge_png_set_benign_errors(png_ptr: png_structrp, allowed: c_int);
    fn bridge_png_set_option(png_ptr: png_structrp, option: c_int, onoff: c_int) -> c_int;
    fn bridge_png_set_bKGD(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        background: png_const_color_16p,
    );
    fn bridge_png_set_cHRM(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        white_x: f64,
        white_y: f64,
        red_x: f64,
        red_y: f64,
        green_x: f64,
        green_y: f64,
        blue_x: f64,
        blue_y: f64,
    );
    fn bridge_png_set_cHRM_fixed(
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
    );
    fn bridge_png_set_eXIf(png_ptr: png_const_structrp, info_ptr: png_inforp, exif: png_bytep);
    fn bridge_png_set_eXIf_1(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        num_exif: png_uint_32,
        exif: png_bytep,
    );
    fn bridge_png_set_gAMA(png_ptr: png_const_structrp, info_ptr: png_inforp, file_gamma: f64);
    fn bridge_png_set_gAMA_fixed(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        file_gamma: png_fixed_point,
    );
    fn bridge_png_set_hIST(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        hist: png_const_uint_16p,
    );
    fn bridge_png_set_IHDR(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        width: png_uint_32,
        height: png_uint_32,
        bit_depth: c_int,
        color_type: c_int,
        interlace_method: c_int,
        compression_method: c_int,
        filter_method: c_int,
    );
    fn bridge_png_set_oFFs(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        offset_x: png_int_32,
        offset_y: png_int_32,
        unit_type: c_int,
    );
    fn bridge_png_set_pCAL(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        purpose: png_const_charp,
        x0: png_int_32,
        x1: png_int_32,
        kind: c_int,
        nparams: c_int,
        units: png_const_charp,
        params: png_charpp,
    );
    fn bridge_png_set_pHYs(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        res_x: png_uint_32,
        res_y: png_uint_32,
        unit_type: c_int,
    );
    fn bridge_png_set_PLTE(
        png_ptr: png_structrp,
        info_ptr: png_inforp,
        palette: png_const_colorp,
        num_palette: c_int,
    );
    fn bridge_png_set_sBIT(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        sig_bit: png_const_color_8p,
    );
    fn bridge_png_set_sRGB(png_ptr: png_const_structrp, info_ptr: png_inforp, srgb_intent: c_int);
    fn bridge_png_set_sRGB_gAMA_and_cHRM(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        srgb_intent: c_int,
    );
    fn bridge_png_set_iCCP(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        name: png_const_charp,
        compression_type: c_int,
        profile: png_const_bytep,
        proflen: png_uint_32,
    );
    fn bridge_png_set_sPLT(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        entries: png_const_sPLT_tp,
        nentries: c_int,
    );
    fn bridge_png_set_text(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        text_ptr: png_const_textp,
        num_text: c_int,
    );
    fn bridge_png_set_tIME(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        mod_time: png_const_timep,
    );
    fn bridge_png_set_tRNS(
        png_ptr: png_structrp,
        info_ptr: png_inforp,
        trans_alpha: png_const_bytep,
        num_trans: c_int,
        trans_color: png_const_color_16p,
    );
    fn bridge_png_set_sCAL(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        unit: c_int,
        width: f64,
        height: f64,
    );
    fn bridge_png_set_sCAL_fixed(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        unit: c_int,
        width: png_fixed_point,
        height: png_fixed_point,
    );
    fn bridge_png_set_sCAL_s(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        unit: c_int,
        swidth: png_const_charp,
        sheight: png_const_charp,
    );
}

fn sync_info_registry(info_ptr: png_inforp) {
    if info_ptr.is_null() {
        return;
    }

    let info_core = read_info_core(info_ptr);
    state::update_info(info_ptr, |state| {
        state.free_me = info_core.free_me;
        state.row_pointers = info_core.row_pointers;
    });
}

macro_rules! delegate_info_setter {
    ($(fn $name:ident(
        $png_ptr:ident : $png_ty:ty,
        $info_ptr:ident : $info_ty:ty
        $(, $arg:ident : $ty:ty)*
    ) => $upstream:ident;)+) => {
        $(
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn $name(
                $png_ptr: $png_ty,
                $info_ptr: $info_ty
                $(, $arg: $ty)*
            ) {
                crate::abi_guard!($png_ptr as png_structrp, unsafe {
                    $upstream($png_ptr, $info_ptr $(, $arg)*);
                    sync_info_registry($info_ptr);
                });
            }
        )+
    };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_sig_bytes(png_ptr: png_structrp, num_bytes: c_int) {
    crate::abi_guard!(png_ptr, unsafe {
        bridge_png_set_sig_bytes(png_ptr, num_bytes);
        state::update_png(png_ptr, |state| {
            state.sig_bytes = num_bytes.clamp(0, 8);
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_rows(
    png_ptr: png_const_structrp,
    info_ptr: png_inforp,
    row_pointers: png_bytepp,
) {
    crate::abi_guard!(png_ptr.cast_mut(), unsafe {
        bridge_png_set_rows(png_ptr, info_ptr, row_pointers);
        sync_info_registry(info_ptr);
    });
}

delegate_info_setter! {
    fn png_set_bKGD(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        background: png_const_color_16p
    ) => bridge_png_set_bKGD;
    fn png_set_cHRM(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        white_x: f64,
        white_y: f64,
        red_x: f64,
        red_y: f64,
        green_x: f64,
        green_y: f64,
        blue_x: f64,
        blue_y: f64
    ) => bridge_png_set_cHRM;
    fn png_set_cHRM_fixed(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        white_x: png_fixed_point,
        white_y: png_fixed_point,
        red_x: png_fixed_point,
        red_y: png_fixed_point,
        green_x: png_fixed_point,
        green_y: png_fixed_point,
        blue_x: png_fixed_point,
        blue_y: png_fixed_point
    ) => bridge_png_set_cHRM_fixed;
    fn png_set_eXIf(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        exif: png_bytep
    ) => bridge_png_set_eXIf;
    fn png_set_eXIf_1(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        num_exif: png_uint_32,
        exif: png_bytep
    ) => bridge_png_set_eXIf_1;
    fn png_set_gAMA(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        file_gamma: f64
    ) => bridge_png_set_gAMA;
    fn png_set_gAMA_fixed(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        file_gamma: png_fixed_point
    ) => bridge_png_set_gAMA_fixed;
    fn png_set_hIST(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        hist: png_const_uint_16p
    ) => bridge_png_set_hIST;
    fn png_set_IHDR(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        width: png_uint_32,
        height: png_uint_32,
        bit_depth: c_int,
        color_type: c_int,
        interlace_method: c_int,
        compression_method: c_int,
        filter_method: c_int
    ) => bridge_png_set_IHDR;
    fn png_set_oFFs(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        offset_x: png_int_32,
        offset_y: png_int_32,
        unit_type: c_int
    ) => bridge_png_set_oFFs;
    fn png_set_pCAL(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        purpose: png_const_charp,
        x0: png_int_32,
        x1: png_int_32,
        kind: c_int,
        nparams: c_int,
        units: png_const_charp,
        params: png_charpp
    ) => bridge_png_set_pCAL;
    fn png_set_pHYs(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        res_x: png_uint_32,
        res_y: png_uint_32,
        unit_type: c_int
    ) => bridge_png_set_pHYs;
    fn png_set_PLTE(
        png_ptr: png_structrp,
        info_ptr: png_inforp,
        palette: png_const_colorp,
        num_palette: c_int
    ) => bridge_png_set_PLTE;
    fn png_set_sBIT(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        sig_bit: png_const_color_8p
    ) => bridge_png_set_sBIT;
    fn png_set_sRGB(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        srgb_intent: c_int
    ) => bridge_png_set_sRGB;
    fn png_set_sRGB_gAMA_and_cHRM(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        srgb_intent: c_int
    ) => bridge_png_set_sRGB_gAMA_and_cHRM;
    fn png_set_iCCP(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        name: png_const_charp,
        compression_type: c_int,
        profile: png_const_bytep,
        proflen: png_uint_32
    ) => bridge_png_set_iCCP;
    fn png_set_sPLT(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        entries: png_const_sPLT_tp,
        nentries: c_int
    ) => bridge_png_set_sPLT;
    fn png_set_text(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        text_ptr: png_const_textp,
        num_text: c_int
    ) => bridge_png_set_text;
    fn png_set_tIME(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        mod_time: png_const_timep
    ) => bridge_png_set_tIME;
    fn png_set_tRNS(
        png_ptr: png_structrp,
        info_ptr: png_inforp,
        trans_alpha: png_const_bytep,
        num_trans: c_int,
        trans_color: png_const_color_16p
    ) => bridge_png_set_tRNS;
    fn png_set_sCAL(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        unit: c_int,
        width: f64,
        height: f64
    ) => bridge_png_set_sCAL;
    fn png_set_sCAL_fixed(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        unit: c_int,
        width: png_fixed_point,
        height: png_fixed_point
    ) => bridge_png_set_sCAL_fixed;
    fn png_set_sCAL_s(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        unit: c_int,
        swidth: png_const_charp,
        sheight: png_const_charp
    ) => bridge_png_set_sCAL_s;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_user_limits(
    png_ptr: png_structrp,
    user_width_max: png_uint_32,
    user_height_max: png_uint_32,
) {
    crate::abi_guard!(png_ptr, unsafe {
        bridge_png_set_user_limits(png_ptr, user_width_max, user_height_max);
        state::update_png(png_ptr, |state| {
            state.user_width_max = user_width_max;
            state.user_height_max = user_height_max;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_chunk_cache_max(
    png_ptr: png_structrp,
    user_chunk_cache_max: png_uint_32,
) {
    crate::abi_guard!(png_ptr, unsafe {
        bridge_png_set_chunk_cache_max(png_ptr, user_chunk_cache_max);
        state::update_png(png_ptr, |state| {
            state.user_chunk_cache_max = user_chunk_cache_max;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_chunk_malloc_max(
    png_ptr: png_structrp,
    user_chunk_malloc_max: png_alloc_size_t,
) {
    crate::abi_guard!(png_ptr, unsafe {
        bridge_png_set_chunk_malloc_max(png_ptr, user_chunk_malloc_max);
        state::update_png(png_ptr, |state| {
            state.user_chunk_malloc_max = user_chunk_malloc_max;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_keep_unknown_chunks(
    png_ptr: png_structrp,
    keep: c_int,
    chunk_list: png_const_bytep,
    num_chunks_in: c_int,
) {
    crate::abi_guard!(png_ptr, {
        if png_ptr.is_null() {
            return;
        }

        if !(0..PNG_HANDLE_CHUNK_LAST).contains(&keep) {
            unsafe {
                let _ =
                    chunks::call_app_error(png_ptr, b"png_set_keep_unknown_chunks: invalid keep\0");
            }
            return;
        }

        if num_chunks_in > 0 && chunk_list.is_null() {
            unsafe {
                let _ = chunks::call_app_error(
                    png_ptr,
                    b"png_set_keep_unknown_chunks: no chunk list\0",
                );
            }
            return;
        }

        let existing = state::get_png(png_ptr)
            .map(|png_state| png_state.unknown_chunk_list.len())
            .unwrap_or(0);
        let requested = if num_chunks_in < 0 {
            crate::read_util::known_chunks_to_ignore().len()
        } else {
            usize::try_from(num_chunks_in).unwrap_or(usize::MAX)
        };
        let max_chunks = (u32::MAX as usize) / 5;
        if requested.saturating_add(existing) > max_chunks {
            unsafe {
                let _ = chunks::call_app_error(
                    png_ptr,
                    b"png_set_keep_unknown_chunks: too many chunks\0",
                );
            }
            return;
        }

        chunks::apply_keep_unknown_chunks_state(png_ptr, keep, chunk_list, num_chunks_in);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_benign_errors(png_ptr: png_structrp, allowed: c_int) {
    crate::abi_guard!(png_ptr, unsafe {
        bridge_png_set_benign_errors(png_ptr, allowed);
        state::update_png(png_ptr, |state| {
            state.benign_errors = if allowed != 0 { 1 } else { 0 };
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_check_for_invalid_index(png_ptr: png_structrp, allowed: c_int) {
    crate::abi_guard!(png_ptr, {
        let mut core = read_core(png_ptr);
        core.num_palette_max = if allowed > 0 { 0 } else { -1 };
        write_core(png_ptr, &core);
        state::update_png(png_ptr, |state| {
            state.check_for_invalid_index = if allowed > 0 { 1 } else { 0 };
            state.palette_max = core.num_palette_max;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_option(
    png_ptr: png_structrp,
    option: c_int,
    onoff: c_int,
) -> c_int {
    crate::abi_guard!(png_ptr, unsafe {
        let result = bridge_png_set_option(png_ptr, option, onoff);
        if result != PNG_OPTION_INVALID {
            state::update_png(png_ptr, |state| {
                let mask = 3u32 << option;
                let setting = (2u32 + u32::from(onoff != 0)) << option;
                state.options = (state.options & !mask) | setting;
            });
        }
        result
    })
}
