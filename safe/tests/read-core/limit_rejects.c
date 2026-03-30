#include <assert.h>
#include <setjmp.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <png.h>

typedef struct {
    const unsigned char *data;
    size_t size;
    size_t offset;
    jmp_buf env;
} memory_source;

typedef struct {
    jmp_buf env;
} file_source;

static const unsigned char two_text_png[] = {
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
    0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
    0x08, 0x00, 0x00, 0x00, 0x00, 0x3a, 0x7e, 0x9b, 0x55, 0x00, 0x00, 0x00,
    0x09, 0x74, 0x45, 0x58, 0x74, 0x6f, 0x6e, 0x65, 0x00, 0x61, 0x6c, 0x70,
    0x68, 0x61, 0x69, 0xd1, 0xd8, 0x3a, 0x00, 0x00, 0x00, 0x08, 0x74, 0x45,
    0x58, 0x74, 0x74, 0x77, 0x6f, 0x00, 0x62, 0x65, 0x74, 0x61, 0xaa, 0xa8,
    0x8a, 0x13, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c,
    0x63, 0x60, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0x48, 0xaf, 0xa4, 0x71,
    0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
};

static const unsigned char low_malloc_ztxt_png[] = {
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
    0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
    0x08, 0x00, 0x00, 0x00, 0x00, 0x3a, 0x7e, 0x9b, 0x55, 0x00, 0x00, 0x00,
    0x11, 0x7a, 0x54, 0x58, 0x74, 0x7a, 0x69, 0x70, 0x00, 0x00, 0x78, 0x9c,
    0x73, 0x74, 0xa4, 0x2d, 0x00, 0x00, 0x9e, 0xcc, 0x18, 0x61, 0xae, 0x6a,
    0x2f, 0xcc, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c,
    0x63, 0x60, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0x48, 0xaf, 0xa4, 0x71,
    0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
};

static void error_cb(png_structp png_ptr, png_const_charp message) {
    (void)message;
    void *ctx = png_get_error_ptr(png_ptr);
    assert(ctx != NULL);
    longjmp(*(jmp_buf *)ctx, 1);
}

static void warning_cb(png_structp png_ptr, png_const_charp message) {
    (void)png_ptr;
    (void)message;
}

static void read_memory(png_structp png_ptr, png_bytep out, size_t length) {
    memory_source *source = (memory_source *)png_get_io_ptr(png_ptr);
    assert(source != NULL);
    if (source->offset + length > source->size) {
        png_error(png_ptr, "short read");
    }
    memcpy(out, source->data + source->offset, length);
    source->offset += length;
}

static void expect_file_rejection(const char *path) {
    FILE *fp = fopen(path, "rb");
    assert(fp != NULL);

    file_source source;
    memset(&source, 0, sizeof source);

    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, &source.env, error_cb, warning_cb);
    assert(png_ptr != NULL);
    png_infop info_ptr = png_create_info_struct(png_ptr);
    assert(info_ptr != NULL);

    if (setjmp(source.env) == 0) {
        png_set_benign_errors(png_ptr, 0);
        png_init_io(png_ptr, fp);
        png_read_info(png_ptr, info_ptr);
        assert(!"expected read_info rejection");
    }

    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
    fclose(fp);
}

static void expect_memory_rejection(
    const unsigned char *data,
    size_t size,
    png_alloc_size_t malloc_max,
    png_uint_32 cache_max
) {
    memory_source source;
    memset(&source, 0, sizeof source);
    source.data = data;
    source.size = size;

    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, &source.env, error_cb, warning_cb);
    assert(png_ptr != NULL);
    png_infop info_ptr = png_create_info_struct(png_ptr);
    assert(info_ptr != NULL);

    if (setjmp(source.env) == 0) {
        png_set_benign_errors(png_ptr, 0);
        if (malloc_max != 0) {
            png_set_chunk_malloc_max(png_ptr, malloc_max);
        }
        if (cache_max != 0) {
            png_set_chunk_cache_max(png_ptr, cache_max);
        }
        png_set_read_fn(png_ptr, &source, read_memory);
        png_read_info(png_ptr, info_ptr);
        assert(!"expected in-memory rejection");
    }

    assert(source.offset < source.size);
    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
}

int main(int argc, char **argv) {
    assert(argc == 5);

    expect_file_rejection(argv[1]);
    expect_file_rejection(argv[2]);
    expect_file_rejection(argv[3]);
    expect_file_rejection(argv[4]);

    expect_memory_rejection(low_malloc_ztxt_png, sizeof low_malloc_ztxt_png, 64, 0);
    expect_memory_rejection(two_text_png, sizeof two_text_png, 0, 3);
    return 0;
}
