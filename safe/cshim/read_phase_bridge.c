#include <stdlib.h>
#include <setjmp.h>
#include <string.h>

#include "pngpriv.h"

typedef struct png_safe_read_core {
    png_uint_32 mode;
    png_uint_32 flags;
    png_uint_32 transformations;
    png_uint_32 width;
    png_uint_32 height;
    png_uint_32 num_rows;
    png_uint_32 row_number;
    png_uint_32 chunk_name;
    png_uint_32 idat_size;
    size_t rowbytes;
    size_t info_rowbytes;
    size_t save_buffer_size;
    size_t buffer_size;
    size_t current_buffer_size;
    png_byte interlaced;
    png_byte color_type;
    png_byte bit_depth;
    png_byte pixel_depth;
    png_byte transformed_pixel_depth;
    png_byte channels;
    png_byte compression_type;
    png_byte filter_type;
    png_byte background_gamma_type;
    png_fixed_point background_gamma;
    png_fixed_point screen_gamma;
    png_color_16 background;
    png_color_8 shift;
    png_colorspace colorspace;
    png_byte rgb_to_gray_status;
    png_byte rgb_to_gray_coefficients_set;
    png_uint_16 rgb_to_gray_red_coeff;
    png_uint_16 rgb_to_gray_green_coeff;
    int pass;
    int process_mode;
    int num_palette_max;
    int unknown_default;
    png_uint_32 num_chunk_list;
} png_safe_read_core;

typedef struct png_safe_info_core {
    png_uint_32 width;
    png_uint_32 height;
    png_uint_32 valid;
    size_t rowbytes;
    png_uint_16 num_palette;
    png_uint_16 num_trans;
    png_byte bit_depth;
    png_byte color_type;
    png_byte compression_type;
    png_byte filter_type;
    png_byte interlace_type;
    png_byte channels;
    png_byte pixel_depth;
    png_color_16 background;
    png_color_8 sig_bit;
    png_color_16 trans_color;
    png_colorspace colorspace;
    png_bytepp row_pointers;
    png_uint_32 free_me;
} png_safe_info_core;

typedef struct png_safe_parse_snapshot {
    int has_png;
    png_struct png;
    int has_info;
    png_info info;
} png_safe_parse_snapshot;

extern void upstream_png_set_quantize(png_structrp png_ptr, png_colorp palette,
                                      int num_palette, int maximum_colors,
                                      png_const_uint_16p histogram,
                                      int full_quantize);
extern void upstream_png_read_row(png_structrp png_ptr, png_bytep row,
                                  png_bytep display_row);

void *png_safe_parse_snapshot_capture(png_const_structrp png_ptr,
                                      png_const_inforp info_ptr) {
    png_safe_parse_snapshot *snapshot =
        (png_safe_parse_snapshot *)calloc(1, sizeof(*snapshot));

    if (snapshot == NULL) {
        return NULL;
    }

    if (png_ptr != NULL) {
        snapshot->has_png = 1;
        snapshot->png = *png_ptr;
    }

    if (info_ptr != NULL) {
        snapshot->has_info = 1;
        snapshot->info = *info_ptr;
    }

    return snapshot;
}

void png_safe_parse_snapshot_restore(png_structrp png_ptr, png_inforp info_ptr,
                                     const void *snapshot_ptr) {
    const png_safe_parse_snapshot *snapshot =
        (const png_safe_parse_snapshot *)snapshot_ptr;

    if (snapshot == NULL) {
        return;
    }

    if (png_ptr != NULL && snapshot->has_png) {
        *png_ptr = snapshot->png;
    }

    if (info_ptr != NULL && snapshot->has_info) {
        *info_ptr = snapshot->info;
    }
}

void png_safe_parse_snapshot_free(void *snapshot_ptr) {
    free(snapshot_ptr);
}

void png_safe_read_core_get(png_const_structrp png_ptr, png_safe_read_core *out) {
    memset(out, 0, sizeof(*out));
    if (png_ptr == NULL) {
        return;
    }

    out->mode = png_ptr->mode;
    out->flags = png_ptr->flags;
    out->transformations = png_ptr->transformations;
    out->width = png_ptr->width;
    out->height = png_ptr->height;
    out->num_rows = png_ptr->num_rows;
    out->row_number = png_ptr->row_number;
    out->chunk_name = png_ptr->chunk_name;
    out->idat_size = png_ptr->idat_size;
    out->rowbytes = png_ptr->rowbytes;
    out->info_rowbytes = png_ptr->info_rowbytes;
    out->save_buffer_size = png_ptr->save_buffer_size;
    out->buffer_size = png_ptr->buffer_size;
    out->current_buffer_size = png_ptr->current_buffer_size;
    out->interlaced = png_ptr->interlaced;
    out->color_type = png_ptr->color_type;
    out->bit_depth = png_ptr->bit_depth;
    out->pixel_depth = png_ptr->pixel_depth;
    out->transformed_pixel_depth = png_ptr->transformed_pixel_depth;
    out->channels = png_ptr->channels;
    out->compression_type = png_ptr->compression_type;
    out->filter_type = png_ptr->filter_type;
    out->background_gamma_type = png_ptr->background_gamma_type;
    out->background_gamma = png_ptr->background_gamma;
    out->screen_gamma = png_ptr->screen_gamma;
    out->background = png_ptr->background;
    out->shift = png_ptr->shift;
    out->colorspace = png_ptr->colorspace;
    out->rgb_to_gray_status = png_ptr->rgb_to_gray_status;
    out->rgb_to_gray_coefficients_set = png_ptr->rgb_to_gray_coefficients_set;
    out->rgb_to_gray_red_coeff = png_ptr->rgb_to_gray_red_coeff;
    out->rgb_to_gray_green_coeff = png_ptr->rgb_to_gray_green_coeff;
    out->pass = png_ptr->pass;
    out->process_mode = png_ptr->process_mode;
    out->num_palette_max = png_ptr->num_palette_max;
    out->unknown_default = png_ptr->unknown_default;
    out->num_chunk_list = png_ptr->num_chunk_list;
}

