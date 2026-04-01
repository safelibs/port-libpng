#include <png.h>
#include <setjmp.h>
#include <stdio.h>
#include <stdlib.h>

static jmp_buf app_jmpbuf;

static void app_error(png_structp png_ptr, png_const_charp message) {
    (void)png_ptr;
    fprintf(stderr, "custom error callback: %s\n", message ? message : "(null)");
    longjmp(app_jmpbuf, 1);
}

int main(int argc, char **argv) {
    if (argc != 2) {
        fprintf(stderr, "usage: %s <png>\n", argv[0]);
        return 2;
    }

    FILE *fp = fopen(argv[1], "rb");
    if (fp == NULL) {
        perror("fopen");
        return 1;
    }

    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, NULL, app_error, NULL);
    if (png_ptr == NULL) {
        fprintf(stderr, "png_create_read_struct failed\n");
        fclose(fp);
        return 1;
    }

    png_infop info_ptr = png_create_info_struct(png_ptr);
    if (info_ptr == NULL) {
        fprintf(stderr, "png_create_info_struct failed\n");
        png_destroy_read_struct(&png_ptr, NULL, NULL);
        fclose(fp);
        return 1;
    }

    if (setjmp(app_jmpbuf) != 0) {
        fprintf(stderr, "unexpected application longjmp while reading image\n");
        png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
        fclose(fp);
        return 1;
    }

    unsigned char signature[8];
    if (fread(signature, 1, sizeof(signature), fp) != sizeof(signature)) {
        fprintf(stderr, "failed to read PNG signature bytes\n");
        png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
        fclose(fp);
        return 1;
    }

    png_set_sig_bytes(png_ptr, (int)sizeof(signature));
    png_init_io(png_ptr, fp);
    png_read_info(png_ptr, info_ptr);

    png_uint_32 width = png_get_image_width(png_ptr, info_ptr);
    png_uint_32 height = png_get_image_height(png_ptr, info_ptr);
    png_size_t rowbytes = png_get_rowbytes(png_ptr, info_ptr);

    png_bytepp rows = malloc(sizeof(*rows) * height);
    if (rows == NULL) {
        fprintf(stderr, "failed to allocate row pointer table\n");
        png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
        fclose(fp);
        return 1;
    }

    for (png_uint_32 y = 0; y < height; ++y) {
        rows[y] = malloc(rowbytes);
        if (rows[y] == NULL) {
            fprintf(stderr, "failed to allocate row %u\n", y);
            for (png_uint_32 i = 0; i < y; ++i) {
                free(rows[i]);
            }
            free(rows);
            png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
            fclose(fp);
            return 1;
        }
    }

    png_read_image(png_ptr, rows);
    png_read_end(png_ptr, NULL);

    for (png_uint_32 y = 0; y < height; ++y) {
        free(rows[y]);
    }
    free(rows);
    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
    fclose(fp);

    if (width == 0 || height == 0 || rowbytes == 0) {
        fprintf(stderr, "unexpected decoded geometry\n");
        return 1;
    }

    return 0;
}
