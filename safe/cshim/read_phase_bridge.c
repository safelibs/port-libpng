#include <stdlib.h>
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
    png_uint_32 zowner;
    png_uint_32 crc;
    png_uint_32 io_state;
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
    png_const_structrp alloc_png_ptr;
    int has_info;
    png_info info;
} png_safe_parse_snapshot;

static png_voidp
png_safe_snapshot_alloc(png_const_structrp png_ptr, size_t size) {
    png_voidp block = png_malloc_base(png_ptr, (png_alloc_size_t)size);
    if (block != NULL) {
        memset(block, 0, size);
    }
    return block;
}

static int
png_safe_snapshot_mul_size(size_t a, size_t b, size_t *out) {
    if (a != 0 && b > PNG_SIZE_MAX / a) {
        return 0;
    }

    *out = a * b;
    return 1;
}

static png_voidp
png_safe_snapshot_dup_bytes(png_const_structrp png_ptr, png_const_voidp src,
                            size_t size) {
    png_voidp dst;

    if (src == NULL || size == 0) {
        return NULL;
    }

    dst = png_malloc_base(png_ptr, (png_alloc_size_t)size);
    if (dst != NULL) {
        memcpy(dst, src, size);
    }

    return dst;
}

static png_charp
png_safe_snapshot_dup_string(png_const_structrp png_ptr, png_const_charp src) {
    size_t length;
    png_charp dst;

    if (src == NULL) {
        return NULL;
    }

    length = strlen(src) + 1;
    dst = png_voidcast(png_charp, png_malloc_base(png_ptr, (png_alloc_size_t)length));
    if (dst != NULL) {
        memcpy(dst, src, length);
    }

    return dst;
}

static int
png_safe_snapshot_clone_text(png_const_structrp png_ptr, png_info *info) {
#ifdef PNG_TEXT_SUPPORTED
    png_textp cloned_text;
    int count, i;
    size_t bytes;

    if (info->text == NULL || info->num_text <= 0) {
        info->text = NULL;
        info->num_text = 0;
        info->max_text = 0;
        return 1;
    }

    count = info->max_text;
    if (count < info->num_text) {
        count = info->num_text;
    }

    if (count <= 0 || !png_safe_snapshot_mul_size((size_t)count, sizeof(*cloned_text), &bytes)) {
        return 0;
    }

    cloned_text = png_voidcast(png_textp, png_safe_snapshot_alloc(png_ptr, bytes));
    if (cloned_text == NULL) {
        return 0;
    }

    for (i = 0; i < info->num_text; ++i) {
        png_const_textp src = info->text + i;
        png_textp dst = cloned_text + i;
        size_t key_len, lang_len, lang_key_len, text_len, total;
        png_charp block, cursor;

        *dst = *src;

        if (src->key == NULL) {
            dst->lang = NULL;
            dst->lang_key = NULL;
            dst->text = NULL;
            continue;
        }

        key_len = strlen(src->key) + 1;
        lang_len = 0;
        lang_key_len = 0;
        text_len = 1;

        if (src->compression > 0) {
            if (src->lang != NULL) {
                lang_len = strlen(src->lang) + 1;
            }
            if (src->lang_key != NULL) {
                lang_key_len = strlen(src->lang_key) + 1;
            }
            text_len = (size_t)src->itxt_length + 1;
        } else {
            text_len = (size_t)src->text_length + 1;
        }

        total = key_len + lang_len + lang_key_len + text_len;
        block = png_voidcast(png_charp, png_malloc_base(png_ptr, (png_alloc_size_t)total));
        if (block == NULL) {
            int j;

            for (j = 0; j < i; ++j) {
                png_free(png_ptr, cloned_text[j].key);
            }
            png_free(png_ptr, cloned_text);
            return 0;
        }

        cursor = block;
        memcpy(cursor, src->key, key_len);
        dst->key = cursor;
        cursor += key_len;

        if (src->compression > 0) {
            if (lang_len != 0) {
                memcpy(cursor, src->lang, lang_len);
                dst->lang = cursor;
                cursor += lang_len;
            } else {
                *cursor = 0;
                dst->lang = cursor;
                cursor += 1;
            }

            if (lang_key_len != 0) {
                memcpy(cursor, src->lang_key, lang_key_len);
                dst->lang_key = cursor;
                cursor += lang_key_len;
            } else {
                *cursor = 0;
                dst->lang_key = cursor;
                cursor += 1;
            }
        } else {
            dst->lang = NULL;
            dst->lang_key = NULL;
        }

        if (src->text != NULL) {
            memcpy(cursor, src->text, text_len);
        } else {
            *cursor = 0;
        }
        dst->text = cursor;
    }

    info->text = cloned_text;
    return 1;
#else
    PNG_UNUSED(png_ptr)
    PNG_UNUSED(info)
    return 1;
#endif
}

