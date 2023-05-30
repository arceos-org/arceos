#include <stdio.h>
typedef void* (*CallBackMalloc)(size_t size);
typedef void* (*CallBackMallocAligned)(size_t size,size_t align);
typedef void (*CallBackFree)(void* ptr,size_t size);
CallBackMalloc cb1_malloc_large;
CallBackMallocAligned cb2_malloc_large;
CallBackFree cb3_malloc_large;

void* large_malloc(size_t size){
    return cb1_malloc_large(size);
}
void* large_malloc_aligned(size_t size,size_t align){
    return cb2_malloc_large(size,align);
}
void large_free(void* ptr,size_t size){
    cb3_malloc_large(ptr,size);
}

