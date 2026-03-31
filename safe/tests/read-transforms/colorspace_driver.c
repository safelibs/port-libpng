#include <assert.h>
#include <setjmp.h>
#include <stdio.h>
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

static void run_fixed_roundtrip(void) {
    test_ctx ctx;
    memset(&ctx, 0, sizeof ctx);

    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, &ctx, error_cb, warning_cb);
    assert(png_ptr != NULL);
    png_infop info_ptr = png_create_info_struct(png_ptr);
    assert(info_ptr != NULL);

    if (setjmp(ctx.env) != 0) {
        assert(!"cHRM fixed roundtrip unexpectedly failed");
    }

    png_set_cHRM_XYZ_fixed(
        png_ptr, info_ptr,
        41240, 21260, 19305,
        35760, 71520, 11920,
        1805, 7220, 95050);

    png_fixed_point red_X = 0;
    png_fixed_point red_Y = 0;
    png_fixed_point red_Z = 0;
    png_fixed_point green_X = 0;
    png_fixed_point green_Y = 0;
    png_fixed_point green_Z = 0;
    png_fixed_point blue_X = 0;
    png_fixed_point blue_Y = 0;
    png_fixed_point blue_Z = 0;

    assert(png_get_cHRM_XYZ_fixed(
               png_ptr, info_ptr, &red_X, &red_Y, &red_Z, &green_X, &green_Y,
               &green_Z, &blue_X, &blue_Y, &blue_Z) != 0);
    assert(red_Y > 0);
    assert(green_Y > red_Y);
    assert(blue_Z > blue_Y);

    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
}

static void run_float_roundtrip(void) {
    test_ctx ctx;
    memset(&ctx, 0, sizeof ctx);

    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, &ctx, error_cb, warning_cb);
    assert(png_ptr != NULL);
    png_infop info_ptr = png_create_info_struct(png_ptr);
    assert(info_ptr != NULL);

    if (setjmp(ctx.env) != 0) {
        assert(!"cHRM float roundtrip unexpectedly failed");
    }

    png_set_cHRM_XYZ(
        png_ptr, info_ptr,
        0.4124, 0.2126, 0.1930,
        0.3576, 0.7152, 0.1192,
        0.0181, 0.0722, 0.9505);

    double red_X = 0.0;
    double red_Y = 0.0;
    double red_Z = 0.0;
    double green_X = 0.0;
    double green_Y = 0.0;
    double green_Z = 0.0;
    double blue_X = 0.0;
    double blue_Y = 0.0;
    double blue_Z = 0.0;

    assert(png_get_cHRM_XYZ(
               png_ptr, info_ptr, &red_X, &red_Y, &red_Z, &green_X, &green_Y,
               &green_Z, &blue_X, &blue_Y, &blue_Z) != 0);
    assert(red_X > 0.4 && red_X < 0.5);
    assert(green_Y > 0.7 && green_Y < 0.8);
    assert(blue_Z > 0.9 && blue_Z < 1.0);

    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
}

static void run_invalid_endpoints_case(void) {
    test_ctx ctx;
    memset(&ctx, 0, sizeof ctx);

    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, &ctx, error_cb, warning_cb);
    assert(png_ptr != NULL);
    png_infop info_ptr = png_create_info_struct(png_ptr);
    assert(info_ptr != NULL);

    if (setjmp(ctx.env) != 0) {
        assert(!"invalid cHRM endpoint case unexpectedly failed");
    }

    png_set_benign_errors(png_ptr, 1);
    png_set_cHRM_XYZ_fixed(png_ptr, info_ptr, 0, 0, 0, 0, 0, 0, 0, 0, 0);

    assert(ctx.warnings > 0);
    assert(png_get_cHRM_XYZ_fixed(
               png_ptr, info_ptr, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
               NULL) == 0);

    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
}

static void run_overflow_endpoints_case(void) {
    test_ctx ctx;
    memset(&ctx, 0, sizeof ctx);

    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, &ctx, error_cb, warning_cb);
    assert(png_ptr != NULL);
    png_infop info_ptr = png_create_info_struct(png_ptr);
    assert(info_ptr != NULL);

    if (setjmp(ctx.env) != 0) {
        assert(!"overflow cHRM endpoint case unexpectedly failed");
    }

    png_set_benign_errors(png_ptr, 1);
    png_set_cHRM_XYZ_fixed(
        png_ptr, info_ptr,
        2000000000, 1, 1,
        2000000000, 1, 1,
        2000000000, 1, 1);

    assert(ctx.warnings > 0);
    assert(png_get_cHRM_XYZ_fixed(
               png_ptr, info_ptr, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
               NULL) == 0);

    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
}

int main(void) {
    run_fixed_roundtrip();
    run_float_roundtrip();
    run_invalid_endpoints_case();
    run_overflow_endpoints_case();
    return 0;
}
