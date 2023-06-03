# Week15

## OS重构

### 结构图对比

原先Arceos结构图：

![ArceOS](\figures\ArceOS.svg)



重构后StarryOS架构图：

![Starry](\figures\Starry.svg)



### 结构说明

* crates：与OS设计无关的公共组件
* modules：与OS设计更加耦合的组件
* apps：unikernel架构下的用户程序，继承原有ArceOS
* ulib：用户库，继承原有ArceOS

1. 为了实现宏内核架构体系，需要对原有Arceos的部分核心模块（如axtask）进行修改。为了防止合并时冲突过多，因此在对应模块下建立`macro*`为前缀的文件夹，存放为宏内核架构实现的内容。同时使用条件编译来选择是宏内核架构还是unikernel架构。

2. 为了实现linux APP兼容，需要实现一系列面向linux的系统调用。我们将系统调用的具体实现部分放在`starry_libax`部分，即以用户库的形式形成一个linux兼容层。通过调用上述模块提供的一系列接口，实现对应的linux 系统调用，并暴露给外界。这个系统兼容层与原有的`libax`进行对应，分别提供不同的接口与服务。

   

3. 模块部分放置可以为宏内核与unikernel尽可能共享的内容，通过条件编译等方式做到尽可能地为不同架构下的兼容层所调用。



#### 理解：umi等OS的模块与Arceos的模块区别

* Umi模块更加接近于Arceos的crates，实现的是某一个特定的功能，如内核堆分配器、随机数生成函数等，其具有较好的可移植性。
* Arceos的modules实现的是内核中的某一个核心模块，如虚存、信号、文件系统等内容。其需要使用到较多的crates与已经实现的modules。移植时的限制可能会更多一些，如使用一些特定的trait等内容。
* Arceos的modules自带的模块化特征，可以保证其通过条件编译等手段，组合实现不同架构下的不同模式。如是否使用宏内核架构，内核架构是否启用分页、信号、文件系统等内容。也就是说，模块化OS可以支持多种启动方式，避免无意义的全部编译。



4. 批量测试、单个测例测试等内容的启动实现在apps中，即通过apps实现内容来操控内核行为。



## 比赛进度

1. 合并后的OS已经可以通过所有初赛测例
2. 实现了lazy alloc与动态加载
3. 可以完成libc测例的批量加载并且统计了所有需要实现的系统调用（暂时）。



## 系统调用

需要实现的系统调用：

```rust
//! 系统调用实现
//!
//! 目前的系统调用规范参照比赛所提供的类似 Linux 系统调用实现。
//! 
//! 有一些注释的系统调用名，那些是 rCore 的约定实现
//! 
//! 这两种调用间比较容易混淆的区别是，比赛测例是用 C 写的，大部分数组都是 4 Byte，
//! 而 rCore 使用 rust，usize/isize 一般是 8 Byte。
//! 这导致一些传入地址(非字符串,字符串大家都是统一的 1Byte 类型)的大小有问题，
//! 如 sys_pipe() 在测例环境下需要将输入作为 *mut u32 而不是 *mut usize

//#![deny(missing_docs)]
UNKNOWN = usize::MAX, // 未识别的系统调用
GETCWD = 17,
DUP = 23,
DUP3 = 24,
FCNTL64 = 25,
IOCTL = 29,
MKDIR = 34,
UNLINKAT = 35,
LINKAT = 37,
UMOUNT = 39,
MOUNT = 40,
STATFS = 43,
ACCESS = 48,	// 不一定要有
CHDIR = 49,
CHMOD = 53,
OPEN = 56,
CLOSE = 57,
PIPE = 59,
GETDENTS64 = 61,
LSEEK = 62,
READ = 63,
WRITE = 64,
READV = 65,
WRITEV = 66,
PREAD = 67,
SENDFILE64 = 71,
PSELECT6 = 72,
PPOLL = 73,		// 不一定要有
READLINKAT = 78,
FSTATAT = 79,
FSTAT = 80,
FSYNC = 82,
FDATASYNC = 83,		// 不一定要有
UTIMENSAT = 88,
EXIT = 93,
EXIT_GROUP = 94,
SET_TID_ADDRESS = 96,
FUTEX = 98,
NANOSLEEP = 101,
GETITIMER = 102,		// 不一定要有
SETITIMER = 103,		// 不一定要有
CLOCK_GET_TIME = 113,
SYSLOG = 116,			// 不一定要有
YIELD = 124,
KILL = 129,
TKILL = 130,
SIGACTION = 134,
SIGPROCMASK = 135,
SIGTIMEDWAIT = 137,
SIGRETURN = 139,
TIMES = 153,
UNAME = 160,
GETRUSAGE = 165,		// 不一定要有
UMASK = 166,			// 不一定要有
PRCTL = 167,			// 不一定要有
GET_TIME_OF_DAY = 169,		
GETPID = 172,
GETPPID = 173,
GETUID = 174,
GETEUID = 175,
GETGID = 176,
GETEGID = 177,
GETTID = 178,
SYSINFO = 179,			// 不一定要有
SOCKET = 198,
BIND = 200,			// 不一定要有
LISTEN = 201,			// 不一定要有
ACCEPT = 202,			// 不一定要有
CONNECT = 203,			// 不一定要有
GETSOCKNAME = 204,		// 不一定要有
GETPEERNAME = 205,		// 不一定要有
SENDTO = 206,
RECVFROM = 207,
SETSOCKOPT = 208,		// 不一定要用
GETSOCKOPT = 209,		// 不一定要有
SHUDOWN = 210,			// 不一定要有
SENDMSG = 211,			// 不一定要有
RECVMSG = 212,			// 不一定要有
BRK = 214,
MUNMAP = 215,
CLONE = 220,
EXECVE = 221,
MMAP = 222,
MPROTECT = 226,
MSYNC = 227,			// 不一定要有
MADVISE = 233,			// 不一定要有
ACCEPT4 = 242,			// 不一定要有
WAIT4 = 260,
PRLIMIT64 = 261,
RENAMEAT2 = 276,		// 不一定要有
MEMBARRIER = 283,
```

