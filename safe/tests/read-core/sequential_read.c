#include <assert.h>
#include <setjmp.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <png.h>

#ifndef PNG_IGNORE_ADLER32
#define PNG_IGNORE_ADLER32 8
#endif

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

static png_structp open_reader(
    const char *path,
    test_ctx *ctx,
    png_infop *info_out,
    png_infop *end_out,
    FILE **fp_out)
{
    png_structp png_ptr;

    *fp_out = fopen(path, "rb");
    assert(*fp_out != NULL);

    png_ptr = png_create_read_struct(PNG_LIBPNG_VER_STRING, ctx, error_cb, warning_cb);
    assert(png_ptr != NULL);

    *info_out = png_create_info_struct(png_ptr);
    *end_out = png_create_info_struct(png_ptr);
    assert(*info_out != NULL);
    assert(*end_out != NULL);

    png_init_io(png_ptr, *fp_out);
    return png_ptr;
}

static void close_reader(
    png_structp *png_ptr,
    png_infop *info_ptr,
    png_infop *end_ptr,
    FILE **fp)
{
    png_destroy_read_struct(png_ptr, info_ptr, end_ptr);
    fclose(*fp);
}

static int read_with_rows_api(const char *path) {
    test_ctx ctx;
    memset(&ctx, 0, sizeof ctx);

    png_infop info_ptr = NULL;
    png_infop end_ptr = NULL;
    FILE *fp = NULL;
    png_structp png_ptr = open_reader(path, &ctx, &info_ptr, &end_ptr, &fp);

    if (setjmp(ctx.env) != 0) {
        close_reader(&png_ptr, &info_ptr, &end_ptr, &fp);
        return 0;
    }

    png_read_info(png_ptr, info_ptr);
    png_read_update_info(png_ptr, info_ptr);

    png_uint_32 height = png_get_image_height(png_ptr, info_ptr);
    size_t rowbytes = png_get_rowbytes(png_ptr, info_ptr);
    assert(height > 0);
    assert(rowbytes > 0);

    png_bytep storage = (png_bytep)calloc(height, rowbytes);
    png_bytep *rows = (png_bytep *)malloc(sizeof(*rows) * height);
    assert(storage != NULL);
    assert(rows != NULL);

    for (png_uint_32 y = 0; y < height; ++y) {
        rows[y] = storage + y * rowbytes;
    }

    png_read_rows(png_ptr, rows, NULL, height);
    png_read_end(png_ptr, end_ptr);

    free(rows);
    free(storage);
    close_reader(&png_ptr, &info_ptr, &end_ptr, &fp);
    return 1;
}

static int read_with_start_read_image(const char *path) {
    test_ctx ctx;
    memset(&ctx, 0, sizeof ctx);

    png_infop info_ptr = NULL;
    png_infop end_ptr = NULL;
    FILE *fp = NULL;
    png_structp png_ptr = open_reader(path, &ctx, &info_ptr, &end_ptr, &fp);

    if (setjmp(ctx.env) != 0) {
        close_reader(&png_ptr, &info_ptr, &end_ptr, &fp);
        return 0;
    }

    png_read_info(png_ptr, info_ptr);
    png_start_read_image(png_ptr);

    png_uint_32 height = png_get_image_height(png_ptr, info_ptr);
    size_t rowbytes = png_get_rowbytes(png_ptr, info_ptr);
    png_bytep row = (png_bytep)malloc(rowbytes);
    assert(height > 0);
    assert(rowbytes > 0);
    assert(row != NULL);

    for (png_uint_32 y = 0; y < height; ++y) {
        png_read_row(png_ptr, row, NULL);
    }

    png_read_end(png_ptr, end_ptr);
    free(row);
    close_reader(&png_ptr, &info_ptr, &end_ptr, &fp);
    return 1;
}

static int read_with_image_api(const char *path, int expect_interlaced_warning) {
    test_ctx ctx;
    memset(&ctx, 0, sizeof ctx);

    png_infop info_ptr = NULL;
    png_infop end_ptr = NULL;
    FILE *fp = NULL;
    png_structp png_ptr = open_reader(path, &ctx, &info_ptr, &end_ptr, &fp);

    if (setjmp(ctx.env) != 0) {
        close_reader(&png_ptr, &info_ptr, &end_ptr, &fp);
        return 0;
    }

    png_read_info(png_ptr, info_ptr);
    png_read_update_info(png_ptr, info_ptr);

    png_uint_32 height = png_get_image_height(png_ptr, info_ptr);
    size_t rowbytes = png_get_rowbytes(png_ptr, info_ptr);
    png_bytep storage = (png_bytep)calloc(height, rowbytes);
    png_bytep *rows = (png_bytep *)malloc(sizeof(*rows) * height);
    assert(height > 0);
    assert(rowbytes > 0);
    assert(storage != NULL);
    assert(rows != NULL);

    for (png_uint_32 y = 0; y < height; ++y) {
        rows[y] = storage + y * rowbytes;
    }

    png_read_image(png_ptr, rows);
    png_read_end(png_ptr, end_ptr);

    if (expect_interlaced_warning) {
        assert(ctx.warnings >= 1);
    }

    free(rows);
    free(storage);
    close_reader(&png_ptr, &info_ptr, &end_ptr, &fp);
    return 1;
}

static int read_with_row_loop(const char *path, int ignore_adler32) {
    test_ctx ctx;
    memset(&ctx, 0, sizeof ctx);

    png_infop info_ptr = NULL;
    png_infop end_ptr = NULL;
    FILE *fp = NULL;
    png_structp png_ptr = open_reader(path, &ctx, &info_ptr, &end_ptr, &fp);

    if (setjmp(ctx.env) != 0) {
        close_reader(&png_ptr, &info_ptr, &end_ptr, &fp);
        return 0;
    }

    png_set_benign_errors(png_ptr, 0);
    if (ignore_adler32) {
        (void)png_set_option(png_ptr, PNG_IGNORE_ADLER32, 1);
    }

    png_read_info(png_ptr, info_ptr);
    png_read_update_info(png_ptr, info_ptr);

    png_uint_32 height = png_get_image_height(png_ptr, info_ptr);
    size_t rowbytes = png_get_rowbytes(png_ptr, info_ptr);
    png_bytep row = (png_bytep)malloc(rowbytes);
    assert(height > 0);
    assert(rowbytes > 0);
    assert(row != NULL);

    for (png_uint_32 y = 0; y < height; ++y) {
        png_read_row(png_ptr, row, NULL);
    }

    png_read_end(png_ptr, end_ptr);
    free(row);
    close_reader(&png_ptr, &info_ptr, &end_ptr, &fp);
    return 1;
}

int main(int argc, char **argv) {
    assert(argc == 4);

    assert(read_with_rows_api(argv[1]) == 1);
    assert(read_with_start_read_image(argv[1]) == 1);
    assert(read_with_image_api(argv[2], 1) == 1);

    assert(read_with_row_loop(argv[3], 0) == 0);
    assert(read_with_row_loop(argv[3], 1) == 1);

    return 0;
}