static int
png_safe_snapshot_clone_unknowns(png_const_structrp png_ptr, png_info *info) {
#ifdef PNG_STORE_UNKNOWN_CHUNKS_SUPPORTED
    png_unknown_chunkp cloned;
    int i;
    size_t bytes;

    if (info->unknown_chunks == NULL || info->unknown_chunks_num <= 0) {
        info->unknown_chunks = NULL;
        info->unknown_chunks_num = 0;
        return 1;
    }

    if (!png_safe_snapshot_mul_size((size_t)info->unknown_chunks_num, sizeof(*cloned), &bytes)) {
        return 0;
    }

    cloned = png_voidcast(png_unknown_chunkp, png_safe_snapshot_alloc(png_ptr, bytes));
    if (cloned == NULL) {
        return 0;
    }

    for (i = 0; i < info->unknown_chunks_num; ++i) {
        cloned[i] = info->unknown_chunks[i];
        if (info->unknown_chunks[i].size != 0 && info->unknown_chunks[i].data != NULL) {
            cloned[i].data = png_voidcast(
                png_bytep,
                png_safe_snapshot_dup_bytes(
                    png_ptr, info->unknown_chunks[i].data, info->unknown_chunks[i].size));
            if (cloned[i].data == NULL) {
                int j;

                for (j = 0; j < i; ++j) {
                    png_free(png_ptr, cloned[j].data);
                }
                png_free(png_ptr, cloned);
                return 0;
            }
        } else {
            cloned[i].data = NULL;
            cloned[i].size = 0;
        }
    }

    info->unknown_chunks = cloned;
    return 1;
#else
    PNG_UNUSED(png_ptr)
    PNG_UNUSED(info)
    return 1;
#endif
}

static int
png_safe_snapshot_clone_splt(png_const_structrp png_ptr, png_info *info) {
#ifdef PNG_sPLT_SUPPORTED
    png_sPLT_tp cloned;
    int i;
    size_t bytes, entry_bytes;

    if (info->splt_palettes == NULL || info->splt_palettes_num <= 0) {
        info->splt_palettes = NULL;
        info->splt_palettes_num = 0;
        return 1;
    }

    if (!png_safe_snapshot_mul_size((size_t)info->splt_palettes_num, sizeof(*cloned), &bytes)) {
        return 0;
    }

    cloned = png_voidcast(png_sPLT_tp, png_safe_snapshot_alloc(png_ptr, bytes));
    if (cloned == NULL) {
        return 0;
    }

    for (i = 0; i < info->splt_palettes_num; ++i) {
        cloned[i] = info->splt_palettes[i];
        cloned[i].name = png_safe_snapshot_dup_string(png_ptr, info->splt_palettes[i].name);
        if (info->splt_palettes[i].name != NULL && cloned[i].name == NULL) {
            int j;

            for (j = 0; j < i; ++j) {
                png_free(png_ptr, cloned[j].name);
                png_free(png_ptr, cloned[j].entries);
            }
            png_free(png_ptr, cloned);
            return 0;
        }

        if (info->splt_palettes[i].entries != NULL && info->splt_palettes[i].nentries > 0) {
            if (!png_safe_snapshot_mul_size((size_t)info->splt_palettes[i].nentries,
                                            sizeof(png_sPLT_entry), &entry_bytes)) {
                png_free(png_ptr, cloned);
                return 0;
            }

            cloned[i].entries = png_voidcast(
                png_sPLT_entryp,
                png_safe_snapshot_dup_bytes(
                    png_ptr, info->splt_palettes[i].entries, entry_bytes));
            if (cloned[i].entries == NULL) {
                int j;

                png_free(png_ptr, cloned[i].name);
                for (j = 0; j < i; ++j) {
                    png_free(png_ptr, cloned[j].name);
                    png_free(png_ptr, cloned[j].entries);
                }
                png_free(png_ptr, cloned);
                return 0;
            }
        } else {
            cloned[i].entries = NULL;
            cloned[i].nentries = 0;
        }
    }

    info->splt_palettes = cloned;
    return 1;
#else
    PNG_UNUSED(png_ptr)
    PNG_UNUSED(info)
    return 1;
#endif
}

