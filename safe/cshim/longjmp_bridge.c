#include <setjmp.h>
#include <stddef.h>

typedef struct png_safe_longjmp_state {
    jmp_buf env;
} png_safe_longjmp_state;

typedef int (*png_safe_longjmp_callback_fn)(void *context);

size_t png_safe_longjmp_state_size(void) {
    return sizeof(png_safe_longjmp_state);
}

int png_safe_longjmp_state_set(void *storage) {
    png_safe_longjmp_state *state = (png_safe_longjmp_state *)storage;
    return setjmp(state->env);
}

int png_safe_longjmp_state_invoke(void *storage,
                                  png_safe_longjmp_callback_fn callback,
                                  void *context) {
    png_safe_longjmp_state *state = (png_safe_longjmp_state *)storage;

    if (setjmp(state->env) != 0) {
        return 0;
    }

    return callback(context);
}

void png_safe_longjmp_state_jump(void *storage, int value) {
    png_safe_longjmp_state *state = (png_safe_longjmp_state *)storage;
    longjmp(state->env, value);
}

void *png_safe_longjmp_state_buf(void *storage) {
    png_safe_longjmp_state *state = (png_safe_longjmp_state *)storage;
    return state->env;
}
