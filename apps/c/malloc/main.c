#define _XOPEN_SOURCE 600
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

int main()
{
    puts("Running memory tests...");
    uintptr_t *brk = (uintptr_t *)malloc(0);
    printf("top of heap=%p\n", brk);
    // c语言下生成 timesepc 指针
    struct timespec time1 = {0, 0};
    struct timespec time2 = {0, 0};
    int test_num[10] = {10, 20, 50, 100, 500, 1000, 2000, 5000, 10000};
    for (int k = 0; k < 9; k = k + 1) {

        int n = test_num[k];

        uintptr_t **p = (uintptr_t **)malloc(n * sizeof(uint64_t));

        clock_gettime(CLOCK_REALTIME, &time1);
        for (int i = 0; i < n; i = i + 1) {
            p[i] = (uintptr_t *)malloc(10 * sizeof(uint64_t));
        }

        for (int i = 0; i < n; i = i + 1) {
            free(p[i]);
        }

        free(p);

        for (int i = 0; i < n * 100; i = i + 1) {
            void *p = malloc(n * sizeof(uint64_t));
            free(p);
        }
        clock_gettime(CLOCK_REALTIME, &time2);
        long long duration =
            ((time2.tv_sec - time1.tv_sec) * 1000000000 + time2.tv_nsec - time1.tv_nsec);
        printf("Malloc: Num: %d, duration: %lld\n", n, duration);
    }
}