static int
png_safe_snapshot_clone_pcal(png_const_structrp png_ptr, png_info *info) {
#ifdef PNG_pCAL_SUPPORTED
    png_charpp params;
    int i;
    size_t bytes;

    {
        png_charp original_purpose = info->pcal_purpose;
        info->pcal_purpose = png_safe_snapshot_dup_string(png_ptr, original_purpose);
        if (original_purpose != NULL && info->pcal_purpose == NULL) {
            return 0;
        }
    }

    {
        png_charp original_units = info->pcal_units;
        info->pcal_units = png_safe_snapshot_dup_string(png_ptr, original_units);
        if (original_units != NULL && info->pcal_units == NULL) {
            png_free(png_ptr, info->pcal_purpose);
            info->pcal_purpose = NULL;
            return 0;
        }
    }

    if (info->pcal_params == NULL || info->pcal_nparams == 0) {
        info->pcal_params = NULL;
        return 1;
    }

    if (!png_safe_snapshot_mul_size((size_t)info->pcal_nparams, sizeof(*params), &bytes)) {
        return 0;
    }

    params = png_voidcast(png_charpp, png_safe_snapshot_alloc(png_ptr, bytes));
    if (params == NULL) {
        png_free(png_ptr, info->pcal_purpose);
        png_free(png_ptr, info->pcal_units);
        info->pcal_purpose = NULL;
        info->pcal_units = NULL;
        return 0;
    }

    for (i = 0; i < info->pcal_nparams; ++i) {
        params[i] = png_safe_snapshot_dup_string(png_ptr, info->pcal_params[i]);
        if (info->pcal_params[i] != NULL && params[i] == NULL) {
            int j;

            for (j = 0; j < i; ++j) {
                png_free(png_ptr, params[j]);
            }
            png_free(png_ptr, params);
            png_free(png_ptr, info->pcal_purpose);
            png_free(png_ptr, info->pcal_units);
            info->pcal_purpose = NULL;
            info->pcal_units = NULL;
            return 0;
        }
    }

    info->pcal_params = params;
    return 1;
#else
    PNG_UNUSED(png_ptr)
    PNG_UNUSED(info)
    return 1;
#endif
}

static int
png_safe_snapshot_clone_info(png_const_structrp png_ptr,
                             png_safe_parse_snapshot *snapshot,
                             png_const_inforp info_ptr) {
    snapshot->info = *info_ptr;

#ifdef PNG_INFO_IMAGE_SUPPORTED
    snapshot->info.row_pointers = NULL;
    snapshot->info.free_me &= ~PNG_FREE_ROWS;
#endif

    if ((snapshot->info.free_me & PNG_FREE_PLTE) != 0 && snapshot->info.palette != NULL) {
        size_t palette_bytes;

        if (!png_safe_snapshot_mul_size((size_t)snapshot->info.num_palette,
                                        sizeof(png_color), &palette_bytes)) {
            return 0;
        }

        snapshot->info.palette = png_voidcast(
            png_colorp,
            png_safe_snapshot_dup_bytes(png_ptr, info_ptr->palette, palette_bytes));
        if (snapshot->info.palette == NULL) {
            return 0;
        }
    }

    if ((snapshot->info.free_me & PNG_FREE_TRNS) != 0 && snapshot->info.trans_alpha != NULL) {
        snapshot->info.trans_alpha = png_voidcast(
            png_bytep,
            png_safe_snapshot_dup_bytes(
                png_ptr, info_ptr->trans_alpha, PNG_MAX_PALETTE_LENGTH));
        if (snapshot->info.trans_alpha == NULL) {
            return 0;
        }
    }

    if ((snapshot->info.free_me & PNG_FREE_HIST) != 0 && snapshot->info.hist != NULL) {
        size_t hist_bytes;

        if (!png_safe_snapshot_mul_size((size_t)snapshot->info.num_palette,
                                        sizeof(png_uint_16), &hist_bytes)) {
            return 0;
        }

        snapshot->info.hist = png_voidcast(
            png_uint_16p,
            png_safe_snapshot_dup_bytes(png_ptr, info_ptr->hist, hist_bytes));
        if (snapshot->info.hist == NULL) {
            return 0;
        }
    }

    if ((snapshot->info.free_me & PNG_FREE_ICCP) != 0) {
        snapshot->info.iccp_name =
            png_safe_snapshot_dup_string(png_ptr, info_ptr->iccp_name);
        if (info_ptr->iccp_name != NULL && snapshot->info.iccp_name == NULL) {
            return 0;
        }

        snapshot->info.iccp_profile = png_voidcast(
            png_bytep,
            png_safe_snapshot_dup_bytes(
                png_ptr, info_ptr->iccp_profile, info_ptr->iccp_proflen));
        if (info_ptr->iccp_profile != NULL && snapshot->info.iccp_profile == NULL) {
            return 0;
        }
    }

    if ((snapshot->info.free_me & PNG_FREE_TEXT) != 0 &&
        !png_safe_snapshot_clone_text(png_ptr, &snapshot->info)) {
        return 0;
    }

    if ((snapshot->info.free_me & PNG_FREE_SCAL) != 0) {
        snapshot->info.scal_s_width =
            png_safe_snapshot_dup_string(png_ptr, info_ptr->scal_s_width);
        if (info_ptr->scal_s_width != NULL && snapshot->info.scal_s_width == NULL) {
            return 0;
        }

        snapshot->info.scal_s_height =
            png_safe_snapshot_dup_string(png_ptr, info_ptr->scal_s_height);
        if (info_ptr->scal_s_height != NULL && snapshot->info.scal_s_height == NULL) {
            return 0;
        }
    }

    if ((snapshot->info.free_me & PNG_FREE_PCAL) != 0 &&
        !png_safe_snapshot_clone_pcal(png_ptr, &snapshot->info)) {
        return 0;
    }

    if ((snapshot->info.free_me & PNG_FREE_UNKN) != 0 &&
        !png_safe_snapshot_clone_unknowns(png_ptr, &snapshot->info)) {
        return 0;
    }

    if ((snapshot->info.free_me & PNG_FREE_SPLT) != 0 &&
        !png_safe_snapshot_clone_splt(png_ptr, &snapshot->info)) {
        return 0;
    }

    if ((snapshot->info.free_me & PNG_FREE_EXIF) != 0) {
        snapshot->info.exif = png_voidcast(
            png_bytep,
            png_safe_snapshot_dup_bytes(png_ptr, info_ptr->exif, info_ptr->num_exif));
        if (info_ptr->exif != NULL && snapshot->info.exif == NULL) {
            return 0;
        }
#ifdef PNG_READ_eXIf_SUPPORTED
        snapshot->info.eXIf_buf = png_voidcast(
            png_bytep,
            png_safe_snapshot_dup_bytes(png_ptr, info_ptr->eXIf_buf, info_ptr->num_exif));
        if (info_ptr->eXIf_buf != NULL && snapshot->info.eXIf_buf == NULL) {
            return 0;
        }
#endif
    }

    return 1;
}

