use crate::chunks::read_core;
use crate::state;
use crate::types::*;
use core::ffi::c_int;

unsafe extern "C" {
    fn upstream_png_get_valid(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        flag: png_uint_32,
    ) -> png_uint_32;
    fn upstream_png_get_rowbytes(png_ptr: png_const_structrp, info_ptr: png_const_inforp) -> usize;
    fn upstream_png_get_rows(png_ptr: png_const_structrp, info_ptr: png_const_inforp)
    -> png_bytepp;
    fn upstream_png_get_image_width(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_uint_32;
    fn upstream_png_get_image_height(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_uint_32;
    fn upstream_png_get_bit_depth(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_byte;
    fn upstream_png_get_color_type(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_byte;
    fn upstream_png_get_filter_type(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_byte;
    fn upstream_png_get_interlace_type(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_byte;
    fn upstream_png_get_compression_type(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_byte;
    fn upstream_png_get_channels(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_byte;
    fn upstream_png_get_user_width_max(png_ptr: png_const_structrp) -> png_uint_32;
    fn upstream_png_get_user_height_max(png_ptr: png_const_structrp) -> png_uint_32;
    fn upstream_png_get_chunk_cache_max(png_ptr: png_const_structrp) -> png_uint_32;
    fn upstream_png_get_chunk_malloc_max(png_ptr: png_const_structrp) -> png_alloc_size_t;
    fn upstream_png_get_bKGD(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        background: *mut png_color_16p,
    ) -> png_uint_32;
    fn upstream_png_get_cHRM(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        white_x: png_doublep,
        white_y: png_doublep,
        red_x: png_doublep,
        red_y: png_doublep,
        green_x: png_doublep,
        green_y: png_doublep,
        blue_x: png_doublep,
        blue_y: png_doublep,
    ) -> png_uint_32;
    fn upstream_png_get_cHRM_fixed(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        white_x: png_fixed_point_p,
        white_y: png_fixed_point_p,
        red_x: png_fixed_point_p,
        red_y: png_fixed_point_p,
        green_x: png_fixed_point_p,
        green_y: png_fixed_point_p,
        blue_x: png_fixed_point_p,
        blue_y: png_fixed_point_p,
    ) -> png_uint_32;
    fn upstream_png_get_eXIf(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        exif: *mut png_bytep,
    ) -> png_uint_32;
    fn upstream_png_get_eXIf_1(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        num_exif: *mut png_uint_32,
        exif: *mut png_bytep,
    ) -> png_uint_32;
    fn upstream_png_get_gAMA(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        file_gamma: png_doublep,
    ) -> png_uint_32;
    fn upstream_png_get_gAMA_fixed(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        file_gamma: png_fixed_point_p,
    ) -> png_uint_32;
    fn upstream_png_get_hIST(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        hist: *mut png_uint_16p,
    ) -> png_uint_32;
    fn upstream_png_get_IHDR(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        width: *mut png_uint_32,
        height: *mut png_uint_32,
        bit_depth: *mut c_int,
        color_type: *mut c_int,
        interlace_method: *mut c_int,
        compression_method: *mut c_int,
        filter_method: *mut c_int,
    ) -> png_uint_32;
    fn upstream_png_get_oFFs(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        offset_x: *mut png_int_32,
        offset_y: *mut png_int_32,
        unit_type: *mut c_int,
    ) -> png_uint_32;
    fn upstream_png_get_pCAL(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        purpose: *mut png_charp,
        x0: *mut png_int_32,
        x1: *mut png_int_32,
        kind: *mut c_int,
        nparams: *mut c_int,
        units: *mut png_charp,
        params: *mut png_charpp,
    ) -> png_uint_32;
    fn upstream_png_get_pHYs(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        res_x: *mut png_uint_32,
        res_y: *mut png_uint_32,
        unit_type: *mut c_int,
    ) -> png_uint_32;
    fn upstream_png_get_PLTE(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        palette: *mut png_colorp,
        num_palette: *mut c_int,
    ) -> png_uint_32;
    fn upstream_png_get_sBIT(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        sig_bit: *mut png_color_8p,
    ) -> png_uint_32;
    fn upstream_png_get_sRGB(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        file_srgb_intent: *mut c_int,
    ) -> png_uint_32;
    fn upstream_png_get_iCCP(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        name: *mut png_charp,
        compression_type: *mut c_int,
        profile: *mut png_bytep,
        proflen: *mut png_uint_32,
    ) -> png_uint_32;
    fn upstream_png_get_sPLT(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        entries: png_sPLT_tpp,
    ) -> c_int;
    fn upstream_png_get_text(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        text_ptr: *mut png_textp,
        num_text: *mut c_int,
    ) -> c_int;
    fn upstream_png_get_tIME(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        mod_time: *mut png_timep,
    ) -> png_uint_32;
    fn upstream_png_get_tRNS(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        trans_alpha: *mut png_bytep,
        num_trans: *mut c_int,
        trans_color: *mut png_color_16p,
    ) -> png_uint_32;
    fn upstream_png_get_sCAL(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        unit: *mut c_int,
        width: png_doublep,
        height: png_doublep,
    ) -> png_uint_32;
    fn upstream_png_get_sCAL_fixed(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        unit: *mut c_int,
        width: png_fixed_point_p,
        height: png_fixed_point_p,
    ) -> png_uint_32;
    fn upstream_png_get_sCAL_s(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        unit: *mut c_int,
        swidth: *mut png_charp,
        sheight: *mut png_charp,
    ) -> png_uint_32;
}

macro_rules! delegate_getter {
    ($(fn $name:ident(
        $png_ptr:ident : $png_ty:ty
        $(, $arg:ident : $ty:ty)*
        $(,)?
    ) -> $ret:ty => $upstream:ident;)+) => {
        $(
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn $name($png_ptr: $png_ty $(, $arg: $ty)*) -> $ret {
                crate::abi_guard!($png_ptr as png_structrp, unsafe {
                    $upstream($png_ptr $(, $arg)*)
                })
            }
        )+
    };
}

delegate_getter! {
    fn png_get_valid(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        flag: png_uint_32,
    ) -> png_uint_32 => upstream_png_get_valid;
    fn png_get_rowbytes(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> usize => upstream_png_get_rowbytes;
    fn png_get_image_width(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_uint_32 => upstream_png_get_image_width;
    fn png_get_image_height(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_uint_32 => upstream_png_get_image_height;
    fn png_get_bit_depth(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_byte => upstream_png_get_bit_depth;
    fn png_get_color_type(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_byte => upstream_png_get_color_type;
    fn png_get_filter_type(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_byte => upstream_png_get_filter_type;
    fn png_get_interlace_type(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_byte => upstream_png_get_interlace_type;
    fn png_get_compression_type(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_byte => upstream_png_get_compression_type;
    fn png_get_channels(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
    ) -> png_byte => upstream_png_get_channels;
    fn png_get_bKGD(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        background: *mut png_color_16p,
    ) -> png_uint_32 => upstream_png_get_bKGD;
    fn png_get_cHRM(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        white_x: png_doublep,
        white_y: png_doublep,
        red_x: png_doublep,
        red_y: png_doublep,
        green_x: png_doublep,
        green_y: png_doublep,
        blue_x: png_doublep,
        blue_y: png_doublep,
    ) -> png_uint_32 => upstream_png_get_cHRM;
    fn png_get_cHRM_fixed(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        white_x: png_fixed_point_p,
        white_y: png_fixed_point_p,
        red_x: png_fixed_point_p,
        red_y: png_fixed_point_p,
        green_x: png_fixed_point_p,
        green_y: png_fixed_point_p,
        blue_x: png_fixed_point_p,
        blue_y: png_fixed_point_p,
    ) -> png_uint_32 => upstream_png_get_cHRM_fixed;
    fn png_get_eXIf(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        exif: *mut png_bytep,
    ) -> png_uint_32 => upstream_png_get_eXIf;
    fn png_get_eXIf_1(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        num_exif: *mut png_uint_32,
        exif: *mut png_bytep,
    ) -> png_uint_32 => upstream_png_get_eXIf_1;
    fn png_get_gAMA(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        file_gamma: png_doublep,
    ) -> png_uint_32 => upstream_png_get_gAMA;
    fn png_get_gAMA_fixed(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        file_gamma: png_fixed_point_p,
    ) -> png_uint_32 => upstream_png_get_gAMA_fixed;
    fn png_get_hIST(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        hist: *mut png_uint_16p,
    ) -> png_uint_32 => upstream_png_get_hIST;
    fn png_get_IHDR(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        width: *mut png_uint_32,
        height: *mut png_uint_32,
        bit_depth: *mut c_int,
        color_type: *mut c_int,
        interlace_method: *mut c_int,
        compression_method: *mut c_int,
        filter_method: *mut c_int,
    ) -> png_uint_32 => upstream_png_get_IHDR;
    fn png_get_oFFs(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        offset_x: *mut png_int_32,
        offset_y: *mut png_int_32,
        unit_type: *mut c_int,
    ) -> png_uint_32 => upstream_png_get_oFFs;
    fn png_get_pCAL(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        purpose: *mut png_charp,
        x0: *mut png_int_32,
        x1: *mut png_int_32,
        kind: *mut c_int,
        nparams: *mut c_int,
        units: *mut png_charp,
        params: *mut png_charpp,
    ) -> png_uint_32 => upstream_png_get_pCAL;
    fn png_get_pHYs(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        res_x: *mut png_uint_32,
        res_y: *mut png_uint_32,
        unit_type: *mut c_int,
    ) -> png_uint_32 => upstream_png_get_pHYs;
    fn png_get_PLTE(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        palette: *mut png_colorp,
        num_palette: *mut c_int,
    ) -> png_uint_32 => upstream_png_get_PLTE;
    fn png_get_sBIT(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        sig_bit: *mut png_color_8p,
    ) -> png_uint_32 => upstream_png_get_sBIT;
    fn png_get_sRGB(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        file_srgb_intent: *mut c_int,
    ) -> png_uint_32 => upstream_png_get_sRGB;
    fn png_get_iCCP(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        name: *mut png_charp,
        compression_type: *mut c_int,
        profile: *mut png_bytep,
        proflen: *mut png_uint_32,
    ) -> png_uint_32 => upstream_png_get_iCCP;
    fn png_get_sPLT(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        entries: png_sPLT_tpp,
    ) -> c_int => upstream_png_get_sPLT;
    fn png_get_text(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        text_ptr: *mut png_textp,
        num_text: *mut c_int,
    ) -> c_int => upstream_png_get_text;
    fn png_get_tIME(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        mod_time: *mut png_timep,
    ) -> png_uint_32 => upstream_png_get_tIME;
    fn png_get_tRNS(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        trans_alpha: *mut png_bytep,
        num_trans: *mut c_int,
        trans_color: *mut png_color_16p,
    ) -> png_uint_32 => upstream_png_get_tRNS;
    fn png_get_sCAL(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        unit: *mut c_int,
        width: png_doublep,
        height: png_doublep,
    ) -> png_uint_32 => upstream_png_get_sCAL;
    fn png_get_sCAL_fixed(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        unit: *mut c_int,
        width: png_fixed_point_p,
        height: png_fixed_point_p,
    ) -> png_uint_32 => upstream_png_get_sCAL_fixed;
    fn png_get_sCAL_s(
        png_ptr: png_const_structrp,
        info_ptr: png_const_inforp,
        unit: *mut c_int,
        swidth: *mut png_charp,
        sheight: *mut png_charp,
    ) -> png_uint_32 => upstream_png_get_sCAL_s;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_rows(
    png_ptr: png_const_structrp,
    info_ptr: png_const_inforp,
) -> png_bytepp {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if let Some(info_state) = state::get_info(info_ptr.cast_mut())
            && !info_state.row_pointers.is_null()
        {
            info_state.row_pointers
        } else {
            unsafe { upstream_png_get_rows(png_ptr, info_ptr) }
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_user_width_max(png_ptr: png_const_structrp) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        state::get_png(png_ptr.cast_mut())
            .map(|state| state.user_width_max)
            .unwrap_or_else(|| unsafe { upstream_png_get_user_width_max(png_ptr) })
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_user_height_max(png_ptr: png_const_structrp) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        state::get_png(png_ptr.cast_mut())
            .map(|state| state.user_height_max)
            .unwrap_or_else(|| unsafe { upstream_png_get_user_height_max(png_ptr) })
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_chunk_cache_max(png_ptr: png_const_structrp) -> png_uint_32 {
    crate::abi_guard!(png_ptr.cast_mut(), {
        state::get_png(png_ptr.cast_mut())
            .map(|state| state.user_chunk_cache_max)
            .unwrap_or_else(|| unsafe { upstream_png_get_chunk_cache_max(png_ptr) })
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_chunk_malloc_max(png_ptr: png_const_structrp) -> png_alloc_size_t {
    crate::abi_guard!(png_ptr.cast_mut(), {
        state::get_png(png_ptr.cast_mut())
            .map(|state| state.user_chunk_malloc_max)
            .unwrap_or_else(|| unsafe { upstream_png_get_chunk_malloc_max(png_ptr) })
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_palette_max(
    png_ptr: png_const_structp,
    info_ptr: png_const_infop,
) -> c_int {
    crate::abi_guard!(png_ptr.cast_mut(), {
        if png_ptr.is_null() || info_ptr.is_null() {
            -1
        } else {
            read_core(png_ptr.cast()).num_palette_max
        }
    })
}
