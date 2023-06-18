#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/time.h>
#include <time.h>
#include <unistd.h>

const int NUM_TASKS = 5;

int parse_time(char *buf, int sec, int usec)
{
    int n = 0;
    n += sprintf(buf, "%d.", sec);
    n += sprintf(buf + n, "%d", usec / 100000 % 10);
    n += sprintf(buf + n, "%d", usec / 10000 % 10);
    n += sprintf(buf + n, "%d", usec / 1000 % 10);
    n += sprintf(buf + n, "%d", usec / 100 % 10);
    n += sprintf(buf + n, "%d", usec / 10 % 10);
    n += sprintf(buf + n, "%d", usec % 10);
    return n;
}

void *tickfunc(void *arg)
{
    char buf[32];
    for (int i = 0; i < 30; i++) {
        sprintf(buf, "  tick %d", i);
        puts(buf);
        usleep(500000);
    }
    return NULL;
}

void *tickfunc2(void *arg)
{
    pid_t task_id = getpid();
    char buf0[128];
    char buf1[128];

    for (int j = 0; j < 3; j++) {
        int sleep_sec = *(int *)arg + 1;
        sprintf(buf0, "task %d sleep %d seconds (%d) ...", task_id, sleep_sec, j);
        puts(buf0);
        struct timespec before, later;
        clock_gettime(0, &before);
        sleep(sleep_sec);
        clock_gettime(0, &later);
        long sec = later.tv_sec - before.tv_sec;
        long nsec = later.tv_nsec - before.tv_nsec;
        if (nsec < 0) {
            sec -= 1;
            nsec += 1000000000;
        }
        int n = sprintf(buf1, "task %d actually sleep ", task_id);
        n += parse_time(buf1 + n, (int)sec, (int)nsec / 1000);
        sprintf(buf1 + n, " seconds (%d) ...", j);
        puts(buf1);
    }
    return NULL;
}

void main()
{
    puts("Hello, main task!");
    struct timespec before, later;
    clock_gettime(0, &before);
    sleep(1);
    clock_gettime(0, &later);
    long sec = later.tv_sec - before.tv_sec;
    long nsec = later.tv_nsec - before.tv_nsec;
    if (nsec < 0) {
        sec -= 1;
        nsec += 1000000000;
    }
    char buf[128] = {0};
    int n = sprintf(buf, "main task sleep for ");
    n += parse_time(buf + n, (int)sec, (int)nsec / 1000);
    sprintf(buf + n, "s");
    puts(buf);

    pthread_t t1;
    pthread_create(&t1, NULL, tickfunc, NULL);

    pthread_t tasks[NUM_TASKS + 1];
    int sleep_times[NUM_TASKS];

    for (int i = 0; i < NUM_TASKS; i++) {
        sleep_times[i] = i;
    }

    tasks[NUM_TASKS] = t1;

    for (int i = 0; i < NUM_TASKS; i++) {
        pthread_t t;
        pthread_create(&t, NULL, tickfunc2, (void *)&sleep_times[i]);
        tasks[i] = t;
    }

    for (int i = 0; i < NUM_TASKS + 1; i++) {
        pthread_join(tasks[i], NULL);
    }

    puts("(C)Sleep tests run OK!");
}
