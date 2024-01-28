use axhal::{
    paging::MappingFlags,
    time::{current_time_nanos, nanos_to_ticks, MICROS_PER_SEC, NANOS_PER_MICROS, NANOS_PER_SEC},
};
use bitflags::*;
use core::panic;
pub const NSEC_PER_SEC: usize = 1_000_000_000;
bitflags! {
    /// 指定 sys_wait4 的选项
    pub struct WaitFlags: u32 {
        /// 不挂起当前进程，直接返回
        const WNOHANG = 1 << 0;
        /// 报告已执行结束的用户进程的状态
        const WIMTRACED = 1 << 1;
        /// 报告还未结束的用户进程的状态
        const WCONTINUED = 1 << 3;
    }
}
/// sys_times 中指定的结构体类型
#[repr(C)]
pub struct TMS {
    /// 进程用户态执行时间，单位为us
    pub tms_utime: usize,
    /// 进程内核态执行时间，单位为us
    pub tms_stime: usize,
    /// 子进程用户态执行时间和，单位为us
    pub tms_cutime: usize,
    /// 子进程内核态执行时间和，单位为us
    pub tms_cstime: usize,
}

/// sys_gettimeofday 中指定的类型
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

impl TimeVal {
    pub fn to_nanos(&self) -> usize {
        self.sec * NANOS_PER_SEC as usize + self.usec * NANOS_PER_MICROS as usize
    }

    pub fn from_micro(micro: usize) -> Self {
        TimeVal {
            sec: micro / (MICROS_PER_SEC as usize),
            usec: micro % (MICROS_PER_SEC as usize),
        }
    }

    pub fn to_ticks(&self) -> u64 {
        (self.sec * axconfig::TIMER_FREQUENCY) as u64
            + nanos_to_ticks((self.usec as u64) * NANOS_PER_MICROS)
    }
}

/// sys_gettimer / sys_settimer 指定的类型，用户输入输出计时器
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ITimerVal {
    pub it_interval: TimeVal,
    pub it_value: TimeVal,
}

// sys_nanosleep指定的结构体类型
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct TimeSecs {
    pub tv_sec: usize,
    pub tv_nsec: usize,
}
/// 当 nsec 为这个特殊值时，指示修改时间为现在
pub const UTIME_NOW: usize = 0x3fffffff;
/// 当 nsec 为这个特殊值时，指示不修改时间
pub const UTIME_OMIT: usize = 0x3ffffffe;
impl TimeSecs {
    /// 根据当前的时间构造一个 TimeSecs
    pub fn now() -> Self {
        let nano = current_time_nanos() as usize;
        let tv_sec = nano / NSEC_PER_SEC;
        let tv_nsec = nano - tv_sec * NSEC_PER_SEC;
        TimeSecs { tv_sec, tv_nsec }
    }

    pub fn to_nano(&self) -> usize {
        self.tv_sec * NSEC_PER_SEC + self.tv_nsec
    }

    pub fn get_ticks(&self) -> usize {
        self.tv_sec * axconfig::TIMER_FREQUENCY + (nanos_to_ticks(self.tv_nsec as u64) as usize)
    }

    pub fn set_as_utime(&mut self, other: &TimeSecs) {
        match other.tv_nsec {
            UTIME_NOW => {
                *self = TimeSecs::now();
            } // 设为当前时间
            UTIME_OMIT => {} // 忽略
            _ => {
                *self = *other;
            } // 设为指定时间
        }
    }
}

bitflags! {
    #[derive(Debug)]
    /// 指定 mmap 的选项
    pub struct MMAPPROT: u32 {
        /// 区域内容可读取
        const PROT_READ = 1 << 0;
        /// 区域内容可修改
        const PROT_WRITE = 1 << 1;
        /// 区域内容可执行
        const PROT_EXEC = 1 << 2;
    }
}

impl Into<MappingFlags> for MMAPPROT {
    fn into(self) -> MappingFlags {
        let mut flags = MappingFlags::USER;
        if self.contains(MMAPPROT::PROT_READ) {
            flags |= MappingFlags::READ;
        }
        if self.contains(MMAPPROT::PROT_WRITE) {
            flags |= MappingFlags::WRITE;
        }
        if self.contains(MMAPPROT::PROT_EXEC) {
            flags |= MappingFlags::EXECUTE;
        }
        flags
    }
}

