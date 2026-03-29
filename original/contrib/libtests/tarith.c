/* tarith.c
 *
 * Copyright (c) 2021 Cosmin Truta
 * Copyright (c) 2011-2013 John Cunningham Bowler
 *
 * This code is released under the libpng license.
 * For conditions of distribution and use, see the disclaimer
 * and license in png.h
 *
 * Public arithmetic-related regression tests for libpng.
 *
 * Historically this test reached into private libpng implementation details by
 * including pngpriv.h and png.c directly, then validating internal helpers
 * such as png_ascii_from_fp, png_check_fp_number, png_muldiv, and the internal
 * gamma correction functions.  That approach is not suitable for consumers of
 * the public API, so this program now exercises the closest exported entry
 * points instead:
 *
 *   - png_set_sCAL[_fixed/_s] / png_get_sCAL[_fixed/_s]
 *   - png_set_pHYs / png_get_*pixels_per_* / png_get_pixel_aspect_ratio[_fixed]
 *   - png_set_oFFs / png_get_*offset_inches[_fixed]
 *   - png_set_gamma[_fixed] through an in-memory read/write round trip
 *
 * Coverage necessarily decreases relative to the original private-function
 * harness, but the tests below retain public coverage of the formatting,
 * parsing, fixed-point arithmetic, and gamma-conversion behavior that callers
 * can rely on.
 */
#define _POSIX_SOURCE 1
#define _ISOC99_SOURCE 1

#include <math.h>
#include <setjmp.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef PNG_FREESTANDING_TESTS
#  include <png.h>
#else
#  include "../../png.h"
#endif

typedef struct test_context
{
   int warning_count;
   char message[256];
}
test_context;

typedef struct memory_buffer
{
   png_bytep data;
   size_t size;
   size_t capacity;
   size_t offset;
}
memory_buffer;

static int failures = 0;

static void
record_message(test_context *context, png_const_charp message)
{
   if (context == NULL)
      return;

   if (message == NULL)
      message = "(no message)";

   strncpy(context->message, message, sizeof context->message - 1);
   context->message[sizeof context->message - 1] = 0;
}

static void PNGCBAPI
test_error(png_structp png_ptr, png_const_charp message)
{
   test_context *context = (test_context*)png_get_error_ptr(png_ptr);
   record_message(context, message);
   png_longjmp(png_ptr, 1);
}

static void PNGCBAPI
test_warning(png_structp png_ptr, png_const_charp message)
{
   test_context *context = (test_context*)png_get_error_ptr(png_ptr);
   record_message(context, message);

   if (context != NULL)
      ++context->warning_count;
}

static int
check_true(int condition, const char *label)
{
   if (!condition)
   {
      fprintf(stderr, "FAIL: %s\n", label);
      ++failures;
   }

   return condition;
}

static int
check_close_double(double actual, double expected, double tolerance,
   const char *label)
{
   if (fabs(actual - expected) > tolerance)
   {
      fprintf(stderr, "FAIL: %s (got %.17g expected %.17g tolerance %.17g)\n",
         label, actual, expected, tolerance);
      ++failures;
      return 0;
   }

   return 1;
}

static int
check_close_long(long actual, long expected, long tolerance, const char *label)
{
   long delta = actual - expected;

   if (delta < 0)
      delta = -delta;

   if (delta > tolerance)
   {
      fprintf(stderr, "FAIL: %s (got %ld expected %ld tolerance %ld)\n",
         label, actual, expected, tolerance);
      ++failures;
      return 0;
   }

   return 1;
}

static png_structp
make_write_struct(test_context *context)
{
   return png_create_write_struct(PNG_LIBPNG_VER_STRING, context, test_error,
      test_warning);
}

static png_structp
make_read_struct(test_context *context)
{
   return png_create_read_struct(PNG_LIBPNG_VER_STRING, context, test_error,
      test_warning);
}

static png_uint_32
round_div_u64(unsigned long long numerator, unsigned long long denominator)
{
   return (png_uint_32)((numerator + denominator / 2) / denominator);
}

static png_uint_32
expected_ppi(png_uint_32 ppm)
{
   return round_div_u64((unsigned long long)ppm * 127U, 5000U);
}

static png_fixed_point
expected_ratio_fixed(png_uint_32 x_ppu, png_uint_32 y_ppu)
{
   return (png_fixed_point)round_div_u64(
      (unsigned long long)y_ppu * PNG_FP_1, x_ppu);
}

