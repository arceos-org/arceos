#include <pthread.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

const int ROUND = 5;

void *ChildFunc(void *arg)
{
    int *fd = (int *)arg;
    int i = 0;
    char buf[32];
    while (i++ < ROUND) {
        sprintf(buf, "Child thread send message(%d)", i);
        puts(buf);
        sprintf(buf, "I am child(%d)!", i);
        write(fd[1], buf, strlen(buf) + 1);
        sleep(1);
    }
    close(fd[1]);
}

void main()
{
    int fd[2];
    int ret = pipe(fd);
    if (ret != 0) {
        puts("Fail to create pipe");
        return;
    }

    pthread_t t1;
    pthread_create(&t1, NULL, ChildFunc, (void *)fd);

    char msg[100];
    int j = 0;
    while (j++ < ROUND) {
        read(fd[0], msg, 15);
        char buf[64];
        sprintf(buf, "Main thread recieve (%d): %s", j, msg);
        puts(buf);
    }

    puts("(C)Pipe tests run OK");
    return;
}
