/* Benchmark malloc and free functions.
   Copyright (C) 2019-2021 Free Software Foundation, Inc.
   This file is part of the GNU C Library.

   The GNU C Library is free software; you can redistribute it and/or
   modify it under the terms of the GNU Lesser General Public
   License as published by the Free Software Foundation; either
   version 2.1 of the License, or (at your option) any later version.

   The GNU C Library is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
   Lesser General Public License for more details.

   You should have received a copy of the GNU Lesser General Public
   License along with the GNU C Library; if not, see
   <https://www.gnu.org/licenses/>.  */

// modified by Daan Leijen to fit the bench suite and add lifo/fifo free order.

#include <stdio.h>
#include <stdlib.h>
#include "glibc_bench.h"
// #include "bench-timing.h"
// #include "json-lib.h"

/* Benchmark the malloc/free performance of a varying number of blocks of a
   given size.  This enables performance tracking of the t-cache and fastbins.
   It tests 3 different scenarios: single-threaded using main arena,
   multi-threaded using thread-arena, and main arena with SINGLE_THREAD_P
   false.  */

#define NUM_ITERS 2000000
#define NUM_ALLOCS 4
#define MAX_ALLOCS 1600

// Daan: disable timing
typedef long timing_t;
#define TIMING_NOW(s) 
#define TIMING_DIFF(e,start,stop)


typedef struct
{
  size_t iters;
  size_t size;
  int n;
  timing_t elapsed;
} malloc_args;

static void
do_benchmark (malloc_args *args, char**arr)
{
  printf("do benchmark: %d %d %d %d\n",args->iters,args->size,args->n,args->elapsed);
  timing_t start, stop;
  size_t iters = args->iters;
  size_t size = args->size;
  int n = args->n;

  TIMING_NOW (start);

  int *_size = glibc_bench_malloc(n * sizeof(int));

  for (int j = 0; j < iters; j++)
    {
      for (int i = 0; i < n; i++) {
        arr[i] = glibc_bench_malloc (size);
        _size[i] = size;
        for(int g = 0; g < size; g++) { arr[i][g] =(char)g; }
      }

      // free half in fifo order
      for (int i = 0; i < n/2; i++) {
	      glibc_bench_free (arr[i],_size[i]);  
      }
    
      // and the other half in lifo order
      for(int i = n-1; i >= n/2; i--) {
        glibc_bench_free(arr[i],_size[i]);
      }
  }

  glibc_bench_free(_size, n * sizeof(int));


  TIMING_NOW (stop);

  TIMING_DIFF (args->elapsed, start, stop);
}

static malloc_args tests[3][NUM_ALLOCS];
static int allocs[NUM_ALLOCS] = { 25, 100, 400, MAX_ALLOCS };

static void *
thread_test (void *p)
{
  char **arr = (char**)p;

  /* Run benchmark multi-threaded.  */
  for (int i = 0; i < NUM_ALLOCS; i++)
    do_benchmark (&tests[2][i], arr);

  return p;
}

void
bench (unsigned long size)
{
  printf("bench: size = %d\n",size);
  size_t iters = NUM_ITERS;
  char**arr = (char**)glibc_bench_malloc (MAX_ALLOCS * sizeof (void*));

  for (int t = 0; t < 3; t++)
    for (int i = 0; i < NUM_ALLOCS; i++)
      {
	tests[t][i].n = allocs[i];
	tests[t][i].size = size;
	tests[t][i].iters = iters / allocs[i];

	/* Do a quick warmup run.  */
	if (t == 0)
	  do_benchmark (&tests[0][i], arr);
      }

  /* Run benchmark single threaded in main_arena.  */
  for (int i = 0; i < NUM_ALLOCS; i++)
    do_benchmark (&tests[0][i], arr);

  /* Repeat benchmark in main_arena with SINGLE_THREAD_P == false.  */
  for (int i = 0; i < NUM_ALLOCS; i++)
    do_benchmark (&tests[1][i], arr);

  glibc_bench_free (arr, MAX_ALLOCS * sizeof (void*));

}

void glibc_bench_test_start(CallBackMalloc _cb1,CallBackMallocAligned _cb2,CallBackFree _cb3) {
  cb1 = _cb1;
  cb2 = _cb2;
  cb3 = _cb3;
  long size = 16;

  bench (size);
  bench (2*size);
  bench (4*size);
}