static png_fixed_point
expected_inches_fixed(png_int_32 microns)
{
   return (png_fixed_point)round_div_u64(
      (unsigned long long)microns * 500U, 127U);
}

static unsigned int
expected_gamma_sample(unsigned int input, unsigned int max_value,
   double file_gamma, double screen_gamma)
{
   double encoded = 0;
   double corrected;
   double scaled;

   if (max_value > 0)
      encoded = (double)input / (double)max_value;

   corrected = pow(encoded, 1.0 / (file_gamma * screen_gamma));
   scaled = corrected * (double)max_value;

   if (scaled < 0)
      scaled = 0;
   else if (scaled > max_value)
      scaled = (double)max_value;

   return (unsigned int)floor(scaled + .5);
}

static void PNGCBAPI
write_memory(png_structp png_ptr, png_bytep data, png_size_t length)
{
   memory_buffer *buffer = (memory_buffer*)png_get_io_ptr(png_ptr);
   size_t required;

   if (buffer == NULL)
      png_error(png_ptr, "missing write buffer");

   required = buffer->size + length;

   if (required > buffer->capacity)
   {
      size_t new_capacity = buffer->capacity;
      png_bytep new_data;

      if (new_capacity == 0)
         new_capacity = 128;

      while (new_capacity < required)
         new_capacity *= 2;

      new_data = (png_bytep)realloc(buffer->data, new_capacity);

      if (new_data == NULL)
         png_error(png_ptr, "out of memory growing write buffer");

      buffer->data = new_data;
      buffer->capacity = new_capacity;
   }

   memcpy(buffer->data + buffer->size, data, length);
   buffer->size += length;
}

static void PNGCBAPI
flush_memory(png_structp png_ptr)
{
   (void)png_ptr;
}

static void PNGCBAPI
read_memory(png_structp png_ptr, png_bytep data, png_size_t length)
{
   memory_buffer *buffer = (memory_buffer*)png_get_io_ptr(png_ptr);

   if (buffer == NULL)
      png_error(png_ptr, "missing read buffer");

   if (buffer->offset + length > buffer->size)
      png_error(png_ptr, "read past end of input buffer");

   memcpy(data, buffer->data + buffer->offset, length);
   buffer->offset += length;
}

static int
write_gray_png(memory_buffer *buffer, int bit_depth, unsigned int sample,
   png_fixed_point file_gamma)
{
   test_context context;
   png_structp png_ptr = NULL;
   png_infop info_ptr = NULL;
   int ok = 1;

   memset(&context, 0, sizeof context);
   buffer->size = 0;
   buffer->offset = 0;

   png_ptr = make_write_struct(&context);
   if (png_ptr != NULL)
      info_ptr = png_create_info_struct(png_ptr);

   if (!check_true(png_ptr != NULL, "create write struct") ||
       !check_true(info_ptr != NULL, "create write info"))
      ok = 0;

   if (ok && setjmp(png_jmpbuf(png_ptr)))
   {
      fprintf(stderr, "FAIL: write_gray_png (%s)\n", context.message);
      ++failures;
      ok = 0;
   }

   if (ok)
   {
      png_set_write_fn(png_ptr, buffer, write_memory, flush_memory);
      png_set_IHDR(png_ptr, info_ptr, 1, 1, bit_depth, PNG_COLOR_TYPE_GRAY,
         PNG_INTERLACE_NONE, PNG_COMPRESSION_TYPE_BASE, PNG_FILTER_TYPE_BASE);

#ifdef PNG_FIXED_POINT_SUPPORTED
      png_set_gAMA_fixed(png_ptr, info_ptr, file_gamma);
#elif defined(PNG_FLOATING_POINT_SUPPORTED)
      png_set_gAMA(png_ptr, info_ptr, file_gamma / (double)PNG_FP_1);
#endif

      png_write_info(png_ptr, info_ptr);

      if (bit_depth == 8)
      {
         png_byte row[1];
         row[0] = (png_byte)sample;
         png_write_row(png_ptr, row);
      }

#ifdef PNG_WRITE_16BIT_SUPPORTED
      else if (bit_depth == 16)
      {
         png_byte row[2];
         png_save_uint_16(row, sample);
         png_write_row(png_ptr, row);
      }
#endif

      else
      {
         fprintf(stderr, "FAIL: unsupported bit depth %d\n", bit_depth);
         ++failures;
         ok = 0;
      }

      if (ok)
         png_write_end(png_ptr, info_ptr);
   }

   png_destroy_write_struct(&png_ptr, &info_ptr);
   return ok;
}

