#include <assert.h>
#include <setjmp.h>
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

static void run_read_png_transform_case(const char *path) {
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
        assert(!"png_read_png transform case unexpectedly failed");
    }

    png_init_io(png_ptr, fp);
    png_read_png(png_ptr, info_ptr,
        PNG_TRANSFORM_EXPAND | PNG_TRANSFORM_BGR |
            PNG_TRANSFORM_SWAP_ALPHA | PNG_TRANSFORM_INVERT_ALPHA,
        NULL);

    assert((png_get_valid(png_ptr, info_ptr, PNG_INFO_IDAT) & PNG_INFO_IDAT) != 0);
    assert(png_get_rows(png_ptr, info_ptr) != NULL);
    assert(png_get_rowbytes(png_ptr, info_ptr) == png_get_image_width(png_ptr, info_ptr) * 4U);
    assert(png_get_channels(png_ptr, info_ptr) == 4);

    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
    fclose(fp);
}

static void run_invalid_index_case(const char *path, int allowed, int expect_enabled) {
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
        assert(!"invalid-index read case unexpectedly failed");
    }

    png_init_io(png_ptr, fp);
    png_set_check_for_invalid_index(png_ptr, allowed);
    png_read_png(png_ptr, info_ptr, 0, NULL);

    if (expect_enabled) {
        png_colorp palette = NULL;
        int num_palette = 0;
        int palette_max = png_get_palette_max(png_ptr, info_ptr);
        assert((png_get_PLTE(png_ptr, info_ptr, &palette, &num_palette) & PNG_INFO_PLTE) != 0);
        assert(num_palette > 0);
        assert(palette_max >= num_palette);
    } else {
        assert(png_get_palette_max(png_ptr, info_ptr) == -1);
    }

    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
    fclose(fp);
}

static void run_bad_iccp_case(const char *path) {
    test_ctx ctx;
    memset(&ctx, 0, sizeof ctx);

    FILE *fp = fopen(path, "rb");
    assert(fp != NULL);

    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, &ctx, error_cb, warning_cb);
    assert(png_ptr != NULL);
    png_infop info_ptr = png_create_info_struct(png_ptr);
    assert(info_ptr != NULL);

    if (setjmp(ctx.env) == 0) {
        png_set_benign_errors(png_ptr, 0);
        png_init_io(png_ptr, fp);
        png_read_png(png_ptr, info_ptr, 0, NULL);
        assert(!"bad iCCP fixture unexpectedly decoded");
    }

    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
    fclose(fp);
}

int main(int argc, char **argv) {
    assert(argc == 4);

    run_read_png_transform_case(argv[1]);
    run_invalid_index_case(argv[2], 1, 1);
    run_invalid_index_case(argv[2], -1, 0);
    run_bad_iccp_case(argv[3]);
    return 0;
}
