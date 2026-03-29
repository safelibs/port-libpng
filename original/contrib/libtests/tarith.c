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
 * Historically this test included pngpriv.h and png.c directly and validated
 * internal helpers such as png_ascii_from_fp, png_check_fp_number,
 * png_muldiv, and the internal gamma correction routines.  This rewrite keeps
 * the focus on the same behaviors but reaches them only through exported APIs:
 *
 *   - png_set_sCAL[_fixed/_s] / png_get_sCAL[_fixed/_s]
 *   - png_set_pHYs / png_get_*pixels_per_* / png_get_pixel_aspect_ratio_fixed
 *   - png_set_oFFs / png_get_*offset_inches[_fixed]
 *   - png_set_gamma[_fixed] through in-memory read/write round-trips
 *
 * Some internal-only coverage is unavoidably lost because the public API does
 * not expose parser state or the private lookup-table helpers directly.
 * However, this program keeps broad public coverage by exercising many random
 * and exhaustive cases rather than a small set of spot checks.
 */
#define _POSIX_SOURCE 1
#define _ISOC99_SOURCE 1

#include <ctype.h>
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

typedef struct scal_acceptor
{
   test_context context;
   png_structp png_ptr;
   png_infop info_ptr;
}
scal_acceptor;

static int failures = 0;
static int test_iterations = 5000;
static int checkfp_max_length = 4;
static png_uint_32 random_state = 0x504E4721U;

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
next_random_u32(void)
{
   random_state = random_state * 1103515245U + 12345U;
   return random_state;
}

static png_uint_32
random_u31_nonzero(void)
{
   png_uint_32 value = next_random_u32() & PNG_UINT_31_MAX;

   if (value == 0)
      value = 1;

   return value;
}

static png_int_32
random_i31(void)
{
   png_int_32 value = (png_int_32)(next_random_u32() & PNG_UINT_31_MAX);

   if ((next_random_u32() & 1U) != 0)
      value = -value;

   return value;
}

static double
random_unit_double(void)
{
   return (next_random_u32() + 0.5) / 4294967296.0;
}

static double
random_positive_double(void)
{
   int exponent = (int)(next_random_u32() % 100) - 50;
   double mantissa = 1.0 + random_unit_double() * 9.0;
   return ldexp(mantissa, exponent);
}

static png_uint_32
round_div_u64(unsigned long long numerator, unsigned long long denominator)
{
   return (png_uint_32)((numerator + denominator / 2) / denominator);
}

static long long
round_div_s64(long long numerator, long long denominator)
{
   if ((numerator < 0) != (denominator < 0))
      numerator -= denominator / 2;
   else
      numerator += denominator / 2;

   return numerator / denominator;
}

static png_uint_32
expected_ppi(png_uint_32 ppm)
{
   return round_div_u64((unsigned long long)ppm * 127U, 5000U);
}

static png_fixed_point
expected_ratio_fixed(png_uint_32 x_ppu, png_uint_32 y_ppu, int *ok)
{
   unsigned long long numerator =
      (unsigned long long)y_ppu * (unsigned long long)PNG_FP_1;
   png_uint_32 result;

   result = round_div_u64(numerator, x_ppu);

   if (result > PNG_UINT_31_MAX)
   {
      *ok = 0;
      return 0;
   }

   *ok = 1;
   return (png_fixed_point)result;
}

static png_fixed_point
expected_inches_fixed(png_int_32 microns, int *ok)
{
   long long result = round_div_s64((long long)microns * 500, 127);

   if (result < -1 - (long long)PNG_UINT_31_MAX ||
       result > PNG_UINT_31_MAX)
   {
      *ok = 0;
      return 0;
   }

   *ok = 1;
   return (png_fixed_point)result;
}

static void
store_u16(png_bytep buffer, unsigned int value)
{
   buffer[0] = (png_byte)((value >> 8) & 0xffU);
   buffer[1] = (png_byte)(value & 0xffU);
}

static unsigned int
load_u16(png_const_bytep buffer)
{
   return ((unsigned int)buffer[0] << 8) + buffer[1];
}

static double
scal_relative_tolerance(void)
{
   return .5 / pow(10.0, PNG_sCAL_PRECISION - 1);
}

static int
check_scal_roundtrip(double actual, double expected, const char *label)
{
   double change;
   double tolerance;

   if (expected == 0)
      return check_close_double(actual, expected, 0, label);

   change = fabs((actual - expected) / expected);
   tolerance = scal_relative_tolerance();

   if (change > tolerance)
   {
      fprintf(stderr,
         "FAIL: %s (got %.17g expected %.17g relative error %.17g tolerance %.17g)\n",
         label, actual, expected, change, tolerance);
      ++failures;
      return 0;
   }

   return 1;
}

