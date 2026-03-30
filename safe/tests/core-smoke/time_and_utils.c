#include <assert.h>
#include <string.h>
#include <time.h>

#include <png.h>

int main(void) {
    png_structp png_ptr =
        png_create_read_struct(PNG_LIBPNG_VER_STRING, NULL, NULL, NULL);
    assert(png_ptr != NULL);

    png_time t = {2024, 2, 29, 12, 34, 56};
    char out[29];
    assert(png_convert_to_rfc1123_buffer(out, &t) == 1);
    assert(strcmp(out, "29 Feb 2024 12:34:56 +0000") == 0);
    assert(strcmp(png_convert_to_rfc1123(png_ptr, &t), out) == 0);

    struct tm tm_value;
    memset(&tm_value, 0, sizeof tm_value);
    tm_value.tm_year = 124;
    tm_value.tm_mon = 1;
    tm_value.tm_mday = 29;
    tm_value.tm_hour = 12;
    tm_value.tm_min = 34;
    tm_value.tm_sec = 56;

    png_time converted;
    memset(&converted, 0, sizeof converted);
    png_convert_from_struct_tm(&converted, &tm_value);
    assert(converted.year == 2024);
    assert(converted.month == 2);
    assert(converted.day == 29);
    assert(converted.hour == 12);
    assert(converted.minute == 34);
    assert(converted.second == 56);

    time_t epoch = 0;
    png_convert_from_time_t(&converted, epoch);
    assert(converted.year == 1970);
    assert(converted.month == 1);
    assert(converted.day == 1);
    assert(converted.hour == 0);
    assert(converted.minute == 0);
    assert(converted.second == 0);

    unsigned char bytes[4] = {0x12, 0x34, 0x56, 0x78};
    assert(png_get_uint_32(bytes) == 0x12345678U);
    assert(png_get_uint_16(bytes) == 0x1234U);
    unsigned char neg_two[4] = {0xff, 0xff, 0xff, 0xfe};
    assert(png_get_int_32(neg_two) == -2);

    unsigned char saved[4] = {0, 0, 0, 0};
    png_save_uint_32(saved, 0x89abcdefU);
    assert(saved[0] == 0x89 && saved[1] == 0xab && saved[2] == 0xcd &&
           saved[3] == 0xef);
    png_save_uint_16(saved, 0x1357U);
    assert(saved[0] == 0x13 && saved[1] == 0x57);

    assert(png_access_version_number() == 10643U);
    assert(strcmp(png_get_libpng_ver(NULL), "1.6.43") == 0);
    assert(strstr(png_get_header_version(NULL), "libpng version 1.6.43") != NULL);
    assert(strstr(png_get_copyright(NULL), "Cosmin Truta") != NULL);

    unsigned char signature[8] = {137, 80, 78, 71, 13, 10, 26, 10};
    assert(png_sig_cmp(signature, 0, sizeof signature) == 0);

    png_destroy_read_struct(&png_ptr, NULL, NULL);
    return 0;
}
