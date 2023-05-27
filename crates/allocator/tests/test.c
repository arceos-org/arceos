#include <stdio.h>
typedef int (*CallBack)(int x);

int hello(int x, CallBack cb){
    printf("hello c! %d\n",x);
    return cb(x + 1);
}