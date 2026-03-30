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
    return 0;
}
