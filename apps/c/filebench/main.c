#include <fcntl.h>
#include <stdio.h>
#include <time.h>
#include <unistd.h>
int main()
{
    // c语言下生成 timesepc 指针
    clock_t start, end;
    start = clock();

    for (int i = 0; i < 100; i = i + 1) {
        int fd = open("hello", O_RDONLY);
        close(fd);
    }

    end = clock();
    printf("Time: %ld\n", end - start);
    return 0;
}