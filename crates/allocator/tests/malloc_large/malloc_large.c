// Test allocation large blocks between 2 and 5 MiB with up to 10 live at any time.
// Provided by Leonid Stolyarov in issue #447 and modified by Daan Leijen.
#include <stdio.h>
#include <stdlib.h>
#include "malloc_large.h"
int get_random(int xmin,int xmax){
  return rand() % (xmax - xmin + 1) + xmin;
}
void malloc_large_test_start(CallBackMalloc _cb1,CallBackMallocAligned _cb2,CallBackFree _cb3) {
  cb1 = _cb1;
  cb2 = _cb2;
  cb3 = _cb3;
  static const int kNumBuffers = 10;
  static const size_t kMinBufferSize = 2 * 1024 * 1024;//2MB
  static const size_t kMaxBufferSize = 5 * 1024 * 1024;//5MB
  char* buffers[kNumBuffers];
  int size[kNumBuffers];

  srand(42);
  static const int kNumIterations = 100000;
  for (int i = 0; i < kNumBuffers; ++i){
    buffers[i] = large_malloc(kMinBufferSize);
    size[i] = kMinBufferSize;
  }
  for (int i = 0; i < kNumIterations; ++i) {
    int buffer_idx = get_random(0, kNumBuffers - 1);
    size_t new_size = get_random(kMinBufferSize, kMaxBufferSize);
    large_free(buffers[buffer_idx],size[buffer_idx]);
    buffers[buffer_idx] = large_malloc(new_size);
    size[buffer_idx] = new_size;
  }
  for(int i = 0;i < kNumBuffers;++i){
    large_free(buffers[i],size[i]);
  }
}
