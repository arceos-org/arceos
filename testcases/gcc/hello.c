#include <fcntl.h>
#include <stdio.h>
int main()
{
    int fd = open("test1.txt", O_RDONLY);
    if (fd < 0) {
        printf("error1!");
        return 0;
    }

    renameat(AT_FDCWD, "test1.txt", AT_FDCWD, "test2.txt");

    int fd2 = open("test2.txt", O_RDONLY);
    if (fd2 < 0) {
        printf("error2!");
        return 0;
    }
    printf("success!");
    return 0;
}