static int
parse_png_number(png_const_charp string, int *negative, int *zero)
{
   const unsigned char *p = (const unsigned char*)string;
   int saw_integer = 0;
   int saw_fraction = 0;

   *negative = 0;
   *zero = 1;

   if (*p == '+' || *p == '-')
   {
      if (*p == '-')
         *negative = 1;
      ++p;
   }

   while (isdigit(*p))
   {
      saw_integer = 1;
      if (*p != '0')
         *zero = 0;
      ++p;
   }

   if (*p == '.')
   {
      ++p;

      while (isdigit(*p))
      {
         saw_fraction = 1;
         if (*p != '0')
            *zero = 0;
         ++p;
      }
   }

   if (!saw_integer && !saw_fraction)
      return 0;

   if (*p == 'e' || *p == 'E')
   {
      ++p;

      if (*p == '+' || *p == '-')
         ++p;

      if (!isdigit(*p))
         return 0;

      do
         ++p;
      while (isdigit(*p));
   }

   return *p == 0;
}

static void
describe_string(png_const_charp string, char *buffer, size_t size)
{
   size_t out = 0;

   while (*string != 0 && out + 4 < size)
   {
      unsigned int ch = (unsigned char)*string++;

      if (isprint(ch))
         buffer[out++] = (char)ch;
      else
      {
         static const char hex[] = "0123456789ABCDEF";
         buffer[out++] = '<';
         buffer[out++] = hex[(ch >> 4) & 0xf];
         buffer[out++] = hex[ch & 0xf];
         buffer[out++] = '>';
      }
   }

   buffer[out] = 0;
}

static int
scal_acceptor_init(scal_acceptor *acceptor)
{
   memset(acceptor, 0, sizeof *acceptor);
   acceptor->png_ptr = make_write_struct(&acceptor->context);

   if (acceptor->png_ptr != NULL)
      acceptor->info_ptr = png_create_info_struct(acceptor->png_ptr);

   return check_true(acceptor->png_ptr != NULL, "create sCAL acceptor struct")
      && check_true(acceptor->info_ptr != NULL, "create sCAL acceptor info");
}

static void
scal_acceptor_destroy(scal_acceptor *acceptor)
{
   if (acceptor->png_ptr != NULL)
      png_destroy_write_struct(&acceptor->png_ptr, &acceptor->info_ptr);
}

