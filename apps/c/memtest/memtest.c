#include <sched.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/sysinfo.h>

void print_sysinfo()
{
    struct sysinfo info;
    sys_sysinfo(&info);
    printf("sysinfo begin:----------------------\n");
    printf("Uptime: %ld\n", info.uptime);
    printf("Load: %lu%% %lu%% %lu%%\n", info.loads[0] * 100 / FIXED_1,
           info.loads[1] * 100 / FIXED_1, info.loads[2] * 100 / FIXED_1);
    printf("Total RAM: 0x%lx\n", info.totalram);
    printf("Free RAM: 0x%lx\n", info.freeram);
    printf("Shared RAM: 0x%lx\n", info.sharedram);
    printf("Buffer RAM: 0x%lx\n", info.bufferram);
    printf("Total swap: 0x%lx\n", info.totalswap);
    printf("Free swap: 0x%lx\n", info.freeswap);
    printf("Number of processes: %hu\n", info.procs);
    printf("Total high memory size: 0x%lx\n", info.totalhigh);
    printf("Free high memory size: 0x%lx\n", info.freehigh);
    printf("Memory unit size in bytes: 0x%x\n", info.mem_unit);
    printf("sysinfo end----------------------\n");
}

int main()
{
    puts("Running memory tests...");
    uintptr_t *brk = (uintptr_t *)malloc(0);
    printf("top of heap=%p\n", brk);

    int n = 9;
    int i = 0;
    uintptr_t **p = (uintptr_t **)malloc(n * sizeof(uint64_t));
    printf("%d(+8)Byte allocated: p=%p\n", n * sizeof(uint64_t), p, p[1]);
    printf("allocate %d(+8)Byte for %d times:\n", sizeof(uint64_t), n);
    print_sysinfo();
    for (i = 0; i < n; i++) {
        p[i] = (uintptr_t *)malloc(sizeof(uint64_t));
        *p[i] = 233;
        printf("allocated addr=%p\n", p[i]);
    }
    print_sysinfo();
    for (i = 0; i < n; i++) {
        free(p[i]);
    }
    print_sysinfo();
    free(p);
    puts("Memory tests run OK!");
    return 0;
}
