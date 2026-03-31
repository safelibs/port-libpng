#include <setjmp.h>
#include <stddef.h>

#include "pngpriv.h"

extern void upstream_png_set_quantize(png_structrp png_ptr, png_colorp palette,
                                      int num_palette, int maximum_colors,
                                      png_const_uint_16p histogram,
                                      int full_quantize);
extern void upstream_png_read_row(png_structrp png_ptr, png_bytep row,
                                  png_bytep display_row);

size_t png_safe_longjmp_state_size(void) {
    return sizeof(jmp_buf);
}

jmp_buf *png_safe_longjmp_local_buffer(png_structrp png_ptr) {
    if (png_ptr == NULL) {
        return NULL;
    }

    return &png_ptr->jmp_buf_local;
}

jmp_buf *png_safe_longjmp_get_buffer(png_const_structrp png_ptr) {
    if (png_ptr == NULL) {
        return NULL;
    }

    return png_ptr->jmp_buf_ptr;
}

size_t png_safe_longjmp_get_size(png_const_structrp png_ptr) {
    if (png_ptr == NULL) {
        return 0;
    }

    return png_ptr->jmp_buf_size;
}

void png_safe_longjmp_set_fields(png_structrp png_ptr, png_longjmp_ptr longjmp_fn,
                                 jmp_buf *jmp_buf_ptr, size_t jmp_buf_size) {
    if (png_ptr == NULL) {
        return;
    }

    png_ptr->longjmp_fn = longjmp_fn;
    png_ptr->jmp_buf_ptr = jmp_buf_ptr;
    png_ptr->jmp_buf_size = jmp_buf_size;
}

void png_safe_longjmp_call(png_const_structrp png_ptr, int val) {
    if (png_ptr == NULL || png_ptr->longjmp_fn == NULL || png_ptr->jmp_buf_ptr == NULL) {
        return;
    }

    png_ptr->longjmp_fn(*png_ptr->jmp_buf_ptr, val);
}

int png_safe_call_read_data(png_structrp png_ptr, png_bytep buffer, size_t size) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    png_read_data(png_ptr, buffer, size);
    return 1;
}

int png_safe_prepare_idat(png_structrp png_ptr, png_uint_32 length) {
    static const png_byte idat_name[4] = {'I', 'D', 'A', 'T'};

    if (png_ptr == NULL) {
        return 0;
    }

    png_ptr->chunk_name = png_IDAT;
    png_ptr->idat_size = length;
#ifdef PNG_IO_STATE_SUPPORTED
    png_ptr->io_state = PNG_IO_READING | PNG_IO_CHUNK_DATA;
#endif
    png_reset_crc(png_ptr);
    png_calculate_crc(png_ptr, (png_bytep)idat_name, 4);
    return 1;
}

int png_safe_complete_idat(png_structrp png_ptr) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    png_read_finish_IDAT(png_ptr);
    return 1;
}

void png_safe_resume_finish_idat(png_structrp png_ptr) {
    if (png_ptr == NULL) {
        return;
    }

    if ((png_ptr->flags & PNG_FLAG_ZSTREAM_ENDED) != 0 && png_ptr->zowner == 0) {
        png_ptr->zowner = png_IDAT;
    }
}

int png_safe_call_read_row(png_structrp png_ptr, png_bytep row, png_bytep display_row) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    upstream_png_read_row(png_ptr, row, display_row);
    return 1;
}

int png_safe_call_read_start_row(png_structrp png_ptr) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    png_read_start_row(png_ptr);
    return 1;
}

int png_safe_call_read_transform_info(png_structrp png_ptr, png_inforp info_ptr) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    png_read_transform_info(png_ptr, info_ptr);
    return 1;
}

#define PNG_SAFE_WRAP_SETTER(fn, args, call) \
int fn args { \
    if (setjmp(png_jmpbuf((png_structrp)png_ptr)) != 0) { \
        return 0; \
    } \
    call; \
    return 1; \
}

PNG_SAFE_WRAP_SETTER(
    png_safe_set_IHDR,
    (png_structrp png_ptr, png_inforp info_ptr, png_uint_32 width,
     png_uint_32 height, int bit_depth, int color_type, int interlace_type,
     int compression_type, int filter_type),
    png_set_IHDR(png_ptr, info_ptr, width, height, bit_depth, color_type,
                 interlace_type, compression_type, filter_type))

PNG_SAFE_WRAP_SETTER(
    png_safe_set_PLTE,
    (png_structrp png_ptr, png_inforp info_ptr, png_colorp palette,
     int num_palette),
    png_set_PLTE(png_ptr, info_ptr, palette, num_palette))

PNG_SAFE_WRAP_SETTER(
    png_safe_set_tRNS,
    (png_structrp png_ptr, png_inforp info_ptr, png_bytep trans_alpha,
     int num_trans, png_color_16p trans_color),
    png_set_tRNS(png_ptr, info_ptr, trans_alpha, num_trans, trans_color))

PNG_SAFE_WRAP_SETTER(
    png_safe_set_bKGD,
    (png_const_structrp png_ptr, png_inforp info_ptr, png_color_16p background),
    png_set_bKGD(png_ptr, info_ptr, background))

PNG_SAFE_WRAP_SETTER(
    png_safe_set_cHRM_fixed,
    (png_const_structrp png_ptr, png_inforp info_ptr, png_fixed_point white_x,
     png_fixed_point white_y, png_fixed_point red_x, png_fixed_point red_y,
     png_fixed_point green_x, png_fixed_point green_y, png_fixed_point blue_x,
     png_fixed_point blue_y),
    png_set_cHRM_fixed(png_ptr, info_ptr, white_x, white_y, red_x, red_y,
                       green_x, green_y, blue_x, blue_y))