bitflags! {
    #[derive(Debug)]
    pub struct MMAPFlags: u32 {
        /// 对这段内存的修改是共享的
        const MAP_SHARED = 1 << 0;
        /// 对这段内存的修改是私有的
        const MAP_PRIVATE = 1 << 1;
        // 以上两种只能选其一

        /// 取消原来这段位置的映射，即一定要映射到指定位置
        const MAP_FIXED = 1 << 4;
        /// 不映射到实际文件
        const MAP_ANONYMOUS = 1 << 5;
        /// 映射时不保留空间，即可能在实际使用mmp出来的内存时内存溢出
        const MAP_NORESERVE = 1 << 14;
    }
}

/// sys_uname 中指定的结构体类型
#[repr(C)]
pub struct UtsName {
    /// 系统名称
    pub sysname: [u8; 65],
    /// 网络上的主机名称
    pub nodename: [u8; 65],
    /// 发行编号
    pub release: [u8; 65],
    /// 版本
    pub version: [u8; 65],
    /// 硬件类型
    pub machine: [u8; 65],
    /// 域名
    pub domainname: [u8; 65],
}

impl UtsName {
    /// 默认的 UtsName，并没有统一标准
    pub fn default() -> Self {
        Self {
            sysname: Self::from_str("YoimiyaOS"),
            nodename: Self::from_str("YoimiyaOS - machine[0]"),
            release: Self::from_str("114"),
            version: Self::from_str("1.0"),
            machine: Self::from_str("RISC-V 64 on SIFIVE FU740"),
            domainname: Self::from_str("https://github.com/Azure-stars/arceos"),
        }
    }

    fn from_str(info: &str) -> [u8; 65] {
        let mut data: [u8; 65] = [0; 65];
        data[..info.len()].copy_from_slice(info.as_bytes());
        data
    }
}

pub const SIGSET_SIZE_IN_BYTE: usize = 8;

pub enum SigMaskFlag {
    SigBlock = 0,
    SigUnblock = 1,
    SigSetmask = 2,
}

impl SigMaskFlag {
    pub fn from(value: usize) -> Self {
        match value {
            0 => SigMaskFlag::SigBlock,
            1 => SigMaskFlag::SigUnblock,
            2 => SigMaskFlag::SigSetmask,
            _ => panic!("SIG_MASK_FLAG::from: invalid value"),
        }
    }
}

/// sys_prlimit64 使用的数组
#[repr(C)]
pub struct RLimit {
    /// 软上限
    pub rlim_cur: u64,
    /// 硬上限
    pub rlim_max: u64,
}
// sys_prlimit64 使用的选项
/// 用户栈大小
pub const RLIMIT_STACK: i32 = 3;
/// 可以打开的 fd 数
pub const RLIMIT_NOFILE: i32 = 7;
/// 用户地址空间的最大大小
pub const RLIMIT_AS: i32 = 9;

/// robust list
#[repr(C)]
pub struct RobustList {
    pub head: usize,
    pub off: usize,
    pub pending: usize,
}

/// readv/writev使用的结构体
#[repr(C)]
pub struct IoVec {
    pub base: *mut u8,
    pub len: usize,
}
/// 对 futex 的操作
pub enum FutexFlags {
    /// 检查用户地址 uaddr 处的值。如果不是要求的值则等待 wake
    WAIT,
    /// 唤醒最多 val 个在等待 uaddr 位置的线程。
    WAKE,
    REQUEUE,
    UNSUPPORTED,
}

impl FutexFlags {
    pub fn new(val: i32) -> Self {
        match val & 0x7f {
            0 => FutexFlags::WAIT,
            1 => FutexFlags::WAKE,
            3 => FutexFlags::REQUEUE,
            _ => FutexFlags::UNSUPPORTED,
        }
    }
}

numeric_enum_macro::numeric_enum! {
    #[repr(usize)]
    #[allow(non_camel_case_types)]
    #[derive(Debug)]
    /// sys_fcntl64 使用的选项
    pub enum Fcntl64Cmd {
        /// 复制这个 fd，相当于 sys_dup
        F_DUPFD = 0,
        /// 获取 cloexec 信息，即 exec 成功时是否删除该 fd
        F_GETFD = 1,
        /// 设置 cloexec 信息，即 exec 成功时删除该 fd
        F_SETFD = 2,
        /// 获取 flags 信息
        F_GETFL = 3,
        /// 设置 flags 信息
        F_SETFL = 4,
        /// 复制 fd，然后设置 cloexec 信息，即 exec 成功时删除该 fd
        F_DUPFD_CLOEXEC = 1030,
    }
}

