#include <assert.h>
#include <png.h>

int main(void) {
    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, NULL, NULL, NULL);
    assert(png_ptr != NULL);

    assert(png_get_user_width_max(png_ptr) == PNG_USER_WIDTH_MAX);
    assert(png_get_user_height_max(png_ptr) == PNG_USER_HEIGHT_MAX);
    assert(png_get_chunk_cache_max(png_ptr) == PNG_USER_CHUNK_CACHE_MAX);
    assert(png_get_chunk_malloc_max(png_ptr) == PNG_USER_CHUNK_MALLOC_MAX);

    png_set_user_limits(png_ptr, 123u, 456u);
    assert(png_get_user_width_max(png_ptr) == 123u);
    assert(png_get_user_height_max(png_ptr) == 456u);

    png_set_chunk_cache_max(png_ptr, 77u);
    png_set_chunk_malloc_max(png_ptr, 99u);
    assert(png_get_chunk_cache_max(png_ptr) == 77u);
    assert(png_get_chunk_malloc_max(png_ptr) == 99u);

    assert(png_set_option(png_ptr, PNG_MAXIMUM_INFLATE_WINDOW, 1) ==
           PNG_OPTION_UNSET);
    assert(png_set_option(png_ptr, PNG_MAXIMUM_INFLATE_WINDOW, 0) ==
           PNG_OPTION_ON);
    assert(png_set_option(png_ptr, PNG_MAXIMUM_INFLATE_WINDOW, 0) ==
           PNG_OPTION_OFF);
    assert(png_set_option(png_ptr, PNG_OPTION_NEXT, 1) == PNG_OPTION_INVALID);

    png_set_sig_bytes(png_ptr, 4);
    png_set_sig_bytes(png_ptr, -1);

    png_infop info_ptr = png_create_info_struct(png_ptr);
    assert(info_ptr != NULL);
    assert(png_get_palette_max(png_ptr, info_ptr) == 0);
    png_set_rows(png_ptr, info_ptr, NULL);
    png_data_freer(png_ptr, info_ptr, PNG_DESTROY_WILL_FREE_DATA, PNG_FREE_ROWS);
    png_free_data(png_ptr, info_ptr, PNG_FREE_ROWS, 0);
    png_set_check_for_invalid_index(png_ptr, 0);
    assert(png_get_palette_max(png_ptr, info_ptr) == -1);
    png_set_check_for_invalid_index(png_ptr, 1);
    assert(png_get_palette_max(png_ptr, info_ptr) == 0);

    png_set_benign_errors(png_ptr, 1);
    png_set_benign_errors(png_ptr, 0);

    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
    assert(png_ptr == NULL);
    assert(info_ptr == NULL);
    return 0;
}