static int
scal_acceptor_accepts(scal_acceptor *acceptor, png_const_charp string)
{
   acceptor->context.warning_count = 0;
   acceptor->context.message[0] = 0;

   if (setjmp(png_jmpbuf(acceptor->png_ptr)))
   {
      scal_acceptor_destroy(acceptor);
      scal_acceptor_init(acceptor);
      return 0;
   }

   png_free_data(acceptor->png_ptr, acceptor->info_ptr, PNG_FREE_SCAL, -1);
   png_set_sCAL_s(acceptor->png_ptr, acceptor->info_ptr, PNG_SCALE_METER,
      string, "1");
   png_free_data(acceptor->png_ptr, acceptor->info_ptr, PNG_FREE_SCAL, -1);
   return 1;
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
         new_capacity = 1024;

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
write_gray_ramp_png(memory_buffer *buffer, int bit_depth, png_fixed_point gamma)
{
   test_context context;
   png_structp png_ptr = NULL;
   png_infop info_ptr = NULL;
   png_bytep row = NULL;
   png_uint_32 width = bit_depth == 8 ? 256U : 65536U;
   size_t rowbytes = bit_depth == 8 ? width : width * 2U;
   int ok = 1;
   png_uint_32 i;

   memset(&context, 0, sizeof context);
   buffer->size = 0;
   buffer->offset = 0;

   row = (png_bytep)malloc(rowbytes);
   ok &= check_true(row != NULL, "allocate ramp row");

   if (ok)
   {
      if (bit_depth == 8)
      {
         for (i=0; i<width; ++i)
            row[i] = (png_byte)i;
      }

      else
      {
         for (i=0; i<width; ++i)
            store_u16(row + (i << 1), i);
      }
   }

   png_ptr = make_write_struct(&context);

   if (png_ptr != NULL)
      info_ptr = png_create_info_struct(png_ptr);

   ok &= check_true(png_ptr != NULL, "create gamma write struct");
   ok &= check_true(info_ptr != NULL, "create gamma write info");

   if (ok && setjmp(png_jmpbuf(png_ptr)))
   {
      fprintf(stderr, "FAIL: write_gray_ramp_png (%s)\n", context.message);
      ++failures;
      ok = 0;
   }

   if (ok)
   {
      png_set_write_fn(png_ptr, buffer, write_memory, flush_memory);
      png_set_filter(png_ptr, PNG_FILTER_TYPE_BASE, PNG_FILTER_NONE);
      png_set_compression_level(png_ptr, 1);
      png_set_IHDR(png_ptr, info_ptr, width, 1, bit_depth, PNG_COLOR_TYPE_GRAY,
         PNG_INTERLACE_NONE, PNG_COMPRESSION_TYPE_BASE, PNG_FILTER_TYPE_BASE);

#ifdef PNG_FIXED_POINT_SUPPORTED
      png_set_gAMA_fixed(png_ptr, info_ptr, gamma);
#elif defined(PNG_FLOATING_POINT_SUPPORTED)
      png_set_gAMA(png_ptr, info_ptr, gamma / (double)PNG_FP_1);
#endif

      png_write_info(png_ptr, info_ptr);
      png_write_row(png_ptr, row);
      png_write_end(png_ptr, info_ptr);
   }

   png_destroy_write_struct(&png_ptr, &info_ptr);
   free(row);
   return ok;
}

static int
verify_gamma_ramp(memory_buffer *buffer, int bit_depth, png_fixed_point screen,
   int use_fixed, double exponent, const char *label)
{
   test_context context;
   png_structp png_ptr = NULL;
   png_infop info_ptr = NULL;
   png_bytep row = NULL;
   png_uint_32 width = bit_depth == 8 ? 256U : 65536U;
   size_t rowbytes = bit_depth == 8 ? width : width * 2U;
   unsigned int max_value = bit_depth == 8 ? 255U : 65535U;
   int ok = 1;
   png_uint_32 i;
   long worst = 0;

   memset(&context, 0, sizeof context);
   buffer->offset = 0;

   row = (png_bytep)malloc(rowbytes);
   ok &= check_true(row != NULL, "allocate gamma read row");

   png_ptr = make_read_struct(&context);

   if (png_ptr != NULL)
      info_ptr = png_create_info_struct(png_ptr);

   ok &= check_true(png_ptr != NULL, "create gamma read struct");
   ok &= check_true(info_ptr != NULL, "create gamma read info");

   if (ok && setjmp(png_jmpbuf(png_ptr)))
   {
      fprintf(stderr, "FAIL: %s (%s)\n", label, context.message);
      ++failures;
      ok = 0;
   }

   if (ok)
   {
      png_set_read_fn(png_ptr, buffer, read_memory);
      png_read_info(png_ptr, info_ptr);

#ifdef PNG_FIXED_POINT_SUPPORTED
      if (use_fixed)
         png_set_gamma_fixed(png_ptr, screen, PNG_FP_1);
      else
#endif
#ifdef PNG_FLOATING_POINT_SUPPORTED
         png_set_gamma(png_ptr, screen / (double)PNG_FP_1, 1.0);
#else
      (void)screen;
#endif

      png_read_update_info(png_ptr, info_ptr);
      ok &= check_true(png_get_color_type(png_ptr, info_ptr) ==
         PNG_COLOR_TYPE_GRAY, "gamma output color type");
      ok &= check_true(png_get_bit_depth(png_ptr, info_ptr) == bit_depth,
         "gamma output bit depth");
      ok &= check_true(png_get_rowbytes(png_ptr, info_ptr) == rowbytes,
         "gamma output rowbytes");

      if (ok)
      {
         png_read_row(png_ptr, row, NULL);

         for (i=0; i<width; ++i)
         {
            unsigned int actual = bit_depth == 8 ? row[i] : load_u16(row + (i << 1));
            unsigned int expected = (unsigned int)floor(
               pow(i / (double)max_value, exponent) * max_value + .5);
            long delta = (long)actual - (long)expected;
            long tolerance = bit_depth == 8 ? 1L : 2L;

            if (delta < 0)
               delta = -delta;

            if (delta > worst)
               worst = delta;

            if (delta > tolerance)
            {
               fprintf(stderr,
                  "FAIL: %s sample %lu got %u expected %u tolerance %ld\n",
                  label, (unsigned long)i, actual, expected, tolerance);
               ++failures;
               ok = 0;
               break;
            }
         }

         png_read_end(png_ptr, NULL);
      }
   }

   png_destroy_read_struct(&png_ptr, &info_ptr, NULL);
   free(row);
   return ok;
}

static int
validation_ascii_to_fp(int count)
{
   static const double edge_values[] =
   {
      0.0000100001, 0.000099999, 0.00123456, 0.0123456, 0.123456, 1.0,
      1.23456, 12.3456, 123.456, 12345.6, 99999.4, 99999.6
   };
   int i;
   double worst = 0;

   printf("tarith: validating sCAL formatting/parsing with %d public cases\n",
      count);

#ifdef PNG_FLOATING_POINT_SUPPORTED
   for (i=0; i<(int)(sizeof edge_values / sizeof edge_values[0]); ++i)
   {
      test_context context;
      png_structp png_ptr;
      png_infop info_ptr;
      int ok = 1;
      int unit = 0;
      double width = edge_values[i];
      double height = edge_values[(i+3) %
         (sizeof edge_values / sizeof edge_values[0])];
      double width_out = 0;
      double height_out = 0;
      png_charp width_s = NULL;
      png_charp height_s = NULL;

      memset(&context, 0, sizeof context);
      png_ptr = make_write_struct(&context);
      if (png_ptr != NULL)
         info_ptr = png_create_info_struct(png_ptr);
      else
         info_ptr = NULL;

      ok &= check_true(png_ptr != NULL, "ascii edge create write struct");
      ok &= check_true(info_ptr != NULL, "ascii edge create info");

      if (ok && setjmp(png_jmpbuf(png_ptr)))
      {
         fprintf(stderr, "FAIL: ascii edge case (%s)\n", context.message);
         ++failures;
         ok = 0;
      }

      if (ok)
      {
         png_set_sCAL(png_ptr, info_ptr, PNG_SCALE_METER, width, height);
         ok &= check_true(context.warning_count == 0,
            "png_set_sCAL edge emits no warning");
         ok &= check_true(png_get_sCAL(png_ptr, info_ptr, &unit, &width_out,
            &height_out) == PNG_INFO_sCAL, "png_get_sCAL edge valid bit");
         ok &= check_true(unit == PNG_SCALE_METER, "png_get_sCAL edge unit");
         ok &= check_scal_roundtrip(width_out, width,
            "png_set_sCAL edge width roundtrip");
         ok &= check_scal_roundtrip(height_out, height,
            "png_set_sCAL edge height roundtrip");
         ok &= check_true(png_get_sCAL_s(png_ptr, info_ptr, &unit, &width_s,
            &height_s) == PNG_INFO_sCAL, "png_get_sCAL_s edge valid bit");

         if (ok)
         {
            int negative = 0;
            int zero = 0;
            ok &= check_true(parse_png_number(width_s, &negative, &zero) &&
               !negative, "png_set_sCAL edge width string");
            ok &= check_true(parse_png_number(height_s, &negative, &zero) &&
               !negative, "png_set_sCAL edge height string");
         }

         if (ok)
         {
            double width_change = fabs((width_out - width) / width);
            double height_change = fabs((height_out - height) / height);
            if (width_change > worst)
               worst = width_change;
            if (height_change > worst)
               worst = height_change;
         }
      }

      png_destroy_write_struct(&png_ptr, &info_ptr);
   }

   for (i=0; i<count; ++i)
   {
      test_context context;
      png_structp png_ptr;
      png_infop info_ptr;
      int ok = 1;
      int unit = 0;
      double width;
      double height;
      double width_out = 0;
      double height_out = 0;

      memset(&context, 0, sizeof context);

      if ((i % 11) == 0)
      {
         width = -1.0;
         height = 1.0;
      }
      else if ((i % 13) == 0)
      {
         width = 0.0;
         height = 1.0;
      }
      else
      {
         width = random_positive_double();
         height = random_positive_double();
      }

      png_ptr = make_write_struct(&context);
      if (png_ptr != NULL)
         info_ptr = png_create_info_struct(png_ptr);
      else
         info_ptr = NULL;

      ok &= check_true(png_ptr != NULL, "ascii random create write struct");
      ok &= check_true(info_ptr != NULL, "ascii random create info");

      if (ok && setjmp(png_jmpbuf(png_ptr)))
      {
         fprintf(stderr, "FAIL: ascii random case (%s)\n", context.message);
         ++failures;
         ok = 0;
      }

      if (ok)
      {
         png_set_sCAL(png_ptr, info_ptr, PNG_SCALE_METER, width, height);

         if (width <= 0 || height <= 0)
         {
            ok &= check_true(context.warning_count == 1,
               "png_set_sCAL invalid emits one warning");
            ok &= check_true(png_get_sCAL(png_ptr, info_ptr, &unit, &width_out,
               &height_out) == 0, "png_set_sCAL invalid stores nothing");
         }

         else
         {
            double width_change;
            double height_change;

            ok &= check_true(context.warning_count == 0,
               "png_set_sCAL valid emits no warning");
            ok &= check_true(png_get_sCAL(png_ptr, info_ptr, &unit, &width_out,
               &height_out) == PNG_INFO_sCAL, "png_get_sCAL random valid bit");
            ok &= check_scal_roundtrip(width_out, width,
               "png_set_sCAL random width roundtrip");
            ok &= check_scal_roundtrip(height_out, height,
               "png_set_sCAL random height roundtrip");

            width_change = fabs((width_out - width) / width);
            height_change = fabs((height_out - height) / height);
            if (width_change > worst)
               worst = width_change;
            if (height_change > worst)
               worst = height_change;
         }
      }

      png_destroy_write_struct(&png_ptr, &info_ptr);
   }

   printf("tarith: maximum observed sCAL relative error %.17g\n", worst);
#endif

#ifdef PNG_FIXED_POINT_SUPPORTED
   {
      int worst_fixed = 0;

      for (i=0; i<count; ++i)
      {
         test_context context;
         png_structp png_ptr;
         png_infop info_ptr;
         int ok = 1;
         int unit = 0;
         png_fixed_point width;
         png_fixed_point height;
         png_fixed_point width_out = 0;
         png_fixed_point height_out = 0;

         memset(&context, 0, sizeof context);

         if ((i % 17) == 0)
         {
            width = 0;
            height = 100000;
         }
         else if ((i % 19) == 0)
         {
            width = -100000;
            height = 100000;
         }
         else
         {
            width = (png_fixed_point)(random_u31_nonzero() % 500000000U);
            height = (png_fixed_point)(random_u31_nonzero() % 500000000U);
            if (width == 0) width = 1;
            if (height == 0) height = 1;
         }

         png_ptr = make_write_struct(&context);
         if (png_ptr != NULL)
            info_ptr = png_create_info_struct(png_ptr);
         else
            info_ptr = NULL;

         ok &= check_true(png_ptr != NULL, "fixed ascii create write struct");
         ok &= check_true(info_ptr != NULL, "fixed ascii create info");

         if (ok && setjmp(png_jmpbuf(png_ptr)))
         {
            fprintf(stderr, "FAIL: fixed ascii case (%s)\n", context.message);
            ++failures;
            ok = 0;
         }

         if (ok)
         {
            png_set_sCAL_fixed(png_ptr, info_ptr, PNG_SCALE_METER, width,
               height);

            if (width <= 0 || height <= 0)
            {
               ok &= check_true(context.warning_count == 1,
                  "png_set_sCAL_fixed invalid warns");
               ok &= check_true(png_get_sCAL_fixed(png_ptr, info_ptr, &unit,
                  &width_out, &height_out) == 0,
                  "png_set_sCAL_fixed invalid stores nothing");
            }

            else
            {
               int delta;

               ok &= check_true(context.warning_count == 0,
                  "png_set_sCAL_fixed valid emits no warning");
               ok &= check_true(png_get_sCAL_fixed(png_ptr, info_ptr, &unit,
                  &width_out, &height_out) == PNG_INFO_sCAL,
                  "png_get_sCAL_fixed random valid bit");
               ok &= check_close_long(width_out, width, 1,
                  "png_set_sCAL_fixed width roundtrip");
               ok &= check_close_long(height_out, height, 1,
                  "png_set_sCAL_fixed height roundtrip");

               delta = (int)llabs((long long)width_out - width);
               if (delta > worst_fixed)
                  worst_fixed = delta;
               delta = (int)llabs((long long)height_out - height);
               if (delta > worst_fixed)
                  worst_fixed = delta;
            }
         }

         png_destroy_write_struct(&png_ptr, &info_ptr);
      }

      printf("tarith: maximum observed sCAL_fixed error %d\n", worst_fixed);
   }
#endif

   return failures == 0;
}

static void
checkfp_one(scal_acceptor *acceptor, png_const_charp string,
   unsigned long *tested, unsigned long *accepted, unsigned long *rejected)
{
   int negative = 0;
   int zero = 0;
   int is_valid = parse_png_number(string, &negative, &zero);
   int expect_accept = is_valid && !negative;
   int actual_accept = scal_acceptor_accepts(acceptor, string);

   ++*tested;

   if (actual_accept)
      ++*accepted;
   else
      ++*rejected;

   if (actual_accept != expect_accept)
   {
      char described[128];
      describe_string(string, described, sizeof described);
      fprintf(stderr, "FAIL: sCAL parser mismatch for '%s' expected %s got %s\n",
         described, expect_accept ? "accept" : "reject",
         actual_accept ? "accept" : "reject");
      ++failures;
   }
}

static void
checkfp_recursive(scal_acceptor *acceptor, char *buffer, int depth,
   const char *alphabet, int alphabet_length, unsigned long *tested,
   unsigned long *accepted, unsigned long *rejected)
{
   int i;

   if (depth > 0)
      checkfp_one(acceptor, buffer, tested, accepted, rejected);

   if (depth == checkfp_max_length)
      return;

   for (i=0; i<alphabet_length; ++i)
   {
      buffer[depth] = alphabet[i];
      buffer[depth+1] = 0;
      checkfp_recursive(acceptor, buffer, depth+1, alphabet, alphabet_length,
         tested, accepted, rejected);
   }
}

static int
validation_checkfp(void)
{
   static const char alphabet[] =
      "+-.eE0123456789aA# \t\n\r\x7f";
   scal_acceptor acceptor;
   char buffer[16];
   unsigned long tested = 0;
   unsigned long accepted = 0;
   unsigned long rejected = 0;
   int ch;

   printf("tarith: exhaustively validating public sCAL parser strings up to length %d\n",
      checkfp_max_length);

   if (!scal_acceptor_init(&acceptor))
      return 0;

   buffer[0] = 0;
   checkfp_recursive(&acceptor, buffer, 0, alphabet,
      (int)(sizeof alphabet - 1), &tested, &accepted, &rejected);

   for (ch=1; ch<256; ++ch)
   {
      buffer[0] = (char)ch;
      buffer[1] = 0;
      checkfp_one(&acceptor, buffer, &tested, &accepted, &rejected);
   }

   scal_acceptor_destroy(&acceptor);

   printf("tarith: checked %lu public parser strings (%lu accepted, %lu rejected)\n",
      tested, accepted, rejected);
   return failures == 0;
}

static int
validation_muldiv(int count)
{
   test_context context;
   png_structp png_ptr;
   png_infop info_ptr;
   int ok = 1;
   int i;
   static const png_uint_32 equal_ppm_cases[] =
   {
      1U, 2U, 3U, 127U, 5000U, 3779U, 10000U, 1000000U, PNG_UINT_31_MAX
   };

   printf("tarith: validating public fixed-point conversions with %d random cases\n",
      count);

   memset(&context, 0, sizeof context);
   png_ptr = make_write_struct(&context);
   if (png_ptr != NULL)
      info_ptr = png_create_info_struct(png_ptr);
   else
      info_ptr = NULL;

   ok &= check_true(png_ptr != NULL, "muldiv create write struct");
   ok &= check_true(info_ptr != NULL, "muldiv create info");

   if (ok && setjmp(png_jmpbuf(png_ptr)))
   {
      fprintf(stderr, "FAIL: muldiv validation (%s)\n", context.message);
      ++failures;
      ok = 0;
   }

   if (ok)
   {
      for (i=0; i<(int)(sizeof equal_ppm_cases / sizeof equal_ppm_cases[0]); ++i)
      {
         png_uint_32 ppm = equal_ppm_cases[i];

         png_set_pHYs(png_ptr, info_ptr, ppm, ppm, PNG_RESOLUTION_METER);
         ok &= check_true(png_get_pixels_per_meter(png_ptr, info_ptr) == ppm,
            "png_get_pixels_per_meter edge");
         ok &= check_true(png_get_pixels_per_inch(png_ptr, info_ptr) ==
            expected_ppi(ppm), "png_get_pixels_per_inch edge");
      }

      png_set_pHYs(png_ptr, info_ptr, 3000, 1000, PNG_RESOLUTION_METER);
      ok &= check_true(png_get_pixels_per_meter(png_ptr, info_ptr) == 0,
         "png_get_pixels_per_meter unequal axes");
#ifdef PNG_FLOATING_POINT_SUPPORTED
      ok &= check_close_double(png_get_pixel_aspect_ratio(png_ptr, info_ptr),
         1.0/3.0, 1e-6, "png_get_pixel_aspect_ratio explicit");
#endif
#ifdef PNG_FIXED_POINT_SUPPORTED
      {
         int ratio_ok = 0;
         png_fixed_point expect = expected_ratio_fixed(3000, 1000, &ratio_ok);
         ok &= check_true(ratio_ok, "expected aspect ratio explicit valid");
         ok &= check_true(png_get_pixel_aspect_ratio_fixed(png_ptr, info_ptr) ==
            expect, "png_get_pixel_aspect_ratio_fixed explicit");
      }
#endif

      png_set_oFFs(png_ptr, info_ptr, 25400, 12700, PNG_OFFSET_MICROMETER);
#ifdef PNG_FLOATING_POINT_SUPPORTED
      ok &= check_close_double(png_get_x_offset_inches(png_ptr, info_ptr), 1.0,
         5e-6, "png_get_x_offset_inches explicit");
      ok &= check_close_double(png_get_y_offset_inches(png_ptr, info_ptr), 0.5,
         5e-6, "png_get_y_offset_inches explicit");
#endif
#ifdef PNG_FIXED_POINT_SUPPORTED
      {
         int inches_ok = 0;
         png_fixed_point expect = expected_inches_fixed(25400, &inches_ok);
         ok &= check_true(inches_ok, "expected inches explicit valid");
         ok &= check_true(png_get_x_offset_inches_fixed(png_ptr, info_ptr) ==
            expect, "png_get_x_offset_inches_fixed explicit");
         expect = expected_inches_fixed(12700, &inches_ok);
         ok &= check_true(inches_ok, "expected inches explicit valid y");
         ok &= check_true(png_get_y_offset_inches_fixed(png_ptr, info_ptr) ==
            expect, "png_get_y_offset_inches_fixed explicit");
      }
#endif

      png_set_pHYs(png_ptr, info_ptr, 1, PNG_UINT_31_MAX, PNG_RESOLUTION_METER);
#ifdef PNG_FIXED_POINT_SUPPORTED
      ok &= check_true(png_get_pixel_aspect_ratio_fixed(png_ptr, info_ptr) == 0,
         "png_get_pixel_aspect_ratio_fixed overflow");
#endif

      png_set_oFFs(png_ptr, info_ptr, PNG_UINT_31_MAX, PNG_UINT_31_MAX,
         PNG_OFFSET_MICROMETER);
#ifdef PNG_FIXED_POINT_SUPPORTED
      ok &= check_true(png_get_x_offset_inches_fixed(png_ptr, info_ptr) == 0,
         "png_get_x_offset_inches_fixed overflow");
      ok &= check_true(png_get_y_offset_inches_fixed(png_ptr, info_ptr) == 0,
         "png_get_y_offset_inches_fixed overflow");
#endif

      for (i=0; i<count; ++i)
      {
         png_uint_32 x_ppu = random_u31_nonzero();
         png_uint_32 y_ppu = ((i % 3) == 0) ? x_ppu : random_u31_nonzero();
         png_int_32 x_off = random_i31();
         png_int_32 y_off = random_i31();

         png_set_pHYs(png_ptr, info_ptr, x_ppu, y_ppu, PNG_RESOLUTION_METER);
         ok &= check_true(png_get_x_pixels_per_meter(png_ptr, info_ptr) == x_ppu,
            "png_get_x_pixels_per_meter random");
         ok &= check_true(png_get_y_pixels_per_meter(png_ptr, info_ptr) == y_ppu,
            "png_get_y_pixels_per_meter random");

         if (x_ppu == y_ppu)
         {
            ok &= check_true(png_get_pixels_per_meter(png_ptr, info_ptr) == x_ppu,
               "png_get_pixels_per_meter random equal");
            ok &= check_true(png_get_pixels_per_inch(png_ptr, info_ptr) ==
               expected_ppi(x_ppu), "png_get_pixels_per_inch random equal");
         }

         else
            ok &= check_true(png_get_pixels_per_meter(png_ptr, info_ptr) == 0,
               "png_get_pixels_per_meter random unequal");

#ifdef PNG_FIXED_POINT_SUPPORTED
         {
            int ratio_ok = 0;
            png_fixed_point expect_ratio = expected_ratio_fixed(x_ppu, y_ppu,
               &ratio_ok);
            png_fixed_point actual_ratio =
               png_get_pixel_aspect_ratio_fixed(png_ptr, info_ptr);

            if (ratio_ok)
               ok &= check_true(actual_ratio == expect_ratio,
                  "png_get_pixel_aspect_ratio_fixed random");
            else
               ok &= check_true(actual_ratio == 0,
                  "png_get_pixel_aspect_ratio_fixed random overflow");
         }
#endif

         png_set_oFFs(png_ptr, info_ptr, x_off, y_off, PNG_OFFSET_MICROMETER);

#ifdef PNG_FIXED_POINT_SUPPORTED
         {
            int inches_ok = 0;
            png_fixed_point expect_x = expected_inches_fixed(x_off, &inches_ok);
            png_fixed_point actual_x =
               png_get_x_offset_inches_fixed(png_ptr, info_ptr);

            if (inches_ok)
               ok &= check_true(actual_x == expect_x,
                  "png_get_x_offset_inches_fixed random");
            else
               ok &= check_true(actual_x == 0,
                  "png_get_x_offset_inches_fixed random overflow");

            expect_x = expected_inches_fixed(y_off, &inches_ok);
            actual_x = png_get_y_offset_inches_fixed(png_ptr, info_ptr);
            if (inches_ok)
               ok &= check_true(actual_x == expect_x,
                  "png_get_y_offset_inches_fixed random");
            else
               ok &= check_true(actual_x == 0,
                  "png_get_y_offset_inches_fixed random overflow");
         }
#endif
      }
   }

   png_destroy_write_struct(&png_ptr, &info_ptr);
   return ok;
}

static int
validation_gamma(void)
{
   static const double gamma_values[] =
   {
      2.2, 1.8, 1.52, 1.45, 1.0, 1.0/1.45, 1.0/1.52, 1.0/1.8, 1.0/2.2
   };
   memory_buffer ramp8, ramp16;
   int ok = 1;
   int i;

   printf("tarith: validating gamma ramps through public APIs\n");

   memset(&ramp8, 0, sizeof ramp8);
   memset(&ramp16, 0, sizeof ramp16);

   ok &= write_gray_ramp_png(&ramp8, 8, PNG_FP_1);
#if defined(PNG_READ_16BIT_SUPPORTED) && defined(PNG_WRITE_16BIT_SUPPORTED)
   ok &= write_gray_ramp_png(&ramp16, 16, PNG_FP_1);
#endif

   for (i=0; ok && i<(int)(sizeof gamma_values / sizeof gamma_values[0]); ++i)
   {
      double exponent = gamma_values[i];
      png_fixed_point screen = (png_fixed_point)floor(
         (1.0 / exponent) * PNG_FP_1 + .5);

#ifdef PNG_FLOATING_POINT_SUPPORTED
      ok &= verify_gamma_ramp(&ramp8, 8, screen, 0, exponent,
         "gamma float 8-bit ramp");
#  if defined(PNG_READ_16BIT_SUPPORTED) && defined(PNG_WRITE_16BIT_SUPPORTED)
      ok &= verify_gamma_ramp(&ramp16, 16, screen, 0, exponent,
         "gamma float 16-bit ramp");
#  endif
#endif

#ifdef PNG_FIXED_POINT_SUPPORTED
      ok &= verify_gamma_ramp(&ramp8, 8, screen, 1, exponent,
         "gamma fixed 8-bit ramp");
#  if defined(PNG_READ_16BIT_SUPPORTED) && defined(PNG_WRITE_16BIT_SUPPORTED)
      ok &= verify_gamma_ramp(&ramp16, 16, screen, 1, exponent,
         "gamma fixed 16-bit ramp");
#  endif
#endif
   }

   free(ramp8.data);
   free(ramp16.data);
   return ok;
}

static int
run_ascii_tests(void)
{
   int start_failures = failures;

   validation_ascii_to_fp(test_iterations);
   return failures == start_failures;
}

static int
run_checkfp_tests(void)
{
   int start_failures = failures;

   validation_checkfp();
   return failures == start_failures;
}

static int
run_muldiv_tests(void)
{
   int start_failures = failures;

   validation_muldiv(test_iterations);
   return failures == start_failures;
}

static int
run_gamma_tests(void)
{
   int start_failures = failures;

   validation_gamma();
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

static int
parse_positive_int(const char *text)
{
   int value = atoi(text);

   if (value <= 0)
      value = 1;

   return value;
}

int
main(int argc, char **argv)
{
   int argi = 1;
   const char *command = "all";

   while (argi < argc)
   {
      if (strcmp(argv[argi], "-v") == 0)
         ++argi;

      else if (argi + 1 < argc && strcmp(argv[argi], "-c") == 0)
      {
         test_iterations = parse_positive_int(argv[argi+1]);
         argi += 2;
      }

      else
         break;
   }

   if (argi < argc)
      command = argv[argi++];

   if (strcmp(command, "checkfp") == 0)
   {
      while (argi < argc)
      {
         if (argi + 1 < argc && strcmp(argv[argi], "-l") == 0)
         {
            checkfp_max_length = parse_positive_int(argv[argi+1]);
            argi += 2;
         }

         else
         {
            fprintf(stderr,
               "usage: tarith [-v] [-c count] [all|ascii|checkfp|muldiv|gamma] [args]\n");
            fprintf(stderr, "       checkfp args: -l max-length\n");
            return 1;
         }
      }
   }

   else if (argi < argc)
   {
      fprintf(stderr,
         "usage: tarith [-v] [-c count] [all|ascii|checkfp|muldiv|gamma] [args]\n");
      fprintf(stderr, "       checkfp args: -l max-length\n");
      return 1;
   }

   if (strcmp(command, "all") == 0)
      run_all_tests();

   else if (strcmp(command, "ascii") == 0)
      run_ascii_tests();

   else if (strcmp(command, "checkfp") == 0)
      run_checkfp_tests();

   else if (strcmp(command, "muldiv") == 0)
      run_muldiv_tests();

   else if (strcmp(command, "gamma") == 0)
      run_gamma_tests();

   else
   {
      fprintf(stderr,
         "usage: tarith [-v] [-c count] [all|ascii|checkfp|muldiv|gamma] [args]\n");
      fprintf(stderr, "       checkfp args: -l max-length\n");
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
