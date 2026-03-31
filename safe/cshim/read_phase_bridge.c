#include "pngpriv.h"

/*
 * Checked-in C surface: only private-layout mirror types and direct field-copy
 * accessors remain visible here.
 */

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

void png_safe_read_core_get(png_const_structrp png_ptr, png_safe_read_core *out);
void png_safe_read_core_set(png_structrp png_ptr, const png_safe_read_core *input);
void png_safe_info_core_get(png_const_inforp info_ptr, png_safe_info_core *out);
void png_safe_info_core_set(png_inforp info_ptr, const png_safe_info_core *input);
void png_safe_sync_png_info_aliases(png_structrp png_ptr, png_const_inforp info_ptr);
