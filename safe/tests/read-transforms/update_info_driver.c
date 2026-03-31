#include <assert.h>
#include <setjmp.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <png.h>

typedef struct {
    jmp_buf env;
    int warnings;
} test_ctx;

static const unsigned pass_start_col[7] = {0, 4, 0, 2, 0, 1, 0};
static const unsigned pass_col_offset[7] = {8, 8, 4, 4, 2, 2, 1};
static const unsigned pass_start_row[7] = {0, 0, 4, 0, 2, 0, 1};
static const unsigned pass_row_offset[7] = {8, 8, 8, 4, 4, 2, 2};

static void error_cb(png_structp png_ptr, png_const_charp message) {
    (void)message;
    test_ctx *ctx = (test_ctx *)png_get_error_ptr(png_ptr);
    assert(ctx != NULL);
    longjmp(ctx->env, 1);
}

static void warning_cb(png_structp png_ptr, png_const_charp message) {
    (void)message;
    test_ctx *ctx = (test_ctx *)png_get_error_ptr(png_ptr);
    assert(ctx != NULL);
    ++ctx->warnings;
}

static png_uint_32 pass_cols(png_uint_32 width, int pass) {
    if (width <= pass_start_col[pass]) {
        return 0;
    }

    return (width - pass_start_col[pass] + pass_col_offset[pass] - 1U) /
           pass_col_offset[pass];
}

static int row_in_pass(png_uint_32 row, int pass) {
    return row >= pass_start_row[pass] &&
           ((row - pass_start_row[pass]) % pass_row_offset[pass]) == 0;
}

static void deinterlace_pass_row(png_bytep dst, png_const_bytep src, png_byte pixel_depth,
                                 png_uint_32 width, int pass) {
    png_uint_32 src_x = 0;
    png_uint_32 out_x = pass_start_col[pass];
    png_uint_32 step = pass_col_offset[pass];

    if (pixel_depth >= 8) {
        size_t pixel_bytes = (size_t)pixel_depth / 8U;
        while (out_x < width) {
            memcpy(dst + (size_t)out_x * pixel_bytes, src + (size_t)src_x * pixel_bytes,
                   pixel_bytes);
            ++src_x;
            out_x += step;
        }
    } else {
        while (out_x < width) {
            size_t src_bit = (size_t)src_x * pixel_depth;
            size_t dst_bit = (size_t)out_x * pixel_depth;
            size_t src_byte = src_bit / 8U;
            size_t dst_byte = dst_bit / 8U;
            unsigned src_shift = 8U - pixel_depth - (unsigned)(src_bit % 8U);
            unsigned dst_shift = 8U - pixel_depth - (unsigned)(dst_bit % 8U);
            png_byte value = (png_byte)((src[src_byte] >> src_shift) & ((1U << pixel_depth) - 1U));
            png_byte mask = (png_byte)(((1U << pixel_depth) - 1U) << dst_shift);
            dst[dst_byte] = (png_byte)((dst[dst_byte] & ~mask) | (value << dst_shift));
            ++src_x;
            out_x += step;
        }
    }
}

static void read_all_rows(png_structp png_ptr, png_infop info_ptr) {
    png_uint_32 height = png_get_image_height(png_ptr, info_ptr);
    size_t rowbytes = png_get_rowbytes(png_ptr, info_ptr);
    png_bytep row = (png_bytep)malloc(rowbytes);
    assert(row != NULL);

    for (png_uint_32 y = 0; y < height; ++y) {
        memset(row, 0, rowbytes);
        png_read_row(png_ptr, row, NULL);
    }

    free(row);
}

static void run_palette_rgba_case(const char *path) {
    test_ctx ctx;
    memset(&ctx, 0, sizeof ctx);

    FILE *fp = fopen(path, "rb");
    assert(fp != NULL);

    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, &ctx, error_cb, warning_cb);
    assert(png_ptr != NULL);
    png_infop info_ptr = png_create_info_struct(png_ptr);
    assert(info_ptr != NULL);

    if (setjmp(ctx.env) != 0) {
        assert(!"palette RGBA transform case unexpectedly failed");
    }

    png_init_io(png_ptr, fp);
    png_read_info(png_ptr, info_ptr);
    png_set_palette_to_rgb(png_ptr);
    png_set_tRNS_to_alpha(png_ptr);
    png_set_bgr(png_ptr);
    png_set_swap_alpha(png_ptr);
    png_set_invert_alpha(png_ptr);
    png_read_update_info(png_ptr, info_ptr);

    assert(png_get_channels(png_ptr, info_ptr) == 4);
    assert(png_get_bit_depth(png_ptr, info_ptr) == 8);
    assert(png_get_rowbytes(png_ptr, info_ptr) == png_get_image_width(png_ptr, info_ptr) * 4U);

    read_all_rows(png_ptr, info_ptr);
    png_read_end(png_ptr, NULL);
    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
    fclose(fp);
}

