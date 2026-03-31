#include <assert.h>
#include <setjmp.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <png.h>

static size_t malloc_calls = 0;
static size_t free_calls = 0;
static size_t error_calls = 0;
static size_t warning_calls = 0;
static png_voidp destroy_error_target = NULL;
static png_voidp destroy_expected_mem_ptr = NULL;
static int create_error_enabled = 0;
static int destroy_context_check_enabled = 0;
static int destroy_context_checked = 0;
static int destroy_error_enabled = 0;

static void *tracked_malloc(png_structp png_ptr, png_alloc_size_t size) {
    if (create_error_enabled) {
        create_error_enabled = 0;
        png_error(png_ptr, "create alloc error");
        return NULL;
    }

    ++malloc_calls;
    return malloc(size);
}

static void tracked_free(png_structp png_ptr, png_voidp ptr) {
    ++free_calls;

    if (destroy_context_check_enabled && ptr == destroy_error_target) {
        assert(png_ptr != ptr);
        assert(png_get_mem_ptr(png_ptr) == destroy_expected_mem_ptr);
        destroy_context_checked = 1;
    }

    if (destroy_error_enabled && ptr == destroy_error_target) {
        destroy_error_enabled = 0;
        png_error(png_ptr, "destroy free error");
    }

    free(ptr);
}

static void noop_error(png_structp png_ptr, png_const_charp message) {
    (void)message;
    ++error_calls;
    png_longjmp(png_ptr, 11);
}

static void noop_warning(png_structp png_ptr, png_const_charp message) {
    (void)png_ptr;
    (void)message;
    ++warning_calls;
}

static void read_data(png_structp png_ptr, png_bytep data, size_t length) {
    (void)png_ptr;
    memset(data, 0, length);
}

static void write_data(png_structp png_ptr, png_bytep data, size_t length) {
    (void)png_ptr;
    (void)data;
    (void)length;
}

static void flush_data(png_structp png_ptr) {
    (void)png_ptr;
}

static void read_status(png_structp png_ptr, png_uint_32 row_number, int pass) {
    (void)png_ptr;
    (void)row_number;
    (void)pass;
}

static void write_status(png_structp png_ptr, png_uint_32 row_number, int pass) {
    (void)png_ptr;
    (void)row_number;
    (void)pass;
}

static void progressive_info(png_structp png_ptr, png_infop info_ptr) {
    (void)png_ptr;
    (void)info_ptr;
}

static void progressive_row(png_structp png_ptr, png_bytep row, png_uint_32 row_num,
                            int pass) {
    (void)png_ptr;
    (void)row;
    (void)row_num;
    (void)pass;
}

static void progressive_end(png_structp png_ptr, png_infop info_ptr) {
    (void)png_ptr;
    (void)info_ptr;
}

static int user_chunk(png_structp png_ptr, png_unknown_chunkp chunk) {
    (void)png_ptr;
    (void)chunk;
    return 0;
}

static void user_transform(png_structp png_ptr, png_row_infop row_info,
                           png_bytep data) {
    (void)png_ptr;
    (void)row_info;
    (void)data;
}

