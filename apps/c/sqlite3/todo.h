#ifndef __AX_STDIO_H__
#define __AX_STDIO_H__

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

// At this point we have already definitions needed for  ocall interface, so:
#define DO_NOT_REDEFINE_FOR_OCALL

// For open64 need to define this
#define O_TMPFILE (__O_TMPFILE | O_DIRECTORY)

#define SGX_SUCCESS 0
typedef int sgx_status_t;

long int sysconf(int name);
int fcntl64(int fd, int cmd, ... /* arg */ );
int open(const char *filename, int flags, ...);
off_t lseek(int fd, off_t offset, int whence);
int gettimeofday(struct timeval *tv, struct timezone *tz);
unsigned int sleep(unsigned int seconds);
void *dlopen(const char *filename, int flag);
char *dlerror(void);
void *dlsym(void *handle, const char *symbol);
int dlclose(void *handle);
int utimes(const char *filename, const struct timeval times[2]);
struct tm *localtime(const time_t *timep);
pid_t getpid(void);
int fsync(int fd);
time_t time(time_t *t);
int close(int fd);
int access(const char *pathname, int mode);
char *getcwd(char *buf, size_t size);
int lstat(const char *path, struct stat *buf);
int stat(const char *path, struct stat *buf);
int fstat(int fd, struct stat *buf);
int ftruncate(int fd, off_t length);
int fcntl(int fd, int cmd, ... /* arg */ );
ssize_t read(int fd, void *buf, size_t count);
ssize_t write(int fd, const void *buf, size_t count);
int fchmod(int fd, mode_t mode);
int unlink(const char *pathname);
int mkdir(const char *pathname, mode_t mode);
int rmdir(const char *pathname);
int fchown(int fd, uid_t owner, gid_t group);
uid_t geteuid(void);
char* getenv(const char *name);
void *mmap(void *addr, size_t len, int prot, int flags, int fildes, off_t off);
int munmap(void *addr, size_t length);
void *mremap(void *old_address, size_t old_size, size_t new_size, int flags, ... /* void *new_address */);
ssize_t readlink(const char *path, char *buf, size_t bufsiz);
char* strerror(int n);

#endif