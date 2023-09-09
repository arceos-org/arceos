//! 定义epoll使用到的文件
extern crate alloc;
use alloc::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    vec::Vec,
};
use axerrno::{AxError, AxResult};

use axfs::api::{FileIO, FileIOType};
use axio::SeekFrom;

use axprocess::{current_process, yield_now_task};
use axsync::Mutex;

use crate::syscall::ErrorNo;

use super::flags::{EpollCtl, EpollEvent, EpollEventType};

pub struct EpollFile {
    /// 定义内部可变变量
    /// 由于存在clone，所以要用arc指针包围
    pub inner: Arc<Mutex<EpollFileInner>>,
}

pub struct EpollFileInner {
    /// 监控的所有事件，通过map来进行映射，根据fd找到对应的event
    monitor_list: BTreeMap<i32, EpollEvent>,
    /// 响应的事件集
    _response_list: BTreeSet<i32>,
}

impl EpollFile {
    /// 新建一个epoll文件
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(EpollFileInner {
                monitor_list: BTreeMap::new(),
                _response_list: BTreeSet::new(),
            })),
        }
    }

    /// 获取另外一份epoll文件，存储在fd manager中
    /// 这是对Arc的clone，即获取指针副本
    pub fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }

    /// 控制指定的事件，改变其对应的事件内容
    ///
    /// 成功返回0，错误返回对应的编号
    pub fn epoll_ctl(&self, op: EpollCtl, fd: i32, event: EpollEvent) -> isize {
        let mut inner = self.inner.lock();
        match op {
            // 添加事件
            EpollCtl::ADD => {
                if inner.monitor_list.contains_key(&fd) {
                    return ErrorNo::EEXIST as isize;
                }
                inner.monitor_list.insert(fd, event);
            }
            // 删除事件
            EpollCtl::DEL => {
                if !inner.monitor_list.contains_key(&fd) {
                    return ErrorNo::ENOENT as isize;
                }
                inner.monitor_list.remove(&fd);
            }
            // 修改对应事件
            EpollCtl::MOD => {
                // 对于不存在的事件，返回错误
                // 即modify要求原先文件存在对应事件，才能进行“修改”
                if !inner.monitor_list.contains_key(&fd) {
                    return ErrorNo::ENOENT as isize;
                }
                inner.monitor_list.insert(fd, event);
            }
        }
        0
    }

    /// 获取list中所有的epoll事件
    pub fn get_events(&self) -> Vec<EpollEvent> {
        let mut ans = Vec::new();
        for (fd, event) in self.inner.lock().monitor_list.iter() {
            let mut nevent = *event;
            if *fd as u64 != nevent.data {
                nevent.data = *fd as u64;
            }
            ans.push(nevent);
        }
        ans
    }

    /// 实现epoll wait，在规定超时时间内收集达到触发条件的事件
    ///
    /// 实现原理和ppoll很像
    pub fn epoll_wait(&self, expire_time: usize) -> AxResult<Vec<EpollEvent>> {
        let events = self.get_events();
        let mut ret_events = Vec::new();
        loop {
            let current_process = current_process();
            let fd_table = current_process.fd_manager.fd_table.lock();
            for req_event in events.iter() {
                if let Some(file) = &fd_table[req_event.data as usize] {
                    let mut ret_event_type = EpollEventType::empty();
                    if file.is_hang_up() {
                        ret_event_type |= EpollEventType::EPOLLHUP;
                    }
                    if file.in_exceptional_conditions() {
                        ret_event_type |= EpollEventType::EPOLLERR;
                    }
                    if file.ready_to_read()
                        && req_event.event_type.contains(EpollEventType::EPOLLIN)
                    {
                        ret_event_type |= EpollEventType::EPOLLIN;
                    }
                    if file.ready_to_write()
                        && req_event.event_type.contains(EpollEventType::EPOLLOUT)
                    {
                        ret_event_type |= EpollEventType::EPOLLOUT;
                    }
                    if !ret_event_type.is_empty() {
                        let mut ret_event = *req_event;
                        ret_event.event_type = ret_event_type;
                        ret_events.push(ret_event);
                    }
                    // 若文件存在但未响应，此时不加入到ret中，并以此作为是否终止的条件
                } else {
                    // 若文件不存在，认为不存在也是一种响应，所以要加入到ret中，并以此作为是否终止的条件
                    ret_events.push(EpollEvent {
                        event_type: EpollEventType::EPOLLERR,
                        data: req_event.data,
                    });
                }
            }
            if !ret_events.is_empty() {
                // 此时收到了响应，直接返回
                return Ok(ret_events);
            }
            // 否则直接block
            if riscv::register::time::read() > expire_time {
                return Ok(ret_events);
            }
            yield_now_task();
            if current_process.have_signals().is_some() {
                return Err(AxError::Timeout);
            }
        }
    }
}

/// EpollFile也是一种文件，应当为其实现一个file io trait
impl FileIO for EpollFile {
    fn read(&self, _buf: &mut [u8]) -> AxResult<usize> {
        Err(AxError::Unsupported)
    }
    fn write(&self, _buf: &[u8]) -> AxResult<usize> {
        Err(AxError::Unsupported)
    }
    fn flush(&self) -> AxResult {
        Err(AxError::Unsupported)
    }
    fn seek(&self, _pos: SeekFrom) -> AxResult<u64> {
        Err(AxError::Unsupported)
    }
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        false
    }
    fn executable(&self) -> bool {
        false
    }
    /// epoll file也是一个文件描述符
    fn get_type(&self) -> FileIOType {
        FileIOType::FileDesc
    }
    fn ready_to_read(&self) -> bool {
        // 如果当前epoll事件确实正在等待事件响应，那么可以认为事件准备好read，尽管无法读到实际内容
        let events = self.get_events();
        let process = current_process();
        let fd_table = process.fd_manager.fd_table.lock();
        for req_event in events.iter() {
            if let Some(file) = fd_table[req_event.data as usize].as_ref() {
                let mut ret_event_type = EpollEventType::empty();
                if file.is_hang_up() {
                    ret_event_type |= EpollEventType::EPOLLHUP;
                }
                if file.in_exceptional_conditions() {
                    ret_event_type |= EpollEventType::EPOLLERR;
                }
                if file.ready_to_read() && req_event.event_type.contains(EpollEventType::EPOLLIN) {
                    ret_event_type |= EpollEventType::EPOLLIN;
                }
                if file.ready_to_write() && req_event.event_type.contains(EpollEventType::EPOLLOUT)
                {
                    ret_event_type |= EpollEventType::EPOLLOUT;
                }
                if !ret_event_type.is_empty() {
                    return true;
                }
            }
        }
        false
    }
}
