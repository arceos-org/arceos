#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <signal.h>
#include <sys/types.h>
#include <sys/wait.h>

// // #define WEXITSTATUS(s) (((s)&0xff00) >> 8)
// // #define WIFEXITED(s)   (!WTERMSIG(s))
// #define WIFEXITED(s)   ((s)&0x7f)

int main(void)
{
    pid_t pid, wpid;
    int status;
    int i = 0;
    pid = fork();
    // printf(" %d \n", pid);
    if (pid == -1) {
        // fork失败
        perror("fork failed");
        exit(EXIT_FAILURE);
    } else if (pid == 0) {               //子进程
        printf("Child --- My Parent is %d\n", getppid());
        sleep(5);
        // 子进程的任务完成，现在退出
        printf("Child Process is exiting\n");
        exit(9); // 退出子进程
    } else if(pid > 0) {           //父进程
        wpid = wait(&status);   //等待回收子进程

        printf("Status %d\n", status);

        if(wpid == -1) {
            perror("wait error:");
            exit(1);
        }

        while(i < 3) {
            printf("Parent Pid = %d, SonPid = %d\n", getpid(), pid);
            sleep(1);
            i++;
        }

        printf("Parent: Status %d WIFEXITED(status) == %d\n", status, WIFEXITED(status));

        //WEXITSTATUS get the return code
        printf("Parent: Status %d The return code WEXITSTATUS(status) == %d\n", status, WEXITSTATUS(status));
    } else {
        perror("for error");
        exit(1);
    }

    return 0;
}