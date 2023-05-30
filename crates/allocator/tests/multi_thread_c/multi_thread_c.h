#include <stdio.h>
typedef void* (*CallBackMalloc)(size_t size);
typedef void* (*CallBackMallocAligned)(size_t size,size_t align);
typedef void (*CallBackFree)(void* ptr,size_t size);
CallBackMalloc cb1;
CallBackMallocAligned cb2;
CallBackFree cb3;

void* multi_thread_c_malloc(size_t size){
    return cb1(size);
}
void* multi_thread_c_malloc_aligned(size_t size,size_t align){
    return cb2(size,align);
}
void multi_thread_c_free(void* ptr,size_t size){
    cb3(ptr,size);
}