static void run_quantize_case(const char *path) {
    test_ctx ctx;
    memset(&ctx, 0, sizeof ctx);

    FILE *fp = fopen(path, "rb");
    assert(fp != NULL);

    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, &ctx, error_cb, warning_cb);
    assert(png_ptr != NULL);
    png_infop info_ptr = png_create_info_struct(png_ptr);
    assert(info_ptr != NULL);

    if (setjmp(ctx.env) != 0) {
        assert(!"quantize case unexpectedly failed");
    }

    png_init_io(png_ptr, fp);
    png_read_info(png_ptr, info_ptr);

    png_colorp palette = NULL;
    int num_palette = 0;
    assert((png_get_PLTE(png_ptr, info_ptr, &palette, &num_palette) & PNG_INFO_PLTE) != 0);
    assert(num_palette > 4);

    png_set_quantize(png_ptr, palette, num_palette, 4, NULL, 0);
    png_read_update_info(png_ptr, info_ptr);

    assert(png_get_rowbytes(png_ptr, info_ptr) > 0);
    read_all_rows(png_ptr, info_ptr);
    png_read_end(png_ptr, NULL);
    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
    fclose(fp);
}

static void run_expand16_case(const char *path) {
    test_ctx ctx;
    memset(&ctx, 0, sizeof ctx);

    FILE *fp = fopen(path, "rb");
    assert(fp != NULL);

    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, &ctx, error_cb, warning_cb);
    assert(png_ptr != NULL);
    png_infop info_ptr = png_create_info_struct(png_ptr);
    assert(info_ptr != NULL);

    if (setjmp(ctx.env) != 0) {
        assert(!"expand_16 case unexpectedly failed");
    }

    png_init_io(png_ptr, fp);
    png_read_info(png_ptr, info_ptr);
    png_set_expand_16(png_ptr);
    png_read_update_info(png_ptr, info_ptr);

    png_byte channels = png_get_channels(png_ptr, info_ptr);
    assert(channels == 1 || channels == 2);
    assert(png_get_bit_depth(png_ptr, info_ptr) == 16);
    assert(
        png_get_rowbytes(png_ptr, info_ptr) ==
        png_get_image_width(png_ptr, info_ptr) * (size_t)channels * 2U);

    read_all_rows(png_ptr, info_ptr);
    png_read_end(png_ptr, NULL);
    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
    fclose(fp);
}

static void run_gray_background_case(const char *path) {
    test_ctx ctx;
    memset(&ctx, 0, sizeof ctx);

    FILE *fp = fopen(path, "rb");
    assert(fp != NULL);

    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, &ctx, error_cb, warning_cb);
    assert(png_ptr != NULL);
    png_infop info_ptr = png_create_info_struct(png_ptr);
    assert(info_ptr != NULL);

    if (setjmp(ctx.env) != 0) {
        assert(!"gray background case unexpectedly failed");
    }

    png_init_io(png_ptr, fp);
    png_read_info(png_ptr, info_ptr);
    png_set_expand(png_ptr);
    png_set_gray_to_rgb(png_ptr);

    png_color_16 background;
    memset(&background, 0, sizeof background);
    background.red = 0x20;
    background.green = 0x60;
    background.blue = 0xa0;
    background.gray = 0x60;

    png_set_background_fixed(
        png_ptr, &background, PNG_BACKGROUND_GAMMA_SCREEN, 1, PNG_GAMMA_sRGB);
    png_read_update_info(png_ptr, info_ptr);

    assert(png_get_channels(png_ptr, info_ptr) == 3);
    assert(png_get_bit_depth(png_ptr, info_ptr) == 8);
    assert(png_get_rowbytes(png_ptr, info_ptr) == png_get_image_width(png_ptr, info_ptr) * 3U);

    read_all_rows(png_ptr, info_ptr);
    png_read_end(png_ptr, NULL);
    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
    fclose(fp);
}

