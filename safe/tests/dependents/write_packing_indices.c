#define _POSIX_C_SOURCE 200809L

#include <png.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static int write_fixture(const char *path) {
    FILE *fp = fopen(path, "wb");
    if (fp == NULL) {
        perror("fopen");
        return 1;
    }

    png_structp png_ptr = png_create_write_struct(PNG_LIBPNG_VER_STRING, NULL, NULL, NULL);
    if (png_ptr == NULL) {
        fclose(fp);
        fprintf(stderr, "png_create_write_struct failed\n");
        return 1;
    }

    png_infop info_ptr = png_create_info_struct(png_ptr);
    if (info_ptr == NULL) {
        png_destroy_write_struct(&png_ptr, NULL);
        fclose(fp);
        fprintf(stderr, "png_create_info_struct failed\n");
        return 1;
    }

    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        png_destroy_write_struct(&png_ptr, &info_ptr);
        fclose(fp);
        fprintf(stderr, "libpng signalled an error while writing packed fixture\n");
        return 1;
    }

    png_color palette[9];
    for (int i = 0; i < 9; ++i) {
        palette[i].red = (png_byte)(60 + i * 8);
        palette[i].green = (png_byte)(70 + i * 8);
        palette[i].blue = (png_byte)(80 + i * 8);
    }

    png_init_io(png_ptr, fp);
    png_set_IHDR(
        png_ptr,
        info_ptr,
        3,
        3,
        4,
        PNG_COLOR_TYPE_PALETTE,
        PNG_INTERLACE_NONE,
        PNG_COMPRESSION_TYPE_DEFAULT,
        PNG_FILTER_TYPE_DEFAULT);
    png_set_PLTE(png_ptr, info_ptr, palette, 9);
    png_set_packing(png_ptr);
    png_write_info(png_ptr, info_ptr);

    png_byte row0[3] = {0, 1, 2};
    png_byte row1[3] = {3, 4, 5};
    png_byte row2[3] = {6, 7, 8};
    png_bytep rows[3] = {row0, row1, row2};
    png_write_image(png_ptr, rows);
    png_write_end(png_ptr, info_ptr);

    png_destroy_write_struct(&png_ptr, &info_ptr);
    fclose(fp);
    return 0;
}

static int read_and_check_fixture(const char *path) {
    static const png_byte expected[9] = {0, 1, 2, 3, 4, 5, 6, 7, 8};

    FILE *fp = fopen(path, "rb");
    if (fp == NULL) {
        perror("fopen");
        return 1;
    }

    png_structp png_ptr = png_create_read_struct(PNG_LIBPNG_VER_STRING, NULL, NULL, NULL);
    if (png_ptr == NULL) {
        fclose(fp);
        fprintf(stderr, "png_create_read_struct failed\n");
        return 1;
    }

    png_infop info_ptr = png_create_info_struct(png_ptr);
    if (info_ptr == NULL) {
        png_destroy_read_struct(&png_ptr, NULL, NULL);
        fclose(fp);
        fprintf(stderr, "png_create_info_struct failed\n");
        return 1;
    }

    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
        fclose(fp);
        fprintf(stderr, "libpng signalled an error while reading packed fixture\n");
        return 1;
    }

    png_init_io(png_ptr, fp);
    png_read_info(png_ptr, info_ptr);

    if (png_get_image_width(png_ptr, info_ptr) != 3 ||
        png_get_image_height(png_ptr, info_ptr) != 3 ||
        png_get_color_type(png_ptr, info_ptr) != PNG_COLOR_TYPE_PALETTE ||
        png_get_bit_depth(png_ptr, info_ptr) != 4) {
        png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
        fclose(fp);
        fprintf(stderr, "unexpected packed fixture metadata\n");
        return 1;
    }

    png_set_packing(png_ptr);
    png_read_update_info(png_ptr, info_ptr);

    png_byte rows[3][3] = {{0}};
    png_bytep row_ptrs[3] = {rows[0], rows[1], rows[2]};
    png_read_image(png_ptr, row_ptrs);
    png_read_end(png_ptr, NULL);

    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
    fclose(fp);

    if (memcmp(rows, expected, sizeof(expected)) != 0) {
        fprintf(stderr, "packed write indices were not preserved:");
        for (size_t i = 0; i < sizeof(expected); ++i) {
            fprintf(stderr, " %u", (unsigned)((png_byte *)rows)[i]);
        }
        fprintf(stderr, "\n");
        return 1;
    }

    return 0;
}

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;

    char path[] = "/tmp/libpng-safe-write-packing-XXXXXX";
    int fd = mkstemp(path);
    if (fd < 0) {
        perror("mkstemp");
        return 1;
    }
    close(fd);

    int status = write_fixture(path);
    if (status == 0) {
        status = read_and_check_fixture(path);
    }
    unlink(path);

    return status;
}