static int
read_gray_png(memory_buffer *buffer, int bit_depth, png_fixed_point screen_gamma,
   int use_fixed, unsigned int *sample_out)
{
   test_context context;
   png_structp png_ptr = NULL;
   png_infop info_ptr = NULL;
   int ok = 1;

   memset(&context, 0, sizeof context);
   buffer->offset = 0;

   png_ptr = make_read_struct(&context);
   if (png_ptr != NULL)
      info_ptr = png_create_info_struct(png_ptr);

   if (!check_true(png_ptr != NULL, "create read struct") ||
       !check_true(info_ptr != NULL, "create read info"))
      ok = 0;

   if (ok && setjmp(png_jmpbuf(png_ptr)))
   {
      fprintf(stderr, "FAIL: read_gray_png (%s)\n", context.message);
      ++failures;
      ok = 0;
   }

   if (ok)
   {
      png_set_read_fn(png_ptr, buffer, read_memory);
      png_read_info(png_ptr, info_ptr);

#ifdef PNG_FIXED_POINT_SUPPORTED
      if (use_fixed)
         png_set_gamma_fixed(png_ptr, screen_gamma, PNG_FP_1);
      else
#endif
#ifdef PNG_FLOATING_POINT_SUPPORTED
         png_set_gamma(png_ptr, screen_gamma / (double)PNG_FP_1, 1.0);
#else
      (void)screen_gamma;
#endif

      png_read_update_info(png_ptr, info_ptr);

      ok &= check_true(png_get_color_type(png_ptr, info_ptr) ==
         PNG_COLOR_TYPE_GRAY, "gamma read keeps grayscale");
      ok &= check_true(png_get_bit_depth(png_ptr, info_ptr) == bit_depth,
         "gamma read keeps bit depth");

      if (ok && bit_depth == 8)
      {
         png_byte row[1];
         png_read_row(png_ptr, row, NULL);
         *sample_out = row[0];
      }

#ifdef PNG_READ_16BIT_SUPPORTED
      else if (ok && bit_depth == 16)
      {
         png_byte row[2];
         png_read_row(png_ptr, row, NULL);
         *sample_out = png_get_uint_16(row);
      }
#endif

      if (ok)
         png_read_end(png_ptr, NULL);
   }

   png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
   return ok;
}

static int
test_scal_string_roundtrip(void)
{
   test_context context;
   png_structp png_ptr;
   png_infop info_ptr;
   int ok = 1;
   int unit = 0;
   png_charp width = NULL;
   png_charp height = NULL;

   memset(&context, 0, sizeof context);
   png_ptr = make_write_struct(&context);
   if (png_ptr != NULL)
      info_ptr = png_create_info_struct(png_ptr);

   if (!check_true(png_ptr != NULL, "sCAL string create write struct") ||
       !check_true(info_ptr != NULL, "sCAL string create info"))
      ok = 0;

   if (ok && setjmp(png_jmpbuf(png_ptr)))
   {
      fprintf(stderr, "FAIL: sCAL string roundtrip (%s)\n", context.message);
      ++failures;
      ok = 0;
   }

   if (ok)
   {
      png_set_sCAL_s(png_ptr, info_ptr, PNG_SCALE_METER, "123.456",
         "789.0123");
      ok &= check_true(png_get_sCAL_s(png_ptr, info_ptr, &unit, &width,
         &height) == PNG_INFO_sCAL, "png_get_sCAL_s returns valid bit");
      ok &= check_true(unit == PNG_SCALE_METER, "png_get_sCAL_s unit");
      ok &= check_true(strcmp(width, "123.456") == 0,
         "png_get_sCAL_s width");
      ok &= check_true(strcmp(height, "789.0123") == 0,
         "png_get_sCAL_s height");

#ifdef PNG_FLOATING_POINT_SUPPORTED
      if (ok)
      {
         double dw = 0;
         double dh = 0;
         ok &= check_true(png_get_sCAL(png_ptr, info_ptr, &unit, &dw, &dh) ==
            PNG_INFO_sCAL, "png_get_sCAL returns valid bit");
         ok &= check_close_double(dw, 123.456, 1e-12, "png_get_sCAL width");
         ok &= check_close_double(dh, 789.0123, 1e-12,
            "png_get_sCAL height");
      }
#endif

#ifdef PNG_FIXED_POINT_SUPPORTED
      if (ok)
      {
         png_fixed_point fw = 0;
         png_fixed_point fh = 0;
         ok &= check_true(png_get_sCAL_fixed(png_ptr, info_ptr, &unit, &fw,
            &fh) == PNG_INFO_sCAL, "png_get_sCAL_fixed returns valid bit");
         ok &= check_close_long(fw, 12345600L, 1L,
            "png_get_sCAL_fixed width");
         ok &= check_close_long(fh, 78901230L, 1L,
            "png_get_sCAL_fixed height");
      }
#endif
   }

   png_destroy_write_struct(&png_ptr, &info_ptr);
   return ok;
}

