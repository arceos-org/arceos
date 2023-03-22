#ifndef __SYS_TYPES_H__
#define __SYS_TYPES_H__

typedef unsigned int uid_t;
typedef unsigned int gid_t;
typedef unsigned int mode_t;

/**
 * https://stackoverflow.com/questions/9635702/in-posix-how-is-type-dev-t-getting-used
 * dev_t in current glibc (2.35) is 64-bit, with 32-bit major and minor numbers.
 */
typedef unsigned int dev_t;

typedef long long int off_t;

typedef unsigned int ino_t;
/**
 * https://stackoverflow.com/questions/15976290/how-to-compare-nlink-t-to-int
 */
typedef unsigned int nlink_t;
typedef int blksize_t;
typedef int blkcnt_t;

typedef unsigned int __off_t;

#endif // __SYS_TYPES_H__
