#include <stdio.h>
typedef void* (*CallBackMalloc)(size_t size);
typedef void* (*CallBackMallocAligned)(size_t size,size_t align);
typedef void (*CallBackFree)(void* ptr,size_t size);
CallBackMalloc cb1_glibc_bench;
CallBackMallocAligned cb2_glibc_bench;
CallBackFree cb3_glibc_bench;

void* glibc_bench_malloc(size_t size){
    return cb1_glibc_bench(size);
}
void* glibc_bench_malloc_aligned(size_t size,size_t align){
    return cb2_glibc_bench(size,align);
}
void glibc_bench_free(void* ptr,size_t size){
    cb3_glibc_bench(ptr,size);
}

