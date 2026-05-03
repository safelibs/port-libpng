#define _POSIX_C_SOURCE 200809L

#include <png.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

/*
 * Reproduces the netpbm pngtopam -alphapam crash on a palette PNG with tRNS.
 * pngx_trns() in netpbm calls png_get_tRNS and unconditionally dereferences
 * the trans_color out-pointer. Upstream libpng always sets *trans_color to
 * &info_ptr->trans_color (even for palette images, where the struct is
 * effectively unused), so the dereference is safe. The safe wrapper used to
 * write NULL into *trans_color, which segfaulted netpbm.
 */

static int write_palette_trns(const char *path) {
    FILE *fp = fopen(path, "wb");
    if (fp == NULL) {
        perror("fopen");
        return 1;
    }
    png_structp png_ptr =
        png_create_write_struct(PNG_LIBPNG_VER_STRING, NULL, NULL, NULL);
    if (png_ptr == NULL) {
        fclose(fp);
        return 1;
    }
    png_infop info_ptr = png_create_info_struct(png_ptr);
    if (info_ptr == NULL) {
        png_destroy_write_struct(&png_ptr, NULL);
        fclose(fp);
        return 1;
    }
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        png_destroy_write_struct(&png_ptr, &info_ptr);
        fclose(fp);
        return 1;
    }

    png_init_io(png_ptr, fp);
    png_set_IHDR(png_ptr, info_ptr, 4, 4, 4, PNG_COLOR_TYPE_PALETTE,
                 PNG_INTERLACE_NONE, PNG_COMPRESSION_TYPE_DEFAULT,
                 PNG_FILTER_TYPE_DEFAULT);

    png_color palette[3];
    memset(palette, 0, sizeof palette);
    palette[0].red = 0;
    palette[0].green = 0;
    palette[0].blue = 0;
    palette[1].red = 255;
    palette[1].green = 255;
    palette[1].blue = 255;
    palette[2].red = 255;
    palette[2].green = 0;
    palette[2].blue = 255;
    png_set_PLTE(png_ptr, info_ptr, palette, 3);

    png_byte trans[3] = {255, 255, 0};
    png_set_tRNS(png_ptr, info_ptr, trans, 3, NULL);

    png_set_packing(png_ptr);
    png_write_info(png_ptr, info_ptr);

    png_byte row[4] = {0, 1, 2, 1};
    for (int y = 0; y < 4; ++y) {
        png_write_row(png_ptr, row);
    }
    png_write_end(png_ptr, NULL);
    png_destroy_write_struct(&png_ptr, &info_ptr);
    fclose(fp);
    return 0;
}

static int read_and_call_get_trns_like_netpbm(const char *path) {
    FILE *fp = fopen(path, "rb");
    if (fp == NULL) {
        perror("fopen");
        return 1;
    }
    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, NULL, NULL, NULL);
    if (png_ptr == NULL) {
        fclose(fp);
        return 1;
    }
    png_infop info_ptr = png_create_info_struct(png_ptr);
    if (info_ptr == NULL) {
        png_destroy_read_struct(&png_ptr, NULL, NULL);
        fclose(fp);
        return 1;
    }
    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
        fclose(fp);
        return 1;
    }

    png_init_io(png_ptr, fp);
    png_read_info(png_ptr, info_ptr);

    if (png_get_color_type(png_ptr, info_ptr) != PNG_COLOR_TYPE_PALETTE) {
        fprintf(stderr, "fixture is not palette\n");
        png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
        fclose(fp);
        return 1;
    }
    if ((png_get_valid(png_ptr, info_ptr, PNG_INFO_tRNS) & PNG_INFO_tRNS) == 0) {
        fprintf(stderr, "fixture is missing tRNS\n");
        png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
        fclose(fp);
        return 1;
    }

    png_bytep trans_alpha = NULL;
    int num_trans = 0;
    png_color_16p trans_color = NULL;
    png_get_tRNS(png_ptr, info_ptr, &trans_alpha, &num_trans, &trans_color);

    /* netpbm dereferences trans_color unconditionally. Upstream libpng
     * guarantees this pointer is non-NULL when tRNS is valid.
     */
    if (trans_color == NULL) {
        fprintf(stderr, "png_get_tRNS returned NULL trans_color for palette image\n");
        png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
        fclose(fp);
        return 1;
    }
    png_color_16 deref = *trans_color;
    (void)deref;

    if (num_trans != 3 || trans_alpha == NULL) {
        fprintf(stderr, "expected num_trans=3 and non-NULL trans_alpha, got %d %p\n",
                num_trans, (void *)trans_alpha);
        png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
        fclose(fp);
        return 1;
    }
    if (trans_alpha[0] != 255 || trans_alpha[1] != 255 || trans_alpha[2] != 0) {
        fprintf(stderr, "trans_alpha values were not preserved: %u %u %u\n",
                trans_alpha[0], trans_alpha[1], trans_alpha[2]);
        png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
        fclose(fp);
        return 1;
    }

    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
    fclose(fp);
    return 0;
}

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;

    char path[] = "/tmp/libpng-safe-trns-color-XXXXXX";
    int fd = mkstemp(path);
    if (fd < 0) {
        perror("mkstemp");
        return 1;
    }
    close(fd);

    int status = write_palette_trns(path);
    if (status == 0) {
        status = read_and_call_get_trns_like_netpbm(path);
    }
    unlink(path);
    return status;
}
