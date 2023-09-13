use bitflags::bitflags;

bitflags! {
    /// 定义epoll事件的类别
    #[derive(Clone, Copy,Debug)]
    pub struct EpollEventType: u32{
        const EPOLLIN = 0x001;
        const EPOLLOUT = 0x004;
        const EPOLLERR = 0x008;
        const EPOLLHUP = 0x010;
        const EPOLLPRI = 0x002;
        const EPOLLRDNORM = 0x040;
        const EPOLLRDBAND = 0x080;
        const EPOLLWRNORM = 0x100;
        const EPOLLWRBAND= 0x200;
        const EPOLLMSG = 0x400;
        const EPOLLRDHUP = 0x2000;
        const EPOLLEXCLUSIVE = 0x1000_0000;
        const EPOLLWAKEUP = 0x2000_0000;
        const EPOLLONESHOT = 0x4000_0000;
        const EPOLLET = 0x8000_0000;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// 定义一个epoll事件
pub struct EpollEvent {
    /// 事件类型
    pub event_type: EpollEventType,
    /// 事件中使用到的数据，如fd等
    pub data: u64,
}

numeric_enum_macro::numeric_enum! {
    #[repr(i32)]
    #[derive(Clone, Copy, Debug)]
    pub enum EpollCtl {
        /// 添加一个文件对应的事件
        ADD = 1,
        /// 删除一个文件对应的事件
        DEL = 2,
        /// 修改一个文件对应的事件
        MOD = 3,
    }
}
