#include "../user_libc.h"

int c_main() {
    int *p = malloc(15);
    for(int i = 0; i < 15; i++) {
        p[i] = 1000 + i;
    }
    dummy_syscall((size_t)p, *(p + 2));
    free(p);
    p = NULL;
    return 7;
}