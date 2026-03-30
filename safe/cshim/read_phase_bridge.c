#include <setjmp.h>
#include <string.h>

#include "pngpriv.h"

typedef struct png_safe_read_core {
    png_uint_32 mode;
    png_uint_32 flags;
    png_uint_32 transformations;
    png_uint_32 width;
    png_uint_32 height;
    size_t rowbytes;
    size_t info_rowbytes;
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
    int num_palette_max;
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

extern void upstream_png_set_quantize(png_structrp png_ptr, png_colorp palette,
                                      int num_palette, int maximum_colors,
                                      png_const_uint_16p histogram,
                                      int full_quantize);

static void png_safe_ignore_warning(png_structp png_ptr,
                                    png_const_charp message) {
    (void)png_ptr;
    (void)message;
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
    out->rowbytes = png_ptr->rowbytes;
    out->info_rowbytes = png_ptr->info_rowbytes;
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
    out->num_palette_max = png_ptr->num_palette_max;
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
    png_ptr->rowbytes = in->rowbytes;
    png_ptr->info_rowbytes = in->info_rowbytes;
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
    png_ptr->num_palette_max = in->num_palette_max;
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

int png_safe_call_read_info(png_structrp png_ptr, png_inforp info_ptr) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    png_read_info(png_ptr, info_ptr);
    return 1;
}

int png_safe_call_read_update_info(png_structrp png_ptr, png_inforp info_ptr) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    png_read_update_info(png_ptr, info_ptr);
    return 1;
}

int png_safe_call_read_image(png_structrp png_ptr, png_bytepp image) {
    png_error_ptr saved_warning_fn = NULL;
    int suppress_interlace_warning = 0;

    if (png_ptr != NULL &&
        (png_ptr->flags & PNG_FLAG_ROW_INIT) != 0 &&
        png_ptr->interlaced != 0 &&
        (png_ptr->transformations & PNG_INTERLACE) == 0) {
        saved_warning_fn = png_ptr->warning_fn;
        png_ptr->warning_fn = png_safe_ignore_warning;
        suppress_interlace_warning = 1;
    }

    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        if (suppress_interlace_warning != 0 && png_ptr != NULL) {
            png_ptr->warning_fn = saved_warning_fn;
        }
        return 0;
    }

    png_read_image(png_ptr, image);

    if (suppress_interlace_warning != 0 && png_ptr != NULL) {
        png_ptr->warning_fn = saved_warning_fn;
    }
    return 1;
}

int png_safe_call_read_end(png_structrp png_ptr, png_inforp info_ptr) {
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        return 0;
    }

    png_read_end(png_ptr, info_ptr);
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
