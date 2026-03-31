#include <setjmp.h>
#include <stddef.h>

#include "pngpriv.h"

size_t png_safe_longjmp_state_size(void) {
    return sizeof(jmp_buf);
}

jmp_buf *png_safe_longjmp_local_buffer(png_structrp png_ptr) {
    if (png_ptr == NULL) {
        return NULL;
    }

    return &png_ptr->jmp_buf_local;
}

jmp_buf *png_safe_longjmp_get_buffer(png_const_structrp png_ptr) {
    if (png_ptr == NULL) {
        return NULL;
    }

    return png_ptr->jmp_buf_ptr;
}

size_t png_safe_longjmp_get_size(png_const_structrp png_ptr) {
    if (png_ptr == NULL) {
        return 0;
    }

    return png_ptr->jmp_buf_size;
}

void png_safe_longjmp_set_fields(png_structrp png_ptr, png_longjmp_ptr longjmp_fn,
                                 jmp_buf *jmp_buf_ptr, size_t jmp_buf_size) {
    if (png_ptr == NULL) {
        return;
    }

    png_ptr->longjmp_fn = longjmp_fn;
    png_ptr->jmp_buf_ptr = jmp_buf_ptr;
    png_ptr->jmp_buf_size = jmp_buf_size;
}

void png_safe_longjmp_call(png_const_structrp png_ptr, int val) {
    if (png_ptr == NULL || png_ptr->longjmp_fn == NULL || png_ptr->jmp_buf_ptr == NULL) {
        return;
    }

    png_ptr->longjmp_fn(*png_ptr->jmp_buf_ptr, val);
}
