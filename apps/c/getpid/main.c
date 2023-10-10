#define _XOPEN_SOURCE 600
#define _GNU_SOURCE
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <unistd.h>

int main()
{
    int test_num[13] = {1, 2, 5, 10, 15, 20, 25, 50, 60, 70, 80, 90, 100};
    // c语言下生成 timesepc 指针
    struct timespec time1 = {0, 0};
    struct timespec time2 = {0, 0};

    for (int k = 0; k < 13; k = k + 1) {
        long long ans = 0;
        int num = test_num[k] * 1000;
        for (int i = 0; i < 10; i = i + 1) {
            clock_gettime(CLOCK_REALTIME, &time1);
            for (int j = 0; j < num; j = j + 1) {
                getpid();
            }

            clock_gettime(CLOCK_REALTIME, &time2);
            long long duration =
                ((time2.tv_sec - time1.tv_sec) * 1000000000 + time2.tv_nsec - time1.tv_nsec);
            ans += duration;
        }
        printf("GetPid: Num: %d, Time: nanos: %lld\n", num, ans / 10);
    }

    return 0;
}