PNG_SAFE_WRAP_SETTER(
    png_safe_set_eXIf_1,
    (png_const_structrp png_ptr, png_inforp info_ptr, png_uint_32 num_exif,
     png_bytep exif),
    png_set_eXIf_1(png_ptr, info_ptr, num_exif, exif))

PNG_SAFE_WRAP_SETTER(
    png_safe_set_gAMA_fixed,
    (png_const_structrp png_ptr, png_inforp info_ptr, png_fixed_point file_gamma),
    png_set_gAMA_fixed(png_ptr, info_ptr, file_gamma))

PNG_SAFE_WRAP_SETTER(
    png_safe_set_hIST,
    (png_const_structrp png_ptr, png_inforp info_ptr, png_const_uint_16p hist),
    png_set_hIST(png_ptr, info_ptr, hist))

PNG_SAFE_WRAP_SETTER(
    png_safe_set_oFFs,
    (png_const_structrp png_ptr, png_inforp info_ptr, png_int_32 offset_x,
     png_int_32 offset_y, int unit_type),
    png_set_oFFs(png_ptr, info_ptr, offset_x, offset_y, unit_type))

PNG_SAFE_WRAP_SETTER(
    png_safe_set_pCAL,
    (png_const_structrp png_ptr, png_inforp info_ptr, png_charp purpose,
     png_int_32 X0, png_int_32 X1, int type, int nparams, png_charp units,
     png_charpp params),
    png_set_pCAL(png_ptr, info_ptr, purpose, X0, X1, type, nparams, units,
                 params))

PNG_SAFE_WRAP_SETTER(
    png_safe_set_pHYs,
    (png_const_structrp png_ptr, png_inforp info_ptr, png_uint_32 res_x,
     png_uint_32 res_y, int unit_type),
    png_set_pHYs(png_ptr, info_ptr, res_x, res_y, unit_type))

PNG_SAFE_WRAP_SETTER(
    png_safe_set_sBIT,
    (png_const_structrp png_ptr, png_inforp info_ptr, png_color_8p sig_bit),
    png_set_sBIT(png_ptr, info_ptr, sig_bit))

PNG_SAFE_WRAP_SETTER(
    png_safe_set_sCAL_s,
    (png_const_structrp png_ptr, png_inforp info_ptr, int unit,
     png_const_charp swidth, png_const_charp sheight),
    png_set_sCAL_s(png_ptr, info_ptr, unit, swidth, sheight))

PNG_SAFE_WRAP_SETTER(
    png_safe_set_sPLT,
    (png_const_structrp png_ptr, png_inforp info_ptr, png_sPLT_tp entries,
     int num_entries),
    png_set_sPLT(png_ptr, info_ptr, entries, num_entries))

PNG_SAFE_WRAP_SETTER(
    png_safe_set_sRGB,
    (png_const_structrp png_ptr, png_inforp info_ptr, int srgb_intent),
    png_set_sRGB(png_ptr, info_ptr, srgb_intent))

PNG_SAFE_WRAP_SETTER(
    png_safe_set_iCCP,
    (png_const_structrp png_ptr, png_inforp info_ptr, png_const_charp name,
     int compression_type, png_const_bytep profile, png_uint_32 proflen),
    png_set_iCCP(png_ptr, info_ptr, name, compression_type, profile, proflen))

PNG_SAFE_WRAP_SETTER(
    png_safe_set_text,
    (png_const_structrp png_ptr, png_inforp info_ptr, png_textp text_ptr,
     int num_text),
    png_set_text(png_ptr, info_ptr, text_ptr, num_text))

PNG_SAFE_WRAP_SETTER(
    png_safe_set_tIME,
    (png_const_structrp png_ptr, png_inforp info_ptr, png_timep mod_time),
    png_set_tIME(png_ptr, info_ptr, mod_time))

PNG_SAFE_WRAP_SETTER(
    png_safe_set_unknown_chunks,
    (png_const_structrp png_ptr, png_inforp info_ptr, png_unknown_chunkp unknowns,
     int num_unknowns),
    png_set_unknown_chunks(png_ptr, info_ptr, unknowns, num_unknowns))

PNG_SAFE_WRAP_SETTER(
    png_safe_set_unknown_chunk_location,
    (png_const_structrp png_ptr, png_inforp info_ptr, int chunk, int location),
    png_set_unknown_chunk_location(png_ptr, info_ptr, chunk, location))

#undef PNG_SAFE_WRAP_SETTER

int png_safe_call_warning(png_structrp png_ptr, png_const_charp message) {
    png_warning(png_ptr, message);
    return 1;
}

int png_safe_call_benign_error(png_structrp png_ptr, png_const_charp message) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    png_benign_error(png_ptr, message);
    return 1;
}

int png_safe_call_app_error(png_structrp png_ptr, png_const_charp message) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    png_app_error(png_ptr, message);
    return 1;
}

int png_safe_call_error(png_structrp png_ptr, png_const_charp message) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    png_error(png_ptr, message);
    return 0;
}

int png_safe_call_set_quantize(png_structrp png_ptr, png_colorp palette,
                               int num_palette, int maximum_colors,
                               png_const_uint_16p histogram,
                               int full_quantize) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    upstream_png_set_quantize(png_ptr, palette, num_palette, maximum_colors,
                              histogram, full_quantize);
    return 1;
}