void png_safe_read_core_set(png_structrp png_ptr, const png_safe_read_core *in) {
    if (png_ptr == NULL || in == NULL) {
        return;
    }

    png_ptr->mode = in->mode;
    png_ptr->flags = in->flags;
    png_ptr->transformations = in->transformations;
    png_ptr->width = in->width;
    png_ptr->height = in->height;
    png_ptr->num_rows = in->num_rows;
    png_ptr->row_number = in->row_number;
    png_ptr->chunk_name = in->chunk_name;
    png_ptr->idat_size = in->idat_size;
    png_ptr->rowbytes = in->rowbytes;
    png_ptr->info_rowbytes = in->info_rowbytes;
    png_ptr->save_buffer_size = in->save_buffer_size;
    png_ptr->buffer_size = in->buffer_size;
    png_ptr->current_buffer_size = in->current_buffer_size;
    png_ptr->interlaced = in->interlaced;
    png_ptr->color_type = in->color_type;
    png_ptr->bit_depth = in->bit_depth;
    png_ptr->pixel_depth = in->pixel_depth;
    png_ptr->transformed_pixel_depth = in->transformed_pixel_depth;
    png_ptr->channels = in->channels;
    png_ptr->compression_type = in->compression_type;
    png_ptr->filter_type = in->filter_type;
    png_ptr->background_gamma_type = in->background_gamma_type;
    png_ptr->background_gamma = in->background_gamma;
    png_ptr->screen_gamma = in->screen_gamma;
    png_ptr->background = in->background;
    png_ptr->shift = in->shift;
    png_ptr->colorspace = in->colorspace;
    png_ptr->rgb_to_gray_status = in->rgb_to_gray_status;
    png_ptr->rgb_to_gray_coefficients_set = in->rgb_to_gray_coefficients_set;
    png_ptr->rgb_to_gray_red_coeff = in->rgb_to_gray_red_coeff;
    png_ptr->rgb_to_gray_green_coeff = in->rgb_to_gray_green_coeff;
    png_ptr->pass = in->pass;
    png_ptr->process_mode = in->process_mode;
    png_ptr->num_palette_max = in->num_palette_max;
    png_ptr->unknown_default = in->unknown_default;
    png_ptr->num_chunk_list = in->num_chunk_list;
}

void png_safe_info_core_get(png_const_inforp info_ptr, png_safe_info_core *out) {
    memset(out, 0, sizeof(*out));
    if (info_ptr == NULL) {
        return;
    }

    out->width = info_ptr->width;
    out->height = info_ptr->height;
    out->valid = info_ptr->valid;
    out->rowbytes = info_ptr->rowbytes;
    out->num_palette = info_ptr->num_palette;
    out->num_trans = info_ptr->num_trans;
    out->bit_depth = info_ptr->bit_depth;
    out->color_type = info_ptr->color_type;
    out->compression_type = info_ptr->compression_type;
    out->filter_type = info_ptr->filter_type;
    out->interlace_type = info_ptr->interlace_type;
    out->channels = info_ptr->channels;
    out->pixel_depth = info_ptr->pixel_depth;
    out->background = info_ptr->background;
    out->sig_bit = info_ptr->sig_bit;
    out->trans_color = info_ptr->trans_color;
    out->colorspace = info_ptr->colorspace;
    out->row_pointers = info_ptr->row_pointers;
    out->free_me = info_ptr->free_me;
}

void png_safe_info_core_set(png_inforp info_ptr, const png_safe_info_core *in) {
    if (info_ptr == NULL || in == NULL) {
        return;
    }

    info_ptr->width = in->width;
    info_ptr->height = in->height;
    info_ptr->valid = in->valid;
    info_ptr->rowbytes = in->rowbytes;
    info_ptr->num_palette = in->num_palette;
    info_ptr->num_trans = in->num_trans;
    info_ptr->bit_depth = in->bit_depth;
    info_ptr->color_type = in->color_type;
    info_ptr->compression_type = in->compression_type;
    info_ptr->filter_type = in->filter_type;
    info_ptr->interlace_type = in->interlace_type;
    info_ptr->channels = in->channels;
    info_ptr->pixel_depth = in->pixel_depth;
    info_ptr->background = in->background;
    info_ptr->sig_bit = in->sig_bit;
    info_ptr->trans_color = in->trans_color;
    info_ptr->colorspace = in->colorspace;
    info_ptr->row_pointers = in->row_pointers;
    info_ptr->free_me = in->free_me;
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

    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
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

    if ((png_ptr->flags & PNG_FLAG_ZSTREAM_ENDED) == 0) {
        png_read_IDAT_data(png_ptr, NULL, 0);
        png_ptr->zstream.next_out = NULL;

        if ((png_ptr->flags & PNG_FLAG_ZSTREAM_ENDED) == 0) {
            png_ptr->mode |= PNG_AFTER_IDAT;
            png_ptr->flags |= PNG_FLAG_ZSTREAM_ENDED;
        }
    }

    if (png_ptr->zowner == png_IDAT) {
        png_ptr->zstream.next_in = NULL;
        png_ptr->zstream.avail_in = 0;
        png_ptr->zowner = 0;
        (void)png_crc_finish(png_ptr, png_ptr->idat_size);
    }

    return 1;
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
    if (setjmp(png_jmpbuf(png_ptr)) != 0) { \
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
