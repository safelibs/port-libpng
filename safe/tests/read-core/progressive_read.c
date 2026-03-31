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
    int info_called;
    int end_called;
    int paused_once;
    int skip_checked;
    png_uint_32 width;
    png_uint_32 height;
    size_t rowbytes;
    png_bytep image;
    png_uint_32 rows_seen;
} progressive_ctx;

static const unsigned char zero_length_first_idat_png[] = {
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
    0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
    0x08, 0x00, 0x00, 0x00, 0x00, 0x3a, 0x7e, 0x9b, 0x55, 0x00, 0x00, 0x00,
    0x00, 0x49, 0x44, 0x41, 0x54, 0x35, 0xaf, 0x06, 0x1e, 0x00, 0x00, 0x00,
    0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x60, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x01, 0x48, 0xaf, 0xa4, 0x71, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
};

static void error_cb(png_structp png_ptr, png_const_charp message) {
    if (message != NULL) {
        fprintf(stderr, "progressive error: %s\n", message);
    }
    (void)message;
    progressive_ctx *ctx = (progressive_ctx *)png_get_error_ptr(png_ptr);
    assert(ctx != NULL);
    longjmp(ctx->env, 1);
}

static void warning_cb(png_structp png_ptr, png_const_charp message) {
    if (message != NULL) {
        fprintf(stderr, "progressive warning: %s\n", message);
    }
    progressive_ctx *ctx = (progressive_ctx *)png_get_error_ptr(png_ptr);
    assert(ctx != NULL);
    ++ctx->warnings;
}

static void info_cb(png_structp png_ptr, png_infop info_ptr) {
    progressive_ctx *ctx = (progressive_ctx *)png_get_progressive_ptr(png_ptr);
    assert(ctx != NULL);
    assert(info_ptr != NULL);

    ctx->info_called = 1;
    png_start_read_image(png_ptr);
    ctx->width = png_get_image_width(png_ptr, info_ptr);
    ctx->height = png_get_image_height(png_ptr, info_ptr);
    ctx->rowbytes = png_get_rowbytes(png_ptr, info_ptr);
    assert(ctx->width > 0);
    assert(ctx->height > 0);
    assert(ctx->rowbytes > 0);
    ctx->image = (png_bytep)calloc(ctx->height, ctx->rowbytes);
    assert(ctx->image != NULL);

    if (!ctx->paused_once) {
        (void)png_process_data_pause(png_ptr, 1);
        ctx->paused_once = 1;
    }
}

static void row_cb(png_structp png_ptr, png_bytep new_row, png_uint_32 row_num, int pass) {
    (void)pass;
    progressive_ctx *ctx = (progressive_ctx *)png_get_progressive_ptr(png_ptr);
    assert(ctx != NULL);
    assert(ctx->image != NULL);
    assert(row_num < ctx->height);

    png_bytep dst = ctx->image + row_num * ctx->rowbytes;
    png_progressive_combine_row(png_ptr, dst, new_row);
    ++ctx->rows_seen;

}

static void end_cb(png_structp png_ptr, png_infop info_ptr) {
    (void)png_ptr;
    (void)info_ptr;
    progressive_ctx *ctx = (progressive_ctx *)png_get_progressive_ptr(png_ptr);
    assert(ctx != NULL);
    ctx->end_called = 1;
}

static unsigned char *read_file(const char *path, size_t *size_out) {
    FILE *fp = fopen(path, "rb");
    assert(fp != NULL);
    assert(fseek(fp, 0, SEEK_END) == 0);
    long size = ftell(fp);
    assert(size >= 0);
    assert(fseek(fp, 0, SEEK_SET) == 0);

    unsigned char *data = (unsigned char *)malloc((size_t)size);
    assert(data != NULL);
    assert(fread(data, 1, (size_t)size, fp) == (size_t)size);
    fclose(fp);
    *size_out = (size_t)size;
    return data;
}

static void run_progressive_stream(const unsigned char *data, size_t size, int use_pause_and_skip) {
    progressive_ctx ctx;
    memset(&ctx, 0, sizeof ctx);

    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, &ctx, error_cb, warning_cb);
    assert(png_ptr != NULL);
    png_infop info_ptr = png_create_info_struct(png_ptr);
    assert(info_ptr != NULL);

    if (setjmp(ctx.env) != 0) {
        free(ctx.image);
        png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
        assert(0 && "progressive decode unexpectedly failed");
    }

    png_set_progressive_read_fn(png_ptr, &ctx, info_cb, row_cb, end_cb);

    size_t offset = 0;
    while (offset < size) {
        size_t chunk = size - offset;
        if (chunk > 17) {
            chunk = 17;
        }

        png_process_data(png_ptr, info_ptr, (png_bytep)(data + offset), chunk);
        offset += chunk;

        if (use_pause_and_skip && !ctx.skip_checked) {
            assert(png_process_data_skip(png_ptr) == 0);
            ctx.skip_checked = 1;
        }
    }

    assert(png_process_data_pause(png_ptr, 0) == 0);
    assert(ctx.info_called == 1);
    assert(ctx.end_called == 1);
    assert(ctx.rows_seen > 0);
    if (use_pause_and_skip) {
        assert(ctx.paused_once == 1);
        assert(ctx.skip_checked == 1);
        assert(ctx.warnings >= 1);
    }

    free(ctx.image);
    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
}

int main(int argc, char **argv) {
    assert(argc == 2);

    size_t size = 0;
    unsigned char *data = read_file(argv[1], &size);
    run_progressive_stream(data, size, 1);
    free(data);

    run_progressive_stream(zero_length_first_idat_png, sizeof zero_length_first_idat_png, 0);
    return 0;
}
