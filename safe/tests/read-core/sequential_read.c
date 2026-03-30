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

static int read_image_rows(const char *path, int expect_interlaced, int ignore_adler32) {
    FILE *fp = fopen(path, "rb");
    assert(fp != NULL);

    test_ctx ctx;
    memset(&ctx, 0, sizeof ctx);

    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, &ctx, error_cb, warning_cb);
    assert(png_ptr != NULL);
    png_infop info_ptr = png_create_info_struct(png_ptr);
    png_infop end_ptr = png_create_info_struct(png_ptr);
    assert(info_ptr != NULL);
    assert(end_ptr != NULL);

    if (setjmp(ctx.env) != 0) {
        png_destroy_read_struct(&png_ptr, &info_ptr, &end_ptr);
        fclose(fp);
        return 0;
    }

    png_set_benign_errors(png_ptr, 0);
    if (ignore_adler32) {
        (void)png_set_option(png_ptr, PNG_IGNORE_ADLER32, 1);
    }

    png_init_io(png_ptr, fp);
    png_read_info(png_ptr, info_ptr);

    png_uint_32 width = png_get_image_width(png_ptr, info_ptr);
    png_uint_32 height = png_get_image_height(png_ptr, info_ptr);
    assert(width > 0);
    assert(height > 0);

    int passes = png_set_interlace_handling(png_ptr);
    if (expect_interlaced) {
        assert(passes == PNG_INTERLACE_ADAM7_PASSES);
    } else {
        assert(passes == 1);
    }

    png_read_update_info(png_ptr, info_ptr);
    size_t rowbytes = png_get_rowbytes(png_ptr, info_ptr);
    assert(rowbytes > 0);

    png_bytep row = (png_bytep)malloc(rowbytes);
    assert(row != NULL);

    png_uint_32 rows_read = 0;
    for (int pass = 0; pass < passes; ++pass) {
        for (png_uint_32 y = 0; y < height; ++y) {
            png_read_row(png_ptr, row, NULL);
            ++rows_read;
        }
    }

    png_read_end(png_ptr, end_ptr);
    free(row);
    png_destroy_read_struct(&png_ptr, &info_ptr, &end_ptr);
    fclose(fp);

    assert(rows_read == height * (png_uint_32)passes);
    return 1;
}

int main(int argc, char **argv) {
    assert(argc == 4);

    assert(read_image_rows(argv[1], 0, 0) == 1);
    assert(read_image_rows(argv[2], 1, 0) == 1);

    assert(read_image_rows(argv[3], 0, 0) == 0);
    assert(read_image_rows(argv[3], 0, 1) == 1);

    return 0;
}