#ifdef PNG_FLOATING_POINT_SUPPORTED
static int
test_scal_float_roundtrip(void)
{
   test_context context;
   png_structp png_ptr;
   png_infop info_ptr;
   int ok = 1;
   int unit = 0;
   png_charp width = NULL;
   png_charp height = NULL;
   double dw = 0;
   double dh = 0;

   memset(&context, 0, sizeof context);
   png_ptr = make_write_struct(&context);
   if (png_ptr != NULL)
      info_ptr = png_create_info_struct(png_ptr);

   if (!check_true(png_ptr != NULL, "sCAL float create write struct") ||
       !check_true(info_ptr != NULL, "sCAL float create info"))
      ok = 0;

   if (ok && setjmp(png_jmpbuf(png_ptr)))
   {
      fprintf(stderr, "FAIL: sCAL float roundtrip (%s)\n", context.message);
      ++failures;
      ok = 0;
   }

   if (ok)
   {
      png_set_sCAL(png_ptr, info_ptr, PNG_SCALE_RADIAN, 1.25, 2.5);
      ok &= check_true(context.warning_count == 0,
         "png_set_sCAL float emits no warning");
      ok &= check_true(png_get_sCAL(png_ptr, info_ptr, &unit, &dw, &dh) ==
         PNG_INFO_sCAL, "png_get_sCAL(float) returns valid bit");
      ok &= check_true(unit == PNG_SCALE_RADIAN, "png_set_sCAL stores unit");
      ok &= check_close_double(dw, 1.25, 1e-12, "png_set_sCAL width");
      ok &= check_close_double(dh, 2.5, 1e-12, "png_set_sCAL height");
      ok &= check_true(png_get_sCAL_s(png_ptr, info_ptr, &unit, &width,
         &height) == PNG_INFO_sCAL, "png_set_sCAL stores strings");
      ok &= check_true(width != NULL && *width != 0,
         "png_set_sCAL width string non-empty");
      ok &= check_true(height != NULL && *height != 0,
         "png_set_sCAL height string non-empty");
   }

   png_destroy_write_struct(&png_ptr, &info_ptr);
   return ok;
}
#endif

#ifdef PNG_FIXED_POINT_SUPPORTED
static int
test_scal_fixed_roundtrip(void)
{
   test_context context;
   png_structp png_ptr;
   png_infop info_ptr;
   int ok = 1;
   int unit = 0;
   png_fixed_point width = 0;
   png_fixed_point height = 0;

   memset(&context, 0, sizeof context);
   png_ptr = make_write_struct(&context);
   if (png_ptr != NULL)
      info_ptr = png_create_info_struct(png_ptr);

   if (!check_true(png_ptr != NULL, "sCAL fixed create write struct") ||
       !check_true(info_ptr != NULL, "sCAL fixed create info"))
      ok = 0;

   if (ok && setjmp(png_jmpbuf(png_ptr)))
   {
      fprintf(stderr, "FAIL: sCAL fixed roundtrip (%s)\n", context.message);
      ++failures;
      ok = 0;
   }

   if (ok)
   {
      png_set_sCAL_fixed(png_ptr, info_ptr, PNG_SCALE_METER, 125000, 250000);
      ok &= check_true(context.warning_count == 0,
         "png_set_sCAL_fixed emits no warning");
      ok &= check_true(png_get_sCAL_fixed(png_ptr, info_ptr, &unit, &width,
         &height) == PNG_INFO_sCAL,
         "png_get_sCAL_fixed returns valid bit");
      ok &= check_true(unit == PNG_SCALE_METER,
         "png_set_sCAL_fixed stores unit");
      ok &= check_true(width == 125000, "png_set_sCAL_fixed width");
      ok &= check_true(height == 250000, "png_set_sCAL_fixed height");
   }

   png_destroy_write_struct(&png_ptr, &info_ptr);
   return ok;
}
#endif

