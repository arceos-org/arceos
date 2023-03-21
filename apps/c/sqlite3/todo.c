#include <stdio.h>
#include <stddef.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <unistd.h>
//#include <sys/time.h>
#include <stdarg.h> // for variable arguments functions
#include <fcntl.h>
#include <stdlib.h>
#include <sys/time.h>
#include <stdio.h>

long int sysconf(int name) {
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

int fcntl64(int fd, int cmd, ... /* arg */ ) {
    __asm__ ("call fcntl");
}

int open(const char *filename, int flags, ...){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    printf("open file: %s\n", filename);
    return -1;
}

off_t lseek(int fd, off_t offset, int whence){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

int gettimeofday(struct timeval *tv, struct timezone *tz){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

unsigned int sleep(unsigned int seconds){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

void *dlopen(const char *filename, int flag){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
}

char *dlerror(void){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

void *dlsym(void *handle, const char *symbol){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);

}

int dlclose(void *handle){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

int utimes(const char *filename, const struct timeval times[2]){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

struct tm *localtime(const time_t *timep){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

pid_t getpid(void){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    printf("getpid\n");
    return -1;
}

int fsync(int fd){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

time_t time(time_t *t){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

int close(int fd){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return -1;
}

int access(const char *pathname, int mode){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

char *getcwd(char *buf, size_t size){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

int lstat(const char *path, struct stat *buf){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

int stat(const char *path, struct stat *buf){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

int fstat(int fd, struct stat *buf){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

int ftruncate(int fd, off_t length){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

int fcntl(int fd, int cmd, ... /* arg */ ){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

ssize_t read(int fd, void *buf, size_t count){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

ssize_t write(int fd, const void *buf, size_t count){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

int fchmod(int fd, mode_t mode){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

int unlink(const char *pathname){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

int mkdir(const char *pathname, mode_t mode) {
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

int rmdir(const char *pathname){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

int fchown(int fd, uid_t owner, gid_t group){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

uid_t geteuid(void){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

char* getenv(const char *name){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

void *mmap(void *addr, size_t len, int prot, int flags, int fildes, off_t off){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
}

int munmap(void *addr, size_t length){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

void *mremap(void *old_address, size_t old_size, size_t new_size, int flags, ... /* void *new_address */){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
}

ssize_t readlink(const char *path, char *buf, size_t bufsiz){
    printf("%s%s\n", "Error: no ax_call implementation for ", __func__);
    return 0;
}

char* strerror(int n) {
    return "";
}

size_t strftime(char * __restrict__ _Buf,size_t _SizeInBytes,const char * __restrict__ _Format,const struct tm * __restrict__ _Tm) {
    return 0;
}

struct tm *gmtime(const time_t *timer) {
    return NULL;
}

int __clzdi2(int a)
{
    return 0;
}
