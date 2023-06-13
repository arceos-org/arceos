#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <assert.h>
#include <pthread.h>

void *ThreadFunc1(void *arg)
{
    if (arg == NULL) {
        puts("Pass NULL argument");
        return NULL;
    } else {
        char buf[64];
        sprintf(buf, "Recieve: %s", (char *)arg);
        puts(buf);
        return "Child thread return message";
    }
}

void *ThreadFunc2(void *arg)
{
    puts("A message before call pthread_exit");
    pthread_exit("Exit message");
    puts("This message should not be printed");
}

void test_create_join()
{
    int res;
    char *s = "Main thread pass message";
    void *thread_result;
    pthread_t t1, t2;
    res = pthread_create(&t1, NULL, ThreadFunc1, NULL);
    if (res != 0) {
        puts("fail to create thread1");
        return;
    }

    res = pthread_join(t1, NULL);
    if (res != 0) {
        puts("First pthread join fail");
    }

    res = pthread_create(&t2, NULL, ThreadFunc1, (void *)s);
    if (res != 0) {
        puts("fail to create thread2");
        return;
    }

    res = pthread_join(t2, &thread_result);
    if (res != 0) {
        puts("Second pthread join fail");
    }

    char buf[128];
    sprintf(buf, "%s", (char *)thread_result);
    puts(buf);
}

void test_create_exit()
{
    int res;
    void *thread_result;
    pthread_t t1;

    res = pthread_create(&t1, NULL, ThreadFunc2, NULL);
    if (res != 0) {
        puts("pthread create fail");
        return;
    }

    res = pthread_join(t1, &thread_result);
    if (res != 0) {
        puts("pthread join fail");
    }

    char buf[16];
    sprintf(buf, "%s", (char *)thread_result);
    puts(buf);
    return;
}

int main()
{
    pthread_t main_thread = pthread_self();
    assert(main_thread != 0);

    test_create_join();
    test_create_exit();
    puts("(C)Pthread basic tests run OK!");

    return 0;
}
