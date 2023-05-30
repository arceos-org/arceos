#include <stdio.h>
#include <assert.h>
#include <pthread.h>
#include <stdlib.h>
#include "multi_thread_c.h"

#define NUM_TASKS 10
#define MUN_TURN 100
#define NUM_ARRAY_PRE_THREAD 1000

void* mtc_a[NUM_TASKS * NUM_ARRAY_PRE_THREAD];
int mtc_size[NUM_TASKS * NUM_ARRAY_PRE_THREAD];
pthread_t mtc_th[NUM_TASKS];


void* func1(void* _tid){
    int tid = *((int*)_tid);
    //printf("thread %d func 1\n",tid);
    for(int i = 0;i < NUM_ARRAY_PRE_THREAD;++i){
        int size = (1 << (rand() % 12)) + (1 << (rand() % 12));
        int idx = i * NUM_TASKS + tid;
        void* ptr = multi_thread_c_malloc(size);
        mtc_a[idx] = ptr;
        mtc_size[idx] = size;
        //printf(")))%d %d %d\n",tid,idx,size);
    }
    for(int i = NUM_ARRAY_PRE_THREAD / 2;i < NUM_ARRAY_PRE_THREAD;++i){
        int idx = i * NUM_TASKS + tid;
        int size = mtc_size[idx];
        void* ptr = mtc_a[idx];
        multi_thread_c_free(ptr,size);
        mtc_a[idx] = NULL;
        mtc_size[idx] = 0;
        //printf("(((%d %d %d\n",tid,idx,size);
    }
    //printf("thread %d func 1 end\n",tid);
    return NULL;
}

void* func2(void* _tid){
    int tid = *((int*)_tid);
    //printf("thread %d func 2\n",tid);
    /*
    for(int i = 0;i < NUM_ARRAY_PRE_THREAD / 2;++i){
        int size = (1 << (rand() % 12)) + (1 << (rand() % 12));
        int idx = NUM_TASKS * NUM_ARRAY_PRE_THREAD / 2 + tid * NUM_ARRAY_PRE_THREAD / 2 + i;
        void* ptr = multi_thread_c_malloc(size);
        mtc_a[idx] = ptr;
        mtc_size[idx] = size;
        //printf("&&&%d %d %d\n",tid,idx,size);
    }
    */
    for(int i = 0;i < NUM_ARRAY_PRE_THREAD / 2;++i){
        int idx = i * NUM_TASKS + tid;
        int size = mtc_size[idx];
        //printf("%d %d %d\n",tid,idx,size);
        while(!size){
            //printf("^^^%d %d %d\n",tid,idx,size);
            size = mtc_size[idx];
        } 
        void* ptr = mtc_a[idx];
        multi_thread_c_free(ptr,size);
        mtc_a[idx] = NULL;
        mtc_size[idx] = 0;
    }
    //printf("thread %d func 2 end\n",tid);
    return NULL;
}

void multi_thread_c_test_start(CallBackMalloc _cb1,CallBackMallocAligned _cb2,CallBackFree _cb3) {
    cb1 = _cb1;
    cb2 = _cb2;
    cb3 = _cb3;
    printf("Hello multi_thread_test_c!\n");
    srand(2333);
    int *_tid = multi_thread_c_malloc(NUM_TASKS * sizeof(int));
    for(int j = 0;j < NUM_TASKS;++j){
        _tid[j] = j;
    }
    for(int i = 0;i < MUN_TURN;++i){
        for(int j = 0;j < NUM_TASKS;++j){
            pthread_create(&mtc_th[j],NULL,func1,&_tid[j]);
        }
        for(int j = 0;j < NUM_TASKS;++j){
            pthread_join(mtc_th[j],NULL);
        }
        for(int j = 0;j < NUM_TASKS;++j){
            pthread_create(&mtc_th[j],NULL,func2,&_tid[j]);
        }
        for(int j = 0;j < NUM_TASKS;++j){
            pthread_join(mtc_th[j],NULL);
        }
    }
    multi_thread_c_free(_tid, NUM_TASKS * sizeof(int));
}
