#include <stdio.h>

#include <unistd.h>

int main()
{
    if (fork() == 0) {
        printf("Hello from child [%d - %d]\n", getpid(), getppid());
    } else {
        printf("Hello from parent [%d - %d]\n", getpid(), getppid());
    }
}