### 文件系统

```rust
GETCWD = 17,
DUP = 23,
DUP3 = 24,
FCNTL64 = 25,
IOCTL = 29,
MKDIR = 34,
UNLINKAT = 35,
LINKAT = 37,
UMOUNT = 39,
MOUNT = 40,
STATFS = 43,
ACCESS = 48,	// 不一定要有
CHDIR = 49,
CHMOD = 53,
OPEN = 56,
CLOSE = 57,
PIPE = 59,
GETDENTS64 = 61,
LSEEK = 62,
READ = 63,
WRITE = 64,
READV = 65,
WRITEV = 66,
PREAD = 67,
SENDFILE64 = 71,
PSELECT6 = 72,
PPOLL = 73,		// 不一定要有
READLINKAT = 78,
FSTATAT = 79,
FSTAT = 80,
FSYNC = 82,
FDATASYNC = 83,		// 不一定要有
UTIMENSAT = 88,
RENAMEAT2 = 276		// 不一定要有
```

### 任务管理

```rust
EXIT = 93,
EXIT_GROUP = 94,
SET_TID_ADDRESS = 96,
FUTEX = 98,
NANOSLEEP = 101,
YIELD = 124,
KILL = 129,
TKILL = 130,
UMASK = 166,			// 不一定要有
GETPID = 172,
GETPPID = 173,
GETUID = 174,
GETEUID = 175,
GETGID = 176,
GETEGID = 177,
GETTID = 178,
CLONE = 220,
EXECVE = 221,
WAIT4 = 260,
```

### 信号相关

```rust
SIGACTION = 134,
SIGPROCMASK = 135,
SIGTIMEDWAIT = 137,
SIGRETURN = 139,
```

### socket相关

```rust
SOCKET = 198,
BIND = 200,			// 不一定要有
LISTEN = 201,			// 不一定要有
ACCEPT = 202,			// 不一定要有
CONNECT = 203,			// 不一定要有
GETSOCKNAME = 204,		// 不一定要有
GETPEERNAME = 205,		// 不一定要有
SENDTO = 206,
RECVFROM = 207,
SETSOCKOPT = 208,		// 不一定要用
GETSOCKOPT = 209,		// 不一定要有
SHUDOWN = 210,			// 不一定要有
SENDMSG = 211,			// 不一定要有
RECVMSG = 212,			// 不一定要有
ACCEPT4 = 242,			// 不一定要有
```

### 内存相关

```rust
BRK = 214,
MUNMAP = 215,
MMAP = 222,
MEMBARRIER = 283,
```

### 其他相关

```rust
GETITIMER = 102,		// 不一定要有
SETITIMER = 103,		// 不一定要有
CLOCK_GET_TIME = 113,
SYSLOG = 116,			// 不一定要有
TIMES = 153,
UNAME = 160,
GETRUSAGE = 165,		// 不一定要有
PRCTL = 167,			// 不一定要有
GET_TIME_OF_DAY = 169,	
PRLIMIT64 = 261,
```





## 后续安排

1. 根据系统调用进行分工，在合并后OS上进行开发。初步计划按照上述划分进行分类。
2. 与大实验同学进行沟通，看看后续可以怎么进行合并。