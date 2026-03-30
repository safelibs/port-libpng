#include <setjmp.h>
#include <stddef.h>

typedef struct png_safe_longjmp_state {
    jmp_buf env;
} png_safe_longjmp_state;

size_t png_safe_longjmp_state_size(void) {
    return sizeof(png_safe_longjmp_state);
}

int png_safe_longjmp_state_set(void *storage) {
    png_safe_longjmp_state *state = (png_safe_longjmp_state *)storage;
    return setjmp(state->env);
}

void png_safe_longjmp_state_jump(void *storage, int value) {
    png_safe_longjmp_state *state = (png_safe_longjmp_state *)storage;
    longjmp(state->env, value);
}

void *png_safe_longjmp_state_buf(void *storage) {
    png_safe_longjmp_state *state = (png_safe_longjmp_state *)storage;
    return state->env;
}
