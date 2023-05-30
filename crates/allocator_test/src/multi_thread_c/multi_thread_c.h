#include <stdio.h>
typedef void* (*CallBackMalloc)(size_t size);
typedef void* (*CallBackMallocAligned)(size_t size,size_t align);
typedef void (*CallBackFree)(void* ptr,size_t size);
CallBackMalloc cb1_multi_thread_c;
CallBackMallocAligned cb2_multi_thread_c;
CallBackFree cb3_multi_thread_c;

void* multi_thread_c_malloc(size_t size){
    return cb1_multi_thread_c(size);
}
void* multi_thread_c_malloc_aligned(size_t size,size_t align){
    return cb2_multi_thread_c(size,align);
}
void multi_thread_c_free(void* ptr,size_t size){
    cb3_multi_thread_c(ptr,size);
}