int main(void) {
    int error_cookie = 17;
    int replacement_error_cookie = 19;
    int mem_cookie = 23;
    int replacement_mem_cookie = 29;
    size_t errors_before = error_calls;

    create_error_enabled = 1;
    assert(png_create_read_struct_2(PNG_LIBPNG_VER_STRING, &error_cookie, noop_error,
                                    noop_warning, &mem_cookie, tracked_malloc,
                                    tracked_free) == NULL);
    assert(error_calls == errors_before + 1);

    png_structp read_ptr = png_create_read_struct_2(
        PNG_LIBPNG_VER_STRING, &error_cookie, noop_error, noop_warning,
        &mem_cookie, tracked_malloc, tracked_free);
    assert(read_ptr != NULL);
    assert(png_get_error_ptr(read_ptr) == &error_cookie);
    assert(png_get_mem_ptr(read_ptr) == &mem_cookie);
    assert(malloc_calls > 0);

    png_set_error_fn(read_ptr, &replacement_error_cookie, noop_error, noop_warning);
    assert(png_get_error_ptr(read_ptr) == &replacement_error_cookie);

    png_set_mem_fn(read_ptr, &replacement_mem_cookie, tracked_malloc, tracked_free);
    assert(png_get_mem_ptr(read_ptr) == &replacement_mem_cookie);

    int read_cookie = 31;
    png_set_read_fn(read_ptr, &read_cookie, read_data);
    assert(png_get_io_ptr(read_ptr) == &read_cookie);
    png_set_read_status_fn(read_ptr, read_status);

    int chunk_cookie = 41;
    png_set_read_user_chunk_fn(read_ptr, &chunk_cookie, user_chunk);
    assert(png_get_user_chunk_ptr(read_ptr) == &chunk_cookie);

    int transform_cookie = 59;
    png_set_read_user_transform_fn(read_ptr, user_transform);
    png_set_user_transform_info(read_ptr, &transform_cookie, 8, 3);
    assert(png_get_user_transform_ptr(read_ptr) == &transform_cookie);

    int progressive_cookie = 67;
    png_set_progressive_read_fn(read_ptr, &progressive_cookie, progressive_info,
                                progressive_row, progressive_end);
    assert(png_get_progressive_ptr(read_ptr) == &progressive_cookie);

    size_t oversized_jmp_buf_size = sizeof(jmp_buf) + 32u;
    jmp_buf *jmp = png_set_longjmp_fn(read_ptr, longjmp, oversized_jmp_buf_size);
    assert(jmp != NULL);
    assert(png_set_longjmp_fn(read_ptr, longjmp, oversized_jmp_buf_size) == jmp);
    errors_before = error_calls;
    if (setjmp(*jmp) == 0) {
        png_error(read_ptr, "expected jump");
        assert(!"png_error should longjmp");
    }
    assert(error_calls == errors_before + 1);

    destroy_error_target = read_ptr;
    destroy_expected_mem_ptr = &replacement_mem_cookie;
    destroy_context_check_enabled = 1;
    destroy_error_enabled = 1;
    if (setjmp(*jmp) == 0) {
        png_destroy_read_struct(&read_ptr, NULL, NULL);
        assert(!"png_destroy_read_struct should longjmp from free_fn");
    }
    assert(destroy_context_checked == 1);
    assert(read_ptr == NULL);
    assert(error_calls == errors_before + 2);
    destroy_error_target = NULL;
    destroy_expected_mem_ptr = NULL;
    destroy_context_check_enabled = 0;

    png_structp write_ptr =
        png_create_write_struct(PNG_LIBPNG_VER_STRING, NULL, noop_error, noop_warning);
    assert(write_ptr != NULL);

    png_init_io(write_ptr, stdout);
    assert(png_get_io_ptr(write_ptr) == stdout);

    int write_cookie = 73;
    png_set_write_fn(write_ptr, &write_cookie, write_data, flush_data);
    assert(png_get_io_ptr(write_ptr) == &write_cookie);
    png_set_write_status_fn(write_ptr, write_status);
    png_set_write_user_transform_fn(write_ptr, user_transform);
    png_set_user_transform_info(write_ptr, &transform_cookie, 16, 4);
    assert(png_get_user_transform_ptr(write_ptr) == &transform_cookie);

    jmp_buf *write_jmp = png_set_longjmp_fn(write_ptr, longjmp, sizeof(jmp_buf));
    assert(write_jmp != NULL);
    size_t warnings_before = warning_calls;
    errors_before = error_calls;
    if (setjmp(*write_jmp) == 0) {
        png_benign_error(write_ptr, "expected benign write error");
        assert(!"png_benign_error on default write struct should longjmp");
    }
    assert(warning_calls == warnings_before);
    assert(error_calls == errors_before + 1);

    png_set_benign_errors(write_ptr, 1);
    warnings_before = warning_calls;
    errors_before = error_calls;
    if (setjmp(*write_jmp) == 0) {
        png_benign_error(write_ptr, "expected benign warning");
        assert(warning_calls == warnings_before + 1);
        assert(error_calls == errors_before);
    } else {
        assert(!"png_benign_error after png_set_benign_errors should warn");
    }

    png_destroy_write_struct(&write_ptr, NULL);
    assert(write_ptr == NULL);
    assert(free_calls > 0);
    return 0;
}
