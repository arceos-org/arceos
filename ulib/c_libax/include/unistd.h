#ifndef __UNISTD_H__
#define __UNISTD_H__

#include <stddef.h>
#include <sys/stat.h>
#include <sys/types.h>

#define _SC_PAGESIZE 30

#ifdef AX_CONFIG_FS
int close(int fd);
off_t lseek(int fd, off_t offset, int whence);
int fsync(int fd);

ssize_t read(int fd, void *buf, size_t count);
ssize_t write(int fd, const void *buf, size_t count);

int fchown(int fd, uid_t owner, gid_t group);

ssize_t readlink(const char *path, char *buf, size_t bufsiz);
int unlink(const char *pathname);
int rmdir(const char *pathname);
int ftruncate(int fd, off_t length);

int access(const char *pathname, int mode);
char *getcwd(char *buf, size_t size);
#endif

unsigned sleep(unsigned seconds);

uid_t geteuid(void);
pid_t getpid(void);

long int sysconf(int name);

#endif