static void run_rgb_to_gray_scale_case(const char *path) {
    test_ctx ctx;
    memset(&ctx, 0, sizeof ctx);

    FILE *fp = fopen(path, "rb");
    assert(fp != NULL);

    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, &ctx, error_cb, warning_cb);
    assert(png_ptr != NULL);
    png_infop info_ptr = png_create_info_struct(png_ptr);
    assert(info_ptr != NULL);

    if (setjmp(ctx.env) != 0) {
        assert(!"rgb_to_gray + scale_16 case unexpectedly failed");
    }

    png_init_io(png_ptr, fp);
    png_read_info(png_ptr, info_ptr);
    png_set_rgb_to_gray_fixed(png_ptr, PNG_ERROR_ACTION_NONE, -1, -1);
    png_set_scale_16(png_ptr);
    png_read_update_info(png_ptr, info_ptr);

    assert(png_get_channels(png_ptr, info_ptr) == 1);
    assert(png_get_bit_depth(png_ptr, info_ptr) == 8);
    assert(png_get_rowbytes(png_ptr, info_ptr) == png_get_image_width(png_ptr, info_ptr));

    read_all_rows(png_ptr, info_ptr);
    png_read_end(png_ptr, NULL);
    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
    fclose(fp);
}

static void run_gray16_shift_strip_case(const char *path) {
    test_ctx ctx;
    memset(&ctx, 0, sizeof ctx);

    FILE *fp = fopen(path, "rb");
    assert(fp != NULL);

    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, &ctx, error_cb, warning_cb);
    assert(png_ptr != NULL);
    png_infop info_ptr = png_create_info_struct(png_ptr);
    assert(info_ptr != NULL);

    if (setjmp(ctx.env) != 0) {
        assert(!"shift + strip_16 + invert_mono case unexpectedly failed");
    }

    png_init_io(png_ptr, fp);
    png_read_info(png_ptr, info_ptr);

    png_color_8 shift;
    memset(&shift, 0, sizeof shift);
    shift.gray = 12;
    png_set_shift(png_ptr, &shift);
    png_set_strip_16(png_ptr);
    png_set_invert_mono(png_ptr);
    png_read_update_info(png_ptr, info_ptr);

    assert(png_get_channels(png_ptr, info_ptr) == 1);
    assert(png_get_bit_depth(png_ptr, info_ptr) == 8);
    assert(png_get_rowbytes(png_ptr, info_ptr) == png_get_image_width(png_ptr, info_ptr));

    read_all_rows(png_ptr, info_ptr);
    png_read_end(png_ptr, NULL);
    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
    fclose(fp);
}

static void run_alpha_mode_case(const char *path) {
    test_ctx ctx;
    memset(&ctx, 0, sizeof ctx);

    FILE *fp = fopen(path, "rb");
    assert(fp != NULL);

    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, &ctx, error_cb, warning_cb);
    assert(png_ptr != NULL);
    png_infop info_ptr = png_create_info_struct(png_ptr);
    assert(info_ptr != NULL);

    if (setjmp(ctx.env) != 0) {
        assert(!"alpha mode case unexpectedly failed");
    }

    png_init_io(png_ptr, fp);
    png_read_info(png_ptr, info_ptr);
    png_set_scale_16(png_ptr);
    png_set_alpha_mode_fixed(png_ptr, PNG_ALPHA_STANDARD, PNG_GAMMA_sRGB);
    png_read_update_info(png_ptr, info_ptr);

    assert(png_get_channels(png_ptr, info_ptr) == 4);
    assert(png_get_bit_depth(png_ptr, info_ptr) == 8);
    assert(png_get_rowbytes(png_ptr, info_ptr) == png_get_image_width(png_ptr, info_ptr) * 4U);

    read_all_rows(png_ptr, info_ptr);
    png_read_end(png_ptr, NULL);
    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
    fclose(fp);
}

static void run_interlace_smoke_case(const char *path) {
    test_ctx ctx;
    memset(&ctx, 0, sizeof ctx);

    FILE *fp = fopen(path, "rb");
    assert(fp != NULL);

    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, &ctx, error_cb, warning_cb);
    assert(png_ptr != NULL);
    png_infop info_ptr = png_create_info_struct(png_ptr);
    assert(info_ptr != NULL);

    if (setjmp(ctx.env) != 0) {
        assert(!"interlace smoke case unexpectedly failed");
    }

    png_init_io(png_ptr, fp);
    png_read_info(png_ptr, info_ptr);
    assert(png_get_interlace_type(png_ptr, info_ptr) == PNG_INTERLACE_ADAM7);
    assert(png_set_interlace_handling(png_ptr) == PNG_INTERLACE_ADAM7_PASSES);
    png_read_update_info(png_ptr, info_ptr);
    read_all_rows(png_ptr, info_ptr);
    png_read_end(png_ptr, NULL);
    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
    fclose(fp);
}