/// syscall_info 用到的 结构体
#[repr(C)]
#[derive(Debug)]
pub struct SysInfo {
    /// 启动时间(以秒计)
    pub uptime: isize,
    /// 1 / 5 / 15 分钟平均负载
    pub loads: [usize; 3],
    /// 内存总量，单位为 mem_unit Byte(见下)
    pub totalram: usize,
    /// 当前可用内存，单位为 mem_unit Byte(见下)
    pub freeram: usize,
    /// 共享内存大小，单位为 mem_unit Byte(见下)
    pub sharedram: usize,
    /// 用于缓存的内存大小，单位为 mem_unit Byte(见下)
    pub bufferram: usize,
    /// swap空间大小，即主存上用于替换内存中非活跃部分的空间大小，单位为 mem_unit Byte(见下)
    pub totalswap: usize,
    /// 可用的swap空间大小，单位为 mem_unit Byte(见下)
    pub freeswap: usize,
    /// 当前进程数，单位为 mem_unit Byte(见下)
    pub procs: u16,
    /// 高地址段的内存大小，单位为 mem_unit Byte(见下)
    pub totalhigh: usize,
    /// 可用的高地址段的内存大小，单位为 mem_unit Byte(见下)
    pub freehigh: usize,
    /// 指定 sys_info 的结构中用到的内存值的单位。
    /// 如 mem_unit = 1024, totalram = 100, 则指示总内存为 100K
    pub mem_unit: u32,
}

// sys_getrusage 用到的选项
#[allow(non_camel_case_types)]
pub enum RusageFlags {
    /// 获取当前进程的资源统计
    RUSAGE_SELF = 0,
    /// 获取当前进程的所有 **已结束并等待资源回收的** 子进程资源统计
    RUSAGE_CHILDREN = -1,
    /// 获取当前线程的资源统计
    RUSAGE_THREAD = 1,
}

impl RusageFlags {
    pub fn from(val: i32) -> Option<Self> {
        match val {
            0 => Some(RusageFlags::RUSAGE_SELF),
            -1 => Some(RusageFlags::RUSAGE_CHILDREN),
            1 => Some(RusageFlags::RUSAGE_THREAD),
            _ => None,
        }
    }
}

/// sched_setscheduler时指定子进程是否继承父进程的调度策略
pub const SCHED_RESET_ON_FORK: usize = 0x40000000;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SchedParam {
    pub sched_priority: usize,
}

numeric_enum_macro::numeric_enum! {
    #[repr(usize)]
    #[allow(non_camel_case_types)]
    #[derive(PartialEq,Eq)]
    /// sys_fcntl64 使用的选项
    pub enum ClockId {
        CLOCK_REALTIME = 0  ,
        CLOCK_MONOTONIC = 1     ,
        CLOCK_PROCESS_CPUTIME_ID = 2,
        CLOCK_THREAD_CPUTIME_ID = 3,
        CLOCK_MONOTONIC_RAW = 4,
        CLOCK_REALTIME_COARSE = 5,
        CLOCK_MONOTONIC_COARSE = 6,
        CLOCK_BOOTTIME = 7,
        CLOCK_REALTIME_ALARM = 8,
        CLOCK_BOOTTIME_ALARM = 9,
        CLOCK_SGI_CYCLE = 10,
        CLOCK_TAI = 11,
    }
}

// impl Clone for FilePath {
//     fn clone(&self) -> Self {
//         Self(self.0.clone())
//     }
// }

/// 目录项
#[repr(C)]
pub struct DirEnt {
    /// 索引结点号
    pub d_ino: u64,
    /// 到下一个dirent的偏移
    pub d_off: i64,
    /// 当前dirent的长度
    pub d_reclen: u16,
    /// 文件类型
    pub d_type: u8,
    /// 文件名
    pub d_name: [u8; 0],
}

#[allow(unused)]
pub enum DirEntType {
    /// 未知类型文件
    UNKNOWN = 0,
    /// 先进先出的文件/队列
    FIFO = 1,
    /// 字符设备
    CHR = 2,
    /// 目录
    DIR = 4,
    /// 块设备
    BLK = 6,
    /// 常规文件
    REG = 8,
    /// 符号链接
    LNK = 10,
    /// socket
    SOCK = 12,
    WHT = 14,
}

