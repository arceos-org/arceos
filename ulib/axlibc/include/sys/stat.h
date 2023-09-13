#ifndef __SYS_STAT_H__
#define __SYS_STAT_H__

#include <sys/time.h>
#include <sys/types.h>

struct stat {
    dev_t st_dev;             /* ID of device containing file*/
    ino_t st_ino;             /* inode number*/
    mode_t st_mode;           /* protection*/
    nlink_t st_nlink;         /* number of hard links*/
    uid_t st_uid;             /* user ID of owner*/
    gid_t st_gid;             /* group ID of owner*/
    dev_t st_rdev;            /* device ID (if special file)*/
    off_t st_size;            /* total size, in bytes*/
    blksize_t st_blksize;     /* blocksize for filesystem I/O*/
    blkcnt_t st_blocks;       /* number of blocks allocated*/
    struct timespec st_atime; /* time of last access*/
    struct timespec st_mtime; /* time of last modification*/
    struct timespec st_ctime; /* time of last status change*/
};

#define st_atime st_atim.tv_sec
#define st_mtime st_mtim.tv_sec
#define st_ctime st_ctim.tv_sec

#define S_IFMT 0170000

#define S_IFDIR  0040000
#define S_IFCHR  0020000
#define S_IFBLK  0060000
#define S_IFREG  0100000
#define S_IFIFO  0010000
#define S_IFLNK  0120000
#define S_IFSOCK 0140000

#define S_TYPEISMQ(buf)  0
#define S_TYPEISSEM(buf) 0
#define S_TYPEISSHM(buf) 0
#define S_TYPEISTMO(buf) 0

#define S_ISDIR(mode)  (((mode)&S_IFMT) == S_IFDIR)
#define S_ISCHR(mode)  (((mode)&S_IFMT) == S_IFCHR)
#define S_ISBLK(mode)  (((mode)&S_IFMT) == S_IFBLK)
#define S_ISREG(mode)  (((mode)&S_IFMT) == S_IFREG)
#define S_ISFIFO(mode) (((mode)&S_IFMT) == S_IFIFO)
#define S_ISLNK(mode)  (((mode)&S_IFMT) == S_IFLNK)
#define S_ISSOCK(mode) (((mode)&S_IFMT) == S_IFSOCK)

#ifndef S_IRUSR
#define S_ISUID 04000
#define S_ISGID 02000
#define S_ISVTX 01000
#define S_IRUSR 0400
#define S_IWUSR 0200
#define S_IXUSR 0100
#define S_IRWXU 0700
#define S_IRGRP 0040
#define S_IWGRP 0020
#define S_IXGRP 0010
#define S_IRWXG 0070
#define S_IROTH 0004
#define S_IWOTH 0002
#define S_IXOTH 0001
#define S_IRWXO 0007
#endif

int stat(const char *path, struct stat *buf);
int fstat(int fd, struct stat *buf);
int lstat(const char *path, struct stat *buf);

int fchmod(int fd, mode_t mode);
int chmod(const char *file, mode_t mode);
int mkdir(const char *pathname, mode_t mode);
mode_t umask(mode_t mask);
int fstatat(int, const char *__restrict, struct stat *__restrict, int);

#endif