static void run_interlace_raw_pass_case(const char *path) {
    test_ctx ref_ctx;
    memset(&ref_ctx, 0, sizeof ref_ctx);

    FILE *ref_fp = fopen(path, "rb");
    assert(ref_fp != NULL);

    png_structp ref_png =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, &ref_ctx, error_cb, warning_cb);
    assert(ref_png != NULL);
    png_infop ref_info = png_create_info_struct(ref_png);
    assert(ref_info != NULL);

    if (setjmp(ref_ctx.env) != 0) {
        assert(!"interlace reference case unexpectedly failed");
    }

    png_init_io(ref_png, ref_fp);
    png_read_info(ref_png, ref_info);
    assert(png_get_interlace_type(ref_png, ref_info) == PNG_INTERLACE_ADAM7);
    assert(png_set_interlace_handling(ref_png) == PNG_INTERLACE_ADAM7_PASSES);
    png_read_update_info(ref_png, ref_info);

    png_uint_32 width = png_get_image_width(ref_png, ref_info);
    png_uint_32 height = png_get_image_height(ref_png, ref_info);
    png_byte pixel_depth =
        (png_byte)(png_get_channels(ref_png, ref_info) * png_get_bit_depth(ref_png, ref_info));
    size_t rowbytes = png_get_rowbytes(ref_png, ref_info);
    png_bytep reference = (png_bytep)calloc(height, rowbytes);
    png_bytepp ref_rows = (png_bytepp)malloc(height * sizeof(png_bytep));
    assert(reference != NULL);
    assert(ref_rows != NULL);
    for (png_uint_32 y = 0; y < height; ++y) {
        ref_rows[y] = reference + (size_t)y * rowbytes;
    }
    png_read_image(ref_png, ref_rows);
    png_read_end(ref_png, NULL);
    png_destroy_read_struct(&ref_png, &ref_info, NULL);
    fclose(ref_fp);
    free(ref_rows);

    test_ctx ctx;
    memset(&ctx, 0, sizeof ctx);

    FILE *fp = fopen(path, "rb");
    assert(fp != NULL);

    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, &ctx, error_cb, warning_cb);
    assert(png_ptr != NULL);
    png_infop info_ptr = png_create_info_struct(png_ptr);
    assert(info_ptr != NULL);

    if (setjmp(ctx.env) != 0) {
        assert(!"raw-pass interlace case unexpectedly failed");
    }

    png_init_io(png_ptr, fp);
    png_read_info(png_ptr, info_ptr);
    assert(png_get_interlace_type(png_ptr, info_ptr) == PNG_INTERLACE_ADAM7);
    png_read_update_info(png_ptr, info_ptr);

    assert(png_get_rowbytes(png_ptr, info_ptr) == rowbytes);
    assert((png_byte)(png_get_channels(png_ptr, info_ptr) * png_get_bit_depth(png_ptr, info_ptr)) ==
           pixel_depth);

    png_bytep reconstructed = (png_bytep)calloc(height, rowbytes);
    png_bytep row = (png_bytep)malloc(rowbytes);
    png_bytep display = (png_bytep)malloc(rowbytes);
    assert(reconstructed != NULL);
    assert(row != NULL);
    assert(display != NULL);

    for (int pass = 0; pass < PNG_INTERLACE_ADAM7_PASSES; ++pass) {
        png_uint_32 pass_width = pass_cols(width, pass);
        size_t pass_rowbytes = (size_t)((pass_width * pixel_depth + 7U) / 8U);
        for (png_uint_32 y = 0; y < height; ++y) {
            if (pass_width == 0 || !row_in_pass(y, pass)) {
                continue;
            }

            memset(row, 0, rowbytes);
            memset(display, 0, rowbytes);
            png_read_row(png_ptr, row, display);
            assert(memcmp(row, display, pass_rowbytes) == 0);
            deinterlace_pass_row(reconstructed + (size_t)y * rowbytes, row, pixel_depth, width,
                                 pass);
        }
    }

    png_read_end(png_ptr, NULL);
    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
    fclose(fp);

    assert(memcmp(reference, reconstructed, height * rowbytes) == 0);

    free(reference);
    free(reconstructed);
    free(row);
    free(display);
}

int main(int argc, char **argv) {
    assert(argc == 7);

    run_palette_rgba_case(argv[1]);
    run_quantize_case(argv[1]);
    run_expand16_case(argv[2]);
    run_gray_background_case(argv[2]);
    run_rgb_to_gray_scale_case(argv[3]);
    run_gray16_shift_strip_case(argv[4]);
    run_alpha_mode_case(argv[5]);
    run_interlace_smoke_case(argv[6]);
    run_interlace_raw_pass_case(argv[6]);
    return 0;
}
