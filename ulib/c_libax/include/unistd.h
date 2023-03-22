#ifndef __UNISTD_H__
#define __UNISTD_H__

#include <stddef.h>
#include <sys/stat.h>
#include <sys/types.h>

#define _SC_PAGESIZE 30

long int sysconf(int name);
off_t lseek(int fd, off_t offset, int whence);
unsigned int sleep(unsigned int seconds);
pid_t getpid(void);
int fsync(int fd);
int close(int fd);
int access(const char *pathname, int mode);
char *getcwd(char *buf, size_t size);
int lstat(const char *path, struct stat *buf);
int stat(const char *path, struct stat *buf);
int fstat(int fd, struct stat *buf);
int ftruncate(int fd, off_t length);
ssize_t read(int fd, void *buf, size_t count);
ssize_t write(int fd, const void *buf, size_t count);
int unlink(const char *pathname);
int rmdir(const char *pathname);
int fchown(int fd, uid_t owner, gid_t group);
uid_t geteuid(void);
ssize_t readlink(const char *path, char *buf, size_t bufsiz);

#endif