static void
png_safe_snapshot_release_info(png_safe_parse_snapshot *snapshot) {
    if (snapshot == NULL || !snapshot->has_info) {
        return;
    }

    png_free_data(snapshot->alloc_png_ptr, &snapshot->info, PNG_FREE_ALL, -1);
    memset(&snapshot->info, 0, sizeof(snapshot->info));
    snapshot->has_info = 0;
}

void *png_safe_parse_snapshot_capture(png_const_structrp png_ptr,
                                      png_const_inforp info_ptr) {
    png_safe_parse_snapshot *snapshot =
        (png_safe_parse_snapshot *)calloc(1, sizeof(*snapshot));

    if (snapshot == NULL) {
        return NULL;
    }

    snapshot->alloc_png_ptr = png_ptr;

    if (info_ptr != NULL) {
        snapshot->has_info = 1;
        if (!png_safe_snapshot_clone_info(png_ptr, snapshot, info_ptr)) {
            png_safe_snapshot_release_info(snapshot);
            free(snapshot);
            return NULL;
        }
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

    if (info_ptr != NULL && snapshot->has_info) {
        png_free_data(png_ptr, info_ptr, PNG_FREE_ALL, -1);
        *info_ptr = snapshot->info;
        {
            png_safe_parse_snapshot *snapshot_mut =
                (png_safe_parse_snapshot *)snapshot_ptr;
            memset(&snapshot_mut->info, 0, sizeof(snapshot_mut->info));
            snapshot_mut->has_info = 0;
        }
    }
}

void png_safe_parse_snapshot_free(void *snapshot_ptr) {
    png_safe_parse_snapshot *snapshot =
        (png_safe_parse_snapshot *)snapshot_ptr;

    if (snapshot == NULL) {
        return;
    }

    png_safe_snapshot_release_info(snapshot);
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
    out->zowner = png_ptr->zowner;
    out->crc = png_ptr->crc;
#ifdef PNG_IO_STATE_SUPPORTED
    out->io_state = png_ptr->io_state;
#else
    out->io_state = 0;
#endif
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
    png_ptr->zowner = in->zowner;
    png_ptr->crc = in->crc;
#ifdef PNG_IO_STATE_SUPPORTED
    png_ptr->io_state = in->io_state;
#endif
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

void png_safe_sync_png_info_aliases(png_structrp png_ptr, png_const_inforp info_ptr) {
    if (png_ptr == NULL || info_ptr == NULL) {
        return;
    }

    png_ptr->palette = info_ptr->palette;
    png_ptr->num_palette = info_ptr->num_palette;
    png_ptr->trans_alpha = info_ptr->trans_alpha;
    png_ptr->num_trans = info_ptr->num_trans;
}
