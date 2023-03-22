#ifndef __SYS_STAT__
#define __SYS_STAT__

#include <sys/types.h>
#include <time.h>

#define O_EXCL   1 // TODO;
#define O_CREAT  2 // TODO;
#define O_RDONLY 3 // TODO;

struct stat {
    dev_t st_dev;         /* ID of device containing file*/
    ino_t st_ino;         /* inode number*/
    mode_t st_mode;       /* protection*/
    nlink_t st_nlink;     /* number of hard links*/
    uid_t st_uid;         /* user ID of owner*/
    gid_t st_gid;         /* group ID of owner*/
    dev_t st_rdev;        /* device ID (if special file)*/
    off_t st_size;        /* total size, in bytes*/
    blksize_t st_blksize; /* blocksize for filesystem I/O*/
    blkcnt_t st_blocks;   /* number of blocks allocated*/
    time_t st_atime;      /* time of last access*/
    time_t st_mtime;      /* time of last modification*/
    time_t st_ctime;      /* time of last status change*/
};

int fchmod(int fd, mode_t mode);
int mkdir(const char *pathname, mode_t mode);

#endif
