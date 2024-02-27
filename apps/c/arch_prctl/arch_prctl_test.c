#include <stdio.h>
#include <stdint.h>
#include <sys/prctl.h>
#include <asm/prctl.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>

int main() {
    //获取当前 FS 寄存器的值
    unsigned long* current_fs_value = 0;
    if (arch_prctl(ARCH_GET_FS, &current_fs_value) != 0) {
        perror("arch_prctl(ARCH_GET_FS)");
        return 1;
    }
    printf("Current FS value set: 0x%lx\n", current_fs_value);
    
    // 设置新的 FS 寄存器的值
    unsigned long new_fs_value = current_fs_value;
    if (arch_prctl(ARCH_SET_FS, new_fs_value) != 0) {
        perror("arch_prctl(ARCH_SET_FS)");
        return 1;
    }

    printf("New FS value set: 0x%lx\n", new_fs_value);

    return 0;
}
