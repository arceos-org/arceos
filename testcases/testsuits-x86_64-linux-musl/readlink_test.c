#include <stdio.h>
#include <unistd.h>

#define BUFFER_SIZE 1024

int main() {
    printf("Readlink test:\n");
    
    char buffer[BUFFER_SIZE];
    ssize_t result = readlink("/proc/self/exe", buffer, sizeof(buffer) - 1);

    if (result != -1) {
        buffer[result] = '\0';
        printf("My symbolic link target path: %s\n", buffer);
    } else {
        perror("error on readlink(\"/proc/self/exe\")");
        return 1;
    }

    return 0;
}
