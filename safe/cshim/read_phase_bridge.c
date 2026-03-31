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
    png_byte transformed_pixel_depth;
    png_byte channels;
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

enum png_safe_chunk_dispatch {
    PNG_SAFE_CHUNK_IHDR = 1,
    PNG_SAFE_CHUNK_IEND = 2,
    PNG_SAFE_CHUNK_PLTE = 3,
    PNG_SAFE_CHUNK_BKGD = 4,
    PNG_SAFE_CHUNK_CHRM = 5,
    PNG_SAFE_CHUNK_EXIF = 6,
    PNG_SAFE_CHUNK_GAMA = 7,
    PNG_SAFE_CHUNK_HIST = 8,
    PNG_SAFE_CHUNK_OFFS = 9,
    PNG_SAFE_CHUNK_PCAL = 10,
    PNG_SAFE_CHUNK_SCAL = 11,
    PNG_SAFE_CHUNK_PHYS = 12,
    PNG_SAFE_CHUNK_SBIT = 13,
    PNG_SAFE_CHUNK_SRGB = 14,
    PNG_SAFE_CHUNK_ICCP = 15,
    PNG_SAFE_CHUNK_SPLT = 16,
    PNG_SAFE_CHUNK_TEXT = 17,
    PNG_SAFE_CHUNK_TIME = 18,
    PNG_SAFE_CHUNK_TRNS = 19,
    PNG_SAFE_CHUNK_ZTXT = 20,
    PNG_SAFE_CHUNK_ITXT = 21,
    PNG_SAFE_CHUNK_UNKNOWN = 22
};

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
    out->transformed_pixel_depth = png_ptr->transformed_pixel_depth;
    out->channels = png_ptr->channels;
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
    png_ptr->transformed_pixel_depth = in->transformed_pixel_depth;
    png_ptr->channels = in->channels;
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

int png_safe_call_read_sig(png_structrp png_ptr, png_inforp info_ptr) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    png_read_sig(png_ptr, info_ptr);
    return 1;
}

int png_safe_call_read_chunk_header(png_structrp png_ptr, png_uint_32 *length_out) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    *length_out = png_read_chunk_header(png_ptr);
    return 1;
}

int png_safe_call_read_row(png_structrp png_ptr, png_bytep row, png_bytep display_row) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    upstream_png_read_row(png_ptr, row, display_row);
    return 1;
}

int png_safe_call_dispatch_chunk(png_structrp png_ptr, png_inforp info_ptr,
                                 png_uint_32 length, int dispatch, int keep) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    switch (dispatch) {
        case PNG_SAFE_CHUNK_IHDR:
            png_handle_IHDR(png_ptr, info_ptr, length);
            break;
        case PNG_SAFE_CHUNK_IEND:
            png_handle_IEND(png_ptr, info_ptr, length);
            break;
        case PNG_SAFE_CHUNK_PLTE:
            png_handle_PLTE(png_ptr, info_ptr, length);
            break;
        case PNG_SAFE_CHUNK_BKGD:
            png_handle_bKGD(png_ptr, info_ptr, length);
            break;
        case PNG_SAFE_CHUNK_CHRM:
            png_handle_cHRM(png_ptr, info_ptr, length);
            break;
        case PNG_SAFE_CHUNK_EXIF:
            png_handle_eXIf(png_ptr, info_ptr, length);
            break;
        case PNG_SAFE_CHUNK_GAMA:
            png_handle_gAMA(png_ptr, info_ptr, length);
            break;
        case PNG_SAFE_CHUNK_HIST:
            png_handle_hIST(png_ptr, info_ptr, length);
            break;
        case PNG_SAFE_CHUNK_OFFS:
            png_handle_oFFs(png_ptr, info_ptr, length);
            break;
        case PNG_SAFE_CHUNK_PCAL:
            png_handle_pCAL(png_ptr, info_ptr, length);
            break;
        case PNG_SAFE_CHUNK_SCAL:
            png_handle_sCAL(png_ptr, info_ptr, length);
            break;
        case PNG_SAFE_CHUNK_PHYS:
            png_handle_pHYs(png_ptr, info_ptr, length);
            break;
        case PNG_SAFE_CHUNK_SBIT:
            png_handle_sBIT(png_ptr, info_ptr, length);
            break;
        case PNG_SAFE_CHUNK_SRGB:
            png_handle_sRGB(png_ptr, info_ptr, length);
            break;
        case PNG_SAFE_CHUNK_ICCP:
            png_handle_iCCP(png_ptr, info_ptr, length);
            break;
        case PNG_SAFE_CHUNK_SPLT:
            png_handle_sPLT(png_ptr, info_ptr, length);
            break;
        case PNG_SAFE_CHUNK_TEXT:
            png_handle_tEXt(png_ptr, info_ptr, length);
            break;
        case PNG_SAFE_CHUNK_TIME:
            png_handle_tIME(png_ptr, info_ptr, length);
            break;
        case PNG_SAFE_CHUNK_TRNS:
            png_handle_tRNS(png_ptr, info_ptr, length);
            break;
        case PNG_SAFE_CHUNK_ZTXT:
            png_handle_zTXt(png_ptr, info_ptr, length);
            break;
        case PNG_SAFE_CHUNK_ITXT:
            png_handle_iTXt(png_ptr, info_ptr, length);
            break;
        case PNG_SAFE_CHUNK_UNKNOWN:
            png_handle_unknown(png_ptr, info_ptr, length, keep);
            break;
        default:
            png_error(png_ptr, "invalid safe chunk dispatch");
            break;
    }

    return 1;
}

int png_safe_call_crc_finish(png_structrp png_ptr, png_uint_32 skip,
                             int *crc_result) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    *crc_result = png_crc_finish(png_ptr, skip);
    return 1;
}

int png_safe_call_read_finish_idat(png_structrp png_ptr) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    png_read_finish_IDAT(png_ptr);
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

int png_safe_call_push_restore_buffer(png_structrp png_ptr, png_bytep buffer,
                                      size_t buffer_size) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    png_push_restore_buffer(png_ptr, buffer, buffer_size);
    return 1;
}

int png_safe_call_push_read_sig(png_structrp png_ptr, png_inforp info_ptr) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    png_push_read_sig(png_ptr, info_ptr);
    return 1;
}

int png_safe_call_push_read_chunk(png_structrp png_ptr, png_inforp info_ptr) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    png_push_read_chunk(png_ptr, info_ptr);
    return 1;
}

int png_safe_call_push_read_idat(png_structrp png_ptr) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    png_push_read_IDAT(png_ptr);
    return 1;
}

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
