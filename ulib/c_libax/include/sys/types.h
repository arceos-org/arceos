#ifndef __SYS_TYPES_H__
#define __SYS_TYPES_H__
#include <stdint.h>

/**
 * https://stackoverflow.com/questions/9635702/in-posix-how-is-type-dev-t-getting-used
 * dev_t in current glibc (2.35) is 64-bit, with 32-bit major and minor numbers.
 */
/// <div rustbindgen replaces="DevT"></div>
typedef uint32_t dev_t;
/// <div rustbindgen replaces="OffT"></div>
typedef int64_t off_t;
/// <div rustbindgen replaces="UID"></div>
typedef uint32_t uid_t;
/// <div rustbindgen replaces="GID"></div>
typedef uint32_t gid_t;
/// <div rustbindgen replaces="ModeT"></div>
typedef uint32_t mode_t;
/// <div rustbindgen replaces="InoT"></div>
typedef uint32_t ino_t;
/**
 * https://stackoverflow.com/questions/15976290/how-to-compare-nlink-t-to-int
 */
/// <div rustbindgen replaces="NlinkT"></div>
typedef uint32_t nlink_t;
/// <div rustbindgen replaces="BlkSizeT"></div>
typedef int32_t blksize_t;
/// <div rustbindgen replaces="BlkCntT"></div>
typedef int32_t blkcnt_t;

typedef uint32_t __off_t;

#endif // __SYS_TYPES_H__
