#define _GNU_SOURCE
#include <sys/prctl.h>
#include <stdio.h>

int main() {
    // 获取进程名称
    char name[16];
    if (prctl(PR_GET_NAME, (unsigned long) name) == -1) {
        perror("prctl");
        return 1;
    }

    printf("Process name: %s\n", name);

    return 0;
}
