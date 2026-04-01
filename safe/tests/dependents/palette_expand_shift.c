#include <png.h>
#include <stdio.h>
#include <stdlib.h>

static int channel_limit(png_color_8 sig_bit, png_byte color_type, int channel) {
    int bits = 8;
    switch (color_type) {
    case PNG_COLOR_TYPE_GRAY:
        bits = sig_bit.gray;
        break;
    case PNG_COLOR_TYPE_RGB:
        if (channel == 0) {
            bits = sig_bit.red;
        } else if (channel == 1) {
            bits = sig_bit.green;
        } else {
            bits = sig_bit.blue;
        }
        break;
    default:
        return 255;
    }

    if (bits <= 0 || bits > 8) {
        return 255;
    }
    return (1 << bits) - 1;
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

    png_structp png_ptr = png_create_read_struct(PNG_LIBPNG_VER_STRING, NULL, NULL, NULL);
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

    if (setjmp(png_jmpbuf(png_ptr)) != 0) {
        fprintf(stderr, "libpng signalled an error while decoding the dependent reproducer\n");
        png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
        fclose(fp);
        return 1;
    }

    png_init_io(png_ptr, fp);
    png_read_info(png_ptr, info_ptr);

    png_color_8p sig_bit_ptr = NULL;
    if ((png_get_sBIT(png_ptr, info_ptr, &sig_bit_ptr) & PNG_INFO_sBIT) == 0 ||
        sig_bit_ptr == NULL) {
        fprintf(stderr, "expected an sBIT chunk in the fixture\n");
        png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
        fclose(fp);
        return 1;
    }

    png_color_8 sig_bit = *sig_bit_ptr;
    png_byte input_color_type = png_get_color_type(png_ptr, info_ptr);
    if (input_color_type == PNG_COLOR_TYPE_PALETTE) {
        png_set_expand(png_ptr);
    }
    if ((input_color_type & PNG_COLOR_MASK_ALPHA) != 0) {
        png_set_strip_alpha(png_ptr);
    }
    png_set_shift(png_ptr, &sig_bit);
    png_set_interlace_handling(png_ptr);
    png_read_update_info(png_ptr, info_ptr);

    png_uint_32 width = png_get_image_width(png_ptr, info_ptr);
    png_uint_32 height = png_get_image_height(png_ptr, info_ptr);
    png_byte color_type = png_get_color_type(png_ptr, info_ptr);
    png_byte bit_depth = png_get_bit_depth(png_ptr, info_ptr);
    png_size_t rowbytes = png_get_rowbytes(png_ptr, info_ptr);

    int channels = 0;
    if (color_type == PNG_COLOR_TYPE_GRAY) {
        channels = 1;
    } else if (color_type == PNG_COLOR_TYPE_RGB) {
        channels = 3;
    } else {
        fprintf(stderr, "unexpected shifted output color type %u\n", (unsigned)color_type);
        png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
        fclose(fp);
        return 1;
    }

    if (bit_depth != 8 || rowbytes != width * (png_uint_32)channels) {
        fprintf(stderr, "unexpected expanded image layout\n");
        png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
        fclose(fp);
        return 1;
    }

    png_bytep row = malloc(rowbytes);
    if (row == NULL) {
        fprintf(stderr, "failed to allocate row buffer\n");
        png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
        fclose(fp);
        return 1;
    }

    int saw_nonzero = 0;
    for (png_uint_32 y = 0; y < height; ++y) {
        png_read_row(png_ptr, row, NULL);
        for (png_uint_32 x = 0; x < width; ++x) {
            for (int channel = 0; channel < channels; ++channel) {
                int sample = row[x * (png_uint_32)channels + (png_uint_32)channel];
                if (sample != 0) {
                    saw_nonzero = 1;
                }
                if (sample > channel_limit(sig_bit, color_type, channel)) {
                    fprintf(
                        stderr,
                        "shifted sample exceeded sBIT limit at (%u,%u) channel %d: %d\n",
                        y,
                        x,
                        channel,
                        sample);
                    free(row);
                    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
                    fclose(fp);
                    return 1;
                }
            }
        }
    }

    png_read_end(png_ptr, NULL);
    free(row);
    png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
    fclose(fp);

    if (!saw_nonzero) {
        fprintf(stderr, "fixture did not exercise non-zero palette samples\n");
        return 1;
    }

    return 0;
}