impl DirEnt {
    /// 定长部分大小
    pub fn fixed_size() -> usize {
        8 + 8 + 2 + 1
    }
    /// 设置定长部分
    pub fn set_fixed_part(&mut self, ino: u64, _off: i64, reclen: usize, type_: DirEntType) {
        self.d_ino = ino;
        self.d_off = -1;
        self.d_reclen = reclen as u16;
        self.d_type = type_ as u8;
    }
}

/// 文件系统的属性
/// 具体参数定义信息来自 `https://man7.org/linux/man-pages/man2/statfs64.2.html`
#[repr(C)]
#[derive(Debug)]
pub struct FsStat {
    /// 是个 magic number，每个知名的 fs 都各有定义，但显然我们没有
    pub f_type: i64,
    /// 最优传输块大小
    pub f_bsize: i64,
    /// 总的块数
    pub f_blocks: u64,
    /// 还剩多少块未分配
    pub f_bfree: u64,
    /// 对用户来说，还有多少块可用
    pub f_bavail: u64,
    /// 总的 inode 数
    pub f_files: u64,
    /// 空闲的 inode 数
    pub f_ffree: u64,
    /// 文件系统编号，但实际上对于不同的OS差异很大，所以不会特地去用
    pub f_fsid: [i32; 2],
    /// 文件名长度限制，这个OS默认FAT已经使用了加长命名
    pub f_namelen: isize,
    /// 片大小
    pub f_frsize: isize,
    /// 一些选项，但其实也没用到
    pub f_flags: isize,
    /// 空余 padding
    pub f_spare: [isize; 4],
}

/// 获取一个基础的fsstat
pub fn get_fs_stat() -> FsStat {
    FsStat {
        f_type: 0,
        f_bsize: 1024,
        f_blocks: 0x4000_0000 / 512,
        f_bfree: 1,
        f_bavail: 1,
        f_files: 1,
        f_ffree: 1,
        f_fsid: [0, 0],
        f_namelen: 256,
        f_frsize: 0x1000,
        f_flags: 0,
        f_spare: [0, 0, 0, 0],
    }
}

bitflags! {
    /// 指定 st_mode 的选项
    pub struct StMode: u32 {
        /// 是普通文件
        const S_IFREG = 1 << 15;
        /// 是目录
        const S_IFDIR = 1 << 14;
        /// 是字符设备
        const S_IFCHR = 1 << 13;
        /// 是否设置 uid/gid/sticky
        //const S_ISUID = 1 << 14;
        //const S_ISGID = 1 << 13;
        //const S_ISVTX = 1 << 12;
        /// 所有者权限
        const S_IXUSR = 1 << 10;
        const S_IWUSR = 1 << 9;
        const S_IRUSR = 1 << 8;
        /// 用户组权限
        const S_IXGRP = 1 << 6;
        const S_IWGRP = 1 << 5;
        const S_IRGRP = 1 << 4;
        /// 其他用户权限
        const S_IXOTH = 1 << 2;
        const S_IWOTH = 1 << 1;
        const S_IROTH = 1 << 0;
        /// 报告已执行结束的用户进程的状态
        const WIMTRACED = 1 << 1;
        /// 报告还未结束的用户进程的状态
        const WCONTINUED = 1 << 3;
    }
}
/// 文件类型，输入 IFCHR / IFDIR / IFREG 等具体类型，
/// 输出这些类型加上普遍的文件属性后得到的 mode 参数
pub fn normal_file_mode(file_type: StMode) -> StMode {
    file_type | StMode::S_IWUSR | StMode::S_IWUSR | StMode::S_IWGRP | StMode::S_IRGRP
}

/// prctl 中 PR_NAME_SIZE 要求的缓冲区长度
pub const PR_NAME_SIZE: usize = 16;

numeric_enum_macro::numeric_enum! {
    #[repr(usize)]
    #[allow(missing_docs)]
    #[allow(non_camel_case_types)]
    #[derive(Eq, PartialEq, Debug, Copy, Clone)]
    /// syscall_prctl的结构体
    pub enum PrctlOption {
        PR_SET_NAME = 15,
        PR_GET_NAME = 16,
    }
}