static int
expect_scal_string_error(const char *label, png_const_charp width,
   png_const_charp height)
{
   test_context context;
   png_structp png_ptr;
   png_infop info_ptr;
   int ok = 1;
   int saw_error = 0;

   memset(&context, 0, sizeof context);
   png_ptr = make_write_struct(&context);
   if (png_ptr != NULL)
      info_ptr = png_create_info_struct(png_ptr);

   if (!check_true(png_ptr != NULL, "invalid sCAL create write struct") ||
       !check_true(info_ptr != NULL, "invalid sCAL create info"))
      ok = 0;

   if (ok && setjmp(png_jmpbuf(png_ptr)))
      saw_error = 1;

   else if (ok)
      png_set_sCAL_s(png_ptr, info_ptr, PNG_SCALE_METER, width, height);

   ok &= check_true(saw_error, label);
   png_destroy_write_struct(&png_ptr, &info_ptr);
   return ok;
}

static int
test_scal_invalid_inputs(void)
{
   int ok = 1;

   ok &= expect_scal_string_error("reject empty sCAL width", "", "1");
   ok &= expect_scal_string_error("reject negative sCAL width", "-1", "1");
   ok &= expect_scal_string_error("reject malformed sCAL width", "1e+", "1");
   ok &= expect_scal_string_error("reject malformed sCAL height", "1",
      "1.2.3");

#ifdef PNG_FLOATING_POINT_SUPPORTED
   {
      test_context context;
      png_structp png_ptr;
      png_infop info_ptr;
      int unit = 0;
      png_charp width = NULL;
      png_charp height = NULL;

      memset(&context, 0, sizeof context);
      png_ptr = make_write_struct(&context);
      if (png_ptr != NULL)
         info_ptr = png_create_info_struct(png_ptr);

      if (!check_true(png_ptr != NULL,
            "png_set_sCAL invalid width create struct") ||
          !check_true(info_ptr != NULL,
            "png_set_sCAL invalid width create info"))
         ok = 0;

      if (ok && setjmp(png_jmpbuf(png_ptr)))
      {
         fprintf(stderr, "FAIL: png_set_sCAL invalid width (%s)\n",
            context.message);
         ++failures;
         ok = 0;
      }

      if (ok)
      {
         png_set_sCAL(png_ptr, info_ptr, PNG_SCALE_METER, 0.0, 1.0);
         ok &= check_true(context.warning_count == 1,
            "png_set_sCAL warns on zero width");
         ok &= check_true(png_get_sCAL_s(png_ptr, info_ptr, &unit, &width,
            &height) == 0, "png_set_sCAL ignores zero width");
      }

      png_destroy_write_struct(&png_ptr, &info_ptr);
   }
#endif

   return ok;
}

static int
test_phys_dpi_conversion(void)
{
   test_context context;
   png_structp png_ptr;
   png_infop info_ptr;
   int ok = 1;
   const png_uint_32 ppm = 3779;

   memset(&context, 0, sizeof context);
   png_ptr = make_write_struct(&context);
   if (png_ptr != NULL)
      info_ptr = png_create_info_struct(png_ptr);

   if (!check_true(png_ptr != NULL, "pHYs create write struct") ||
       !check_true(info_ptr != NULL, "pHYs create info"))
      ok = 0;

   if (ok && setjmp(png_jmpbuf(png_ptr)))
   {
      fprintf(stderr, "FAIL: pHYs conversion (%s)\n", context.message);
      ++failures;
      ok = 0;
   }

   if (ok)
   {
      png_set_pHYs(png_ptr, info_ptr, ppm, ppm, PNG_RESOLUTION_METER);
      ok &= check_true(png_get_pixels_per_meter(png_ptr, info_ptr) == ppm,
         "png_get_pixels_per_meter");
      ok &= check_true(png_get_x_pixels_per_meter(png_ptr, info_ptr) == ppm,
         "png_get_x_pixels_per_meter");
      ok &= check_true(png_get_y_pixels_per_meter(png_ptr, info_ptr) == ppm,
         "png_get_y_pixels_per_meter");
      ok &= check_true(png_get_pixels_per_inch(png_ptr, info_ptr) ==
         expected_ppi(ppm), "png_get_pixels_per_inch");
      ok &= check_true(png_get_x_pixels_per_inch(png_ptr, info_ptr) ==
         expected_ppi(ppm), "png_get_x_pixels_per_inch");
      ok &= check_true(png_get_y_pixels_per_inch(png_ptr, info_ptr) ==
         expected_ppi(ppm), "png_get_y_pixels_per_inch");
   }

   png_destroy_write_struct(&png_ptr, &info_ptr);
   return ok;
}

