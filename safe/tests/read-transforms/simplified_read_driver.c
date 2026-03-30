#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <png.h>

static unsigned char *read_file_bytes(const char *path, size_t *size_out) {
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

static png_bytep simplified_read_from_file(const char *path, png_uint_32 format,
    png_uint_32 *width_out, png_uint_32 *height_out, size_t *size_out) {
    png_image image;
    memset(&image, 0, sizeof image);
    image.version = PNG_IMAGE_VERSION;

    assert(png_image_begin_read_from_file(&image, path) != 0);
    image.format = format;

    size_t size = PNG_IMAGE_SIZE(image);
    png_bytep buffer = (png_bytep)malloc(size);
    assert(buffer != NULL);
    assert(png_image_finish_read(&image, NULL, buffer, 0, NULL) != 0);

    *width_out = image.width;
    *height_out = image.height;
    *size_out = size;
    return buffer;
}

static png_bytep simplified_read_from_stdio(const char *path, png_uint_32 format,
    png_uint_32 *width_out, png_uint_32 *height_out, size_t *size_out) {
    FILE *fp = fopen(path, "rb");
    assert(fp != NULL);

    png_image image;
    memset(&image, 0, sizeof image);
    image.version = PNG_IMAGE_VERSION;

    assert(png_image_begin_read_from_stdio(&image, fp) != 0);
    image.format = format;

    size_t size = PNG_IMAGE_SIZE(image);
    png_bytep buffer = (png_bytep)malloc(size);
    assert(buffer != NULL);
    assert(png_image_finish_read(&image, NULL, buffer, 0, NULL) != 0);

    fclose(fp);
    *width_out = image.width;
    *height_out = image.height;
    *size_out = size;
    return buffer;
}

static png_bytep simplified_read_from_memory(const char *path, png_uint_32 format,
    png_uint_32 *width_out, png_uint_32 *height_out, size_t *size_out) {
    size_t memory_size = 0;
    unsigned char *memory = read_file_bytes(path, &memory_size);

    png_image image;
    memset(&image, 0, sizeof image);
    image.version = PNG_IMAGE_VERSION;

    assert(png_image_begin_read_from_memory(&image, memory, memory_size) != 0);
    image.format = format;

    size_t size = PNG_IMAGE_SIZE(image);
    png_bytep buffer = (png_bytep)malloc(size);
    assert(buffer != NULL);
    assert(png_image_finish_read(&image, NULL, buffer, 0, NULL) != 0);

    free(memory);
    *width_out = image.width;
    *height_out = image.height;
    *size_out = size;
    return buffer;
}

static void run_entrypoint_consistency_case(const char *path) {
    png_uint_32 width_file = 0;
    png_uint_32 height_file = 0;
    png_uint_32 width_stdio = 0;
    png_uint_32 height_stdio = 0;
    png_uint_32 width_memory = 0;
    png_uint_32 height_memory = 0;
    size_t size_file = 0;
    size_t size_stdio = 0;
    size_t size_memory = 0;

    png_bytep file_buffer = simplified_read_from_file(
        path, PNG_FORMAT_RGBA, &width_file, &height_file, &size_file);
    png_bytep stdio_buffer = simplified_read_from_stdio(
        path, PNG_FORMAT_RGBA, &width_stdio, &height_stdio, &size_stdio);
    png_bytep memory_buffer = simplified_read_from_memory(
        path, PNG_FORMAT_RGBA, &width_memory, &height_memory, &size_memory);

    assert(width_file == width_stdio && width_file == width_memory);
    assert(height_file == height_stdio && height_file == height_memory);
    assert(size_file == size_stdio && size_file == size_memory);
    assert(memcmp(file_buffer, stdio_buffer, size_file) == 0);
    assert(memcmp(file_buffer, memory_buffer, size_file) == 0);

    free(file_buffer);
    free(stdio_buffer);
    free(memory_buffer);
}

static void run_message_semantics_case(const char *path) {
    size_t memory_size = 0;
    unsigned char *memory = read_file_bytes(path, &memory_size);

    png_image wrong_version;
    memset(&wrong_version, 0, sizeof wrong_version);
    wrong_version.version = PNG_IMAGE_VERSION + 1;
    assert(png_image_begin_read_from_memory(&wrong_version, memory, memory_size) == 0);
    assert(strstr(wrong_version.message, "incorrect PNG_IMAGE_VERSION") != NULL);

    png_image invalid_begin;
    memset(&invalid_begin, 0, sizeof invalid_begin);
    invalid_begin.version = PNG_IMAGE_VERSION;
    assert(png_image_begin_read_from_memory(&invalid_begin, NULL, 0) == 0);
    assert(strstr(invalid_begin.message, "invalid argument") != NULL);

    png_image invalid_finish;
    memset(&invalid_finish, 0, sizeof invalid_finish);
    invalid_finish.version = PNG_IMAGE_VERSION;
    assert(png_image_begin_read_from_memory(&invalid_finish, memory, memory_size) != 0);
    invalid_finish.format = PNG_FORMAT_RGBA;
    assert(png_image_finish_read(&invalid_finish, NULL, NULL, 0, NULL) == 0);
    assert(strstr(invalid_finish.message, "invalid argument") != NULL);

    free(memory);
}

static void run_image_free_case(const char *path) {
    size_t memory_size = 0;
    unsigned char *memory = read_file_bytes(path, &memory_size);

    png_image image;
    memset(&image, 0, sizeof image);
    image.version = PNG_IMAGE_VERSION;

    assert(png_image_begin_read_from_memory(&image, memory, memory_size) != 0);
    assert(image.opaque != NULL);
    png_image_free(&image);
    assert(image.opaque == NULL);

    free(memory);
}

static png_bytep read_with_stride(const char *path, png_int_32 row_stride,
    png_uint_32 *width_out, png_uint_32 *height_out, size_t *row_bytes_out) {
    png_image image;
    memset(&image, 0, sizeof image);
    image.version = PNG_IMAGE_VERSION;

    assert(png_image_begin_read_from_file(&image, path) != 0);
    image.format = PNG_FORMAT_RGBA;

    size_t row_bytes = PNG_IMAGE_ROW_STRIDE(image);
    size_t stride = row_bytes;
    if (row_stride != 0) {
        stride = (size_t)(row_stride < 0 ? -row_stride : row_stride);
    }

    png_bytep buffer = (png_bytep)calloc(stride, image.height);
    assert(buffer != NULL);
    assert(png_image_finish_read(&image, NULL, buffer, row_stride, NULL) != 0);

    *width_out = image.width;
    *height_out = image.height;
    *row_bytes_out = row_bytes;
    return buffer;
}

static void run_stride_regression_case(const char *path) {
    png_uint_32 width = 0;
    png_uint_32 height = 0;
    size_t row_bytes = 0;

    png_bytep baseline = read_with_stride(path, 0, &width, &height, &row_bytes);

    png_bytep padded = read_with_stride(
        path, (png_int_32)row_bytes + 17, &width, &height, &row_bytes);
    for (png_uint_32 y = 0; y < height; ++y) {
        assert(memcmp(
                   baseline + y * row_bytes,
                   padded + y * (row_bytes + 17),
                   row_bytes) == 0);
    }

    png_bytep bottom_up = read_with_stride(
        path, -((png_int_32)row_bytes + 19), &width, &height, &row_bytes);
    for (png_uint_32 y = 0; y < height; ++y) {
        size_t offset = (size_t)(height - 1 - y) * (row_bytes + 19);
        assert(memcmp(baseline + y * row_bytes, bottom_up + offset, row_bytes) == 0);
    }

    free(baseline);
    free(padded);
    free(bottom_up);
}

int main(int argc, char **argv) {
    assert(argc == 3);

    run_entrypoint_consistency_case(argv[1]);
    run_message_semantics_case(argv[1]);
    run_image_free_case(argv[1]);
    run_stride_regression_case(argv[2]);
    return 0;
}
