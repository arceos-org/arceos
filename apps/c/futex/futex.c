#include <unistd.h>
#include <sys/syscall.h>
#include <linux/futex.h>

int main() {
    int futex_var = 0;

    // 创建一个 futex，初始值为 0
    int *futex_ptr = &futex_var;

    // 唤醒等待在 futex 上的线程（私有唤醒）
    syscall(SYS_futex, futex_ptr, FUTEX_WAKE_PRIVATE, 1, NULL, NULL, 0);

    return 0;
}
