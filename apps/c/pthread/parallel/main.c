#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define NUM_DATA  2000000
#define NUM_TASKS 16

uint64_t array[NUM_DATA] = {0};

uint64_t my_sqrt(uint64_t n)
{
    uint64_t x = n;
    while (1) {
        if (x * x <= n && (x + 1) * (x + 1) > n)
            return x;

        x = (x + n / x) / 2;
    }
}

void *ThreadFunc(void *arg)
{
    int id = *(int *)arg;
    int left = (NUM_DATA / NUM_TASKS) * id;
    int right =
        (left + (NUM_DATA / NUM_TASKS)) < NUM_DATA ? (left + (NUM_DATA / NUM_TASKS)) : NUM_DATA;

    char buf[512];
    sprintf(buf, "part %d: [%d, %d)", id, left, right);
    puts(buf);

    uint64_t *partial_sum = (uint64_t *)calloc(1, sizeof(uint64_t));
    for (int i = left; i < right; i++) *partial_sum += my_sqrt(array[i]);

    sprintf(buf, "part %d finished", id);
    puts(buf);
    return (void *)(partial_sum);
}

int main()
{
    for (int i = 0; i < NUM_DATA; i++) array[i] = rand();

    uint64_t expect = 0;
    for (int i = 0; i < NUM_DATA; i++) expect += my_sqrt(array[i]);

    int thread_part[NUM_TASKS];
    for (int i = 0; i < NUM_TASKS; i++) thread_part[i] = i;

    pthread_t tasks[NUM_TASKS];
    for (int i = 0; i < NUM_TASKS; i++) {
        pthread_t t;
        pthread_create(&t, NULL, ThreadFunc, (void *)(&thread_part[i]));
        tasks[i] = t;
    }

    uint64_t *partial_sum;
    uint64_t actual = 0;
    for (int i = 0; i < NUM_TASKS; i++) {
        pthread_join(tasks[i], (void *)(&partial_sum));
        actual += *partial_sum;
    }

    char buf[64];
    sprintf(buf, "actual sum = %lld", actual);
    puts(buf);

    if (actual == expect)
        puts("(C)Pthread parallel run OK!");
    else {
        puts("(C)Pthread parallel run FAIL!");
    }
    return 0;
}
