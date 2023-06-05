#ifndef __FCNTL_H__
#define __FCNTL_H__

#include <sys/types.h>

#define O_CREAT     0100
#define O_EXCL      0200
#define O_NOCTTY    0400
#define O_TRUNC     01000
#define O_APPEND    02000
#define O_NONBLOCK  04000
#define O_DSYNC     010000
#define O_SYNC      04010000
#define O_RSYNC     04010000
#define O_DIRECTORY 0200000
#define O_NOFOLLOW  0400000
#define O_CLOEXEC   02000000

#define O_ASYNC     020000
#define O_DIRECT    040000
#define O_LARGEFILE 0100000
#define O_NOATIME   01000000
#define O_PATH      010000000
#define O_TMPFILE   020200000
#define O_NDELAY    O_NONBLOCK

#define O_SEARCH   O_PATH
#define O_EXEC     O_PATH
#define O_TTY_INIT 0

#define O_ACCMODE (03 | O_SEARCH)
#define O_RDONLY  00
#define O_WRONLY  01
#define O_RDWR    02

#define F_DUPFD 0
#define F_GETFD 1
#define F_SETFD 2
#define F_GETFL 3
#define F_SETFL 4

#define F_SETOWN 8
#define F_GETOWN 9
#define F_SETSIG 10
#define F_GETSIG 11

#if __LONG_MAX == 0x7fffffffL
#define F_GETLK  12
#define F_SETLK  13
#define F_SETLKW 14
#else
#define F_GETLK  5
#define F_SETLK  6
#define F_SETLKW 7
#endif

#define FD_CLOEXEC      1
#define F_DUPFD_CLOEXEC 1030

#define F_RDLCK 0
#define F_WRLCK 1
#define F_UNLCK 2

#define F_OK    0
#define R_OK    4
#define W_OK    2
#define X_OK    1
#define F_ULOCK 0
#define F_LOCK  1
#define F_TLOCK 2
#define F_TEST  3

struct flock {
    short l_type;
    short l_whence;
    off_t l_start;
    off_t l_len;
    pid_t l_pid;
};

int fcntl(int fd, int cmd, ... /* arg */);

#ifdef AX_CONFIG_FS
int open(const char *filename, int flags, ...);
#endif

#endif
