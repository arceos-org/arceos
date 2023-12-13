#include <stdlib.h>
#include <stdio.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>

void die(const char *msg) {
    fputs(msg, stderr);
    exit(-1);
}

void check_read() {
    int fd = open("/proc/interrupts", O_RDONLY);
    if (fd < 0) {
        die("open failed\n");
    }
    close(fd);
}

void check_write() {
    int fd = open("/proc/interrupts", O_WRONLY);
    if (fd >= 0) {
        int n = write(fd, "a", 1);
        if (n == 1) {
            die("write succeeded\n");
        }
        close(fd);
    }
}

int main() {
    check_write();
    if (remove("/proc/interrupts") == 0) {
        die("remove succeeded\n");
    }
    if (rename("/proc/interrupts", "/proc/interrupts2") == 0) {
        die("rename succeeded\n");
    }
    puts("interrupts-test: passed case 1");
    return 0;
}