static int
test_phys_aspect_ratio(void)
{
   test_context context;
   png_structp png_ptr;
   png_infop info_ptr;
   int ok = 1;
   const png_uint_32 x_ppu = 3000;
   const png_uint_32 y_ppu = 1000;

   memset(&context, 0, sizeof context);
   png_ptr = make_write_struct(&context);
   if (png_ptr != NULL)
      info_ptr = png_create_info_struct(png_ptr);

   if (!check_true(png_ptr != NULL, "aspect ratio create write struct") ||
       !check_true(info_ptr != NULL, "aspect ratio create info"))
      ok = 0;

   if (ok && setjmp(png_jmpbuf(png_ptr)))
   {
      fprintf(stderr, "FAIL: aspect ratio (%s)\n", context.message);
      ++failures;
      ok = 0;
   }

   if (ok)
   {
      png_set_pHYs(png_ptr, info_ptr, x_ppu, y_ppu, PNG_RESOLUTION_METER);
      ok &= check_true(png_get_pixels_per_meter(png_ptr, info_ptr) == 0,
         "png_get_pixels_per_meter requires equal axes");

#ifdef PNG_FLOATING_POINT_SUPPORTED
      ok &= check_close_double(png_get_pixel_aspect_ratio(png_ptr, info_ptr),
         (double)y_ppu / (double)x_ppu, 1e-6,
         "png_get_pixel_aspect_ratio");
#endif

#ifdef PNG_FIXED_POINT_SUPPORTED
      ok &= check_true(png_get_pixel_aspect_ratio_fixed(png_ptr, info_ptr) ==
         expected_ratio_fixed(x_ppu, y_ppu),
         "png_get_pixel_aspect_ratio_fixed");
#endif
   }

   png_destroy_write_struct(&png_ptr, &info_ptr);
   return ok;
}

static int
test_offset_inches_conversion(void)
{
   test_context context;
   png_structp png_ptr;
   png_infop info_ptr;
   int ok = 1;
   const png_int_32 x_microns = 25400;
   const png_int_32 y_microns = 12700;

   memset(&context, 0, sizeof context);
   png_ptr = make_write_struct(&context);
   if (png_ptr != NULL)
      info_ptr = png_create_info_struct(png_ptr);

   if (!check_true(png_ptr != NULL, "oFFs create write struct") ||
       !check_true(info_ptr != NULL, "oFFs create info"))
      ok = 0;

   if (ok && setjmp(png_jmpbuf(png_ptr)))
   {
      fprintf(stderr, "FAIL: oFFs conversion (%s)\n", context.message);
      ++failures;
      ok = 0;
   }

   if (ok)
   {
      png_set_oFFs(png_ptr, info_ptr, x_microns, y_microns,
         PNG_OFFSET_MICROMETER);

#ifdef PNG_FLOATING_POINT_SUPPORTED
      ok &= check_close_double(png_get_x_offset_inches(png_ptr, info_ptr), 1.0,
         5e-6, "png_get_x_offset_inches");
      ok &= check_close_double(png_get_y_offset_inches(png_ptr, info_ptr), 0.5,
         5e-6, "png_get_y_offset_inches");
#endif

#ifdef PNG_FIXED_POINT_SUPPORTED
      ok &= check_true(png_get_x_offset_inches_fixed(png_ptr, info_ptr) ==
         expected_inches_fixed(x_microns), "png_get_x_offset_inches_fixed");
      ok &= check_true(png_get_y_offset_inches_fixed(png_ptr, info_ptr) ==
         expected_inches_fixed(y_microns), "png_get_y_offset_inches_fixed");
#endif
   }

   png_destroy_write_struct(&png_ptr, &info_ptr);
   return ok;
}

