#define _POSIX_C_SOURCE 200809L

#include <png.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

/*
 * Reproduces the netpbm pamtopng -text usage failure. pamtopng builds an array
 * of png_text entries with compression == PNG_TEXT_COMPRESSION_zTXt but with
 * empty text bodies. Upstream png_set_text_2 normalizes those to
 * PNG_TEXT_COMPRESSION_NONE so the chunks are emitted as tEXt; the validator
 * test asserts on the resulting tEXt chunk count.
 */

static int count_chunks(const char *path, const char *want_type) {
    FILE *fp = fopen(path, "rb");
    if (fp == NULL) {
        perror("fopen");
        return -1;
    }
    unsigned char sig[8];
    if (fread(sig, 1, sizeof sig, fp) != sizeof sig) {
        fclose(fp);
        return -1;
    }
    int count = 0;
    for (;;) {
        unsigned char hdr[8];
        if (fread(hdr, 1, sizeof hdr, fp) != sizeof hdr) {
            break;
        }
        unsigned long length =
            ((unsigned long)hdr[0] << 24) | ((unsigned long)hdr[1] << 16) |
            ((unsigned long)hdr[2] << 8) | (unsigned long)hdr[3];
        char ctype[5] = {hdr[4], hdr[5], hdr[6], hdr[7], 0};
        if (memcmp(ctype, want_type, 4) == 0) {
            ++count;
        }
        if (fseek(fp, (long)(length + 4), SEEK_CUR) != 0) {
            break;
        }
        if (memcmp(ctype, "IEND", 4) == 0) {
            break;
        }
    }
    fclose(fp);
    return count;
}

static int write_fixture(const char *path) {
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
    png_set_IHDR(png_ptr, info_ptr, 4, 4, 8, PNG_COLOR_TYPE_RGB,
                 PNG_INTERLACE_NONE, PNG_COMPRESSION_TYPE_DEFAULT,
                 PNG_FILTER_TYPE_DEFAULT);

    /* Mirror what pamtopng does for an English -text file: every chunk is
     * marked as zTXt but the body is empty because the netpbm parser puts the
     * payload on a continuation line that has not been read yet. The validator
     * counts the resulting chunks via raw byte inspection.
     */
    png_text texts[2];
    memset(texts, 0, sizeof texts);
    texts[0].compression = PNG_TEXT_COMPRESSION_zTXt;
    texts[0].key = (png_charp)"Title";
    texts[0].text = (png_charp)"";
    texts[0].text_length = 0;
    texts[1].compression = PNG_TEXT_COMPRESSION_zTXt;
    texts[1].key = (png_charp)"Author";
    texts[1].text = (png_charp)"";
    texts[1].text_length = 0;
    png_set_text(png_ptr, info_ptr, texts, 2);

    png_write_info(png_ptr, info_ptr);
    png_byte row[12] = {0};
    for (int y = 0; y < 4; ++y) {
        png_write_row(png_ptr, row);
    }
    png_write_end(png_ptr, NULL);
    png_destroy_write_struct(&png_ptr, &info_ptr);
    fclose(fp);
    return 0;
}

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;

    char path[] = "/tmp/libpng-safe-text-empty-XXXXXX";
    int fd = mkstemp(path);
    if (fd < 0) {
        perror("mkstemp");
        return 1;
    }
    close(fd);

    int status = write_fixture(path);
    if (status == 0) {
        int text_count = count_chunks(path, "tEXt");
        int ztxt_count = count_chunks(path, "zTXt");
        if (text_count != 2 || ztxt_count != 0) {
            fprintf(stderr,
                    "expected 2 tEXt and 0 zTXt chunks, got tEXt=%d zTXt=%d\n",
                    text_count, ztxt_count);
            status = 1;
        }
    }
    unlink(path);
    return status;
}
