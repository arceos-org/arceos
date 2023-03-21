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

#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2

#define _SC_PAGESIZE 30

typedef unsigned int __off_t;
// struct _IO_FILE
// {
//   int _flags;		/* High-order word is _IO_MAGIC; rest is flags. */

//   /* The following pointers correspond to the C++ streambuf protocol. */
//   char *_IO_read_ptr;	/* Current read pointer */
//   char *_IO_read_end;	/* End of get area. */
//   char *_IO_read_base;	/* Start of putback+get area. */
//   char *_IO_write_base;	/* Start of put area. */
//   char *_IO_write_ptr;	/* Current put pointer. */
//   char *_IO_write_end;	/* End of put area. */
//   char *_IO_buf_base;	/* Start of reserve area. */
//   char *_IO_buf_end;	/* End of reserve area. */

//   /* The following fields are used to support backing up and undo. */
//   char *_IO_save_base; /* Pointer to start of non-current get area. */
//   char *_IO_backup_base;  /* Pointer to first valid character of backup area */
//   char *_IO_save_end; /* Pointer to end of non-current get area. */

//   struct _IO_marker *_markers;

//   struct _IO_FILE *_chain;

//   int _fileno;
//   int _flags2;
//   __off_t _old_offset; /* This used to be _offset but it's too small.  */

//   /* 1+column number of pbase(); 0 is unknown. */
//   unsigned short _cur_column;
//   signed char _vtable_offset;
//   char _shortbuf[1];

// //   _IO_lock_t *_lock;
// };

// typedef struct _IO_FILE FILE;

#endif // __SYS_TYPES_H__