static int
test_gamma_transform_common(int bit_depth, int use_fixed)
{
   memory_buffer buffer;
   int ok = 1;
   unsigned int output = 0;
   const png_fixed_point file_gamma = PNG_FP_1;
   const png_fixed_point screen_gamma = 220000;
   const unsigned int input = bit_depth == 8 ? 128U : 32768U;
   const unsigned int max_value = bit_depth == 8 ? 255U : 65535U;
   const unsigned int expected = expected_gamma_sample(input, max_value,
      file_gamma / (double)PNG_FP_1, screen_gamma / (double)PNG_FP_1);

   memset(&buffer, 0, sizeof buffer);

   ok &= write_gray_png(&buffer, bit_depth, input, file_gamma);
   ok &= read_gray_png(&buffer, bit_depth, screen_gamma, use_fixed, &output);

   if (ok)
   {
      const long tolerance = bit_depth == 8 ? 1L : 2L;
      ok &= check_close_long((long)output, (long)expected, tolerance,
         use_fixed ? "gamma transform fixed" : "gamma transform float");
   }

   free(buffer.data);
   return ok;
}

static int
run_ascii_tests(void)
{
   int start_failures = failures;

   printf("tarith: exercising public sCAL APIs\n");
   test_scal_string_roundtrip();
#ifdef PNG_FLOATING_POINT_SUPPORTED
   test_scal_float_roundtrip();
#endif
#ifdef PNG_FIXED_POINT_SUPPORTED
   test_scal_fixed_roundtrip();
#endif

   return failures == start_failures;
}

static int
run_checkfp_tests(void)
{
   int start_failures = failures;

   printf("tarith: validating sCAL parser rejection through public APIs\n");
   test_scal_invalid_inputs();
   return failures == start_failures;
}

static int
run_muldiv_tests(void)
{
   int start_failures = failures;

   printf("tarith: exercising public fixed-point conversion APIs\n");
   test_phys_dpi_conversion();
   test_phys_aspect_ratio();
   test_offset_inches_conversion();
   return failures == start_failures;
}

static int
run_gamma_tests(void)
{
   int start_failures = failures;

   printf("tarith: exercising public gamma conversion APIs\n");
#if defined(PNG_FLOATING_POINT_SUPPORTED)
   test_gamma_transform_common(8, 0);
#  if defined(PNG_READ_16BIT_SUPPORTED) && defined(PNG_WRITE_16BIT_SUPPORTED)
   test_gamma_transform_common(16, 0);
#  endif
#endif

#if defined(PNG_FIXED_POINT_SUPPORTED)
   test_gamma_transform_common(8, 1);
#  if defined(PNG_READ_16BIT_SUPPORTED) && defined(PNG_WRITE_16BIT_SUPPORTED)
   test_gamma_transform_common(16, 1);
#  endif
#endif

#if !defined(PNG_FLOATING_POINT_SUPPORTED) && !defined(PNG_FIXED_POINT_SUPPORTED)
   printf("tarith: gamma tests skipped; no public gamma API enabled\n");
#endif

   return failures == start_failures;
}

static int
run_all_tests(void)
{
   run_ascii_tests();
   run_checkfp_tests();
   run_muldiv_tests();
   run_gamma_tests();
   return failures == 0;
}

int
main(int argc, char **argv)
{
   while (argc > 1)
   {
      if (strcmp(argv[1], "-v") == 0)
      {
         --argc;
         ++argv;
      }

      else if (argc > 2 && strcmp(argv[1], "-c") == 0)
      {
         argc -= 2;
         argv += 2;
      }

      else
         break;
   }

   if (argc == 1 || strcmp(argv[1], "all") == 0)
      run_all_tests();

   else if (strcmp(argv[1], "ascii") == 0)
      run_ascii_tests();

   else if (strcmp(argv[1], "checkfp") == 0)
      run_checkfp_tests();

   else if (strcmp(argv[1], "muldiv") == 0)
      run_muldiv_tests();

   else if (strcmp(argv[1], "gamma") == 0)
      run_gamma_tests();

   else
   {
      fprintf(stderr,
         "usage: tarith [-v] [-c ignored] [all|ascii|checkfp|muldiv|gamma]\n");
      return 1;
   }

   if (failures == 0)
   {
      printf("tarith: PASS\n");
      return 0;
   }

   fprintf(stderr, "tarith: FAIL (%d failures)\n", failures);
   return 1;
}
