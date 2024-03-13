#include <stdio.h>
#include <unistd.h>
#include <sys/wait.h>
#include <sys/resource.h>

#define BUFFER_SIZE 1024

int main() {
    printf("test readlink(\"/proc/self/exe\",...)\n");
 
    // creaate a new process
    pid_t pid = fork();
    int wstatus;
    if (pid == -1) {
        return -1;
    } else if (pid == 0) {
        // child process
        printf("test processs start\n");
        char *const dummy[1] = {NULL};
        execve("./readlink_test", dummy, dummy);
    } else {
        wait4(pid, &wstatus, WCONTINUED, NULL);
        if (WEXITSTATUS(wstatus) == 0) {
            printf("Test passed!\n");
        } else {
            printf("Test failed.\n");
        }
    }
    return 0;
}
