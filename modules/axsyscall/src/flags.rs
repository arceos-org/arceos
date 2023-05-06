use axhal::paging::MappingFlags;
use bitflags::*;

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
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

// sys_nanosleep指定的结构体类型
#[derive(Clone, Copy)]
pub struct TimeSecs {
    pub tv_sec: usize,
    pub tv_nsec: usize,
}

bitflags! {
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
