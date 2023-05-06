use super::file_io::FileIO;
use crate::flags::OpenFlags;
use alloc::sync::Arc;
use alloc::vec::Vec;
use axerrno::{AxError, AxResult};
use axfs::api::{File, OpenOptions};
use axio::{Read, Seek, SeekFrom, Write};
use axsync::Mutex;
/// 文件描述符
pub struct FileDesc {
    /// 文件
    file: Arc<Mutex<File>>,
    /// 文件打开的标志位, 由低到高: read, write, append, truncate, create, create_new, 0, 0
    flags: OpenFlags,
}

/// 为FileDesc实现FileIO trait
impl FileIO for FileDesc {
    fn readable(&self) -> bool {
        self.flags.readable()
    }

    fn writable(&self) -> bool {
        self.flags.writable()
    }

    fn read(&self, buf: &mut [u8]) -> AxResult<usize> {
        self.file.lock().read(buf)
    }

    fn write(&self, buf: &[u8]) -> AxResult<usize> {
        self.file.lock().write(buf)
    }

    fn seek(&self, offset: usize) -> AxResult<u64> {
        self.file.lock().seek(SeekFrom::Start(offset as u64))
    }
    // fn flush(&self) {
    //     unsafe {
    //         (*self.file.get()).flush()
    //     }
    // }
    // fn set_len(&self, len: usize) {
    //     unsafe {
    //         (*self.file.get()).set_len(len)
    //     }
    // }
    // fn sync_all(&self) {
    //     unsafe {
    //         (*self.file.get()).sync_all()
    //     }
    // }
    // fn sync_data(&self) {
    //     unsafe {
    //         (*self.file.get()).sync_data()
    //     }
    // }
}

/// 文件描述符的实现
impl FileDesc {
    /// 创建一个新的文件描述符
    pub fn new(file: Arc<Mutex<File>>, flags: u8) -> Self {
        Self {
            file,
            flags: OpenFlags::from_bits(flags as u32).unwrap(),
        }
    }
    // /// 从文件描述符对应的文件进行读取，指定了开始的偏移量与读取的长度
    // pub fn read_file(&self, offset: usize, len: usize) -> AxResult<Vec<u8>, AxError> {
    //     let mut buf = [0u8; len];
    //     self.file
    //         .lock()
    //         .seek(SeekFrom::Start(offset as u64))
    //         .unwrap();
    //     self.file.lock().read_exact(&mut buf)?;
    //     Ok(buf.to_vec())
    // }
}

/// 新建一个文件描述符
pub fn new_fd(path: &str, flags: u8) -> AxResult<FileDesc> {
    let file = File::options()
        .read(flags & 0b0000_0001 != 0)
        .write(flags & 0b0000_0010 != 0)
        .append(flags & 0b0000_0100 != 0)
        .truncate(flags & 0b0000_1000 != 0)
        .create(flags & 0b0001_0000 != 0)
        .create_new(flags & 0b0010_0000 != 0)
        .open(path)?;
    // let file_size = file.metadata()?.len();
    let fd = FileDesc::new(Arc::new(Mutex::new(file)), flags);
    Ok(fd)
}

// /// 文件描述符表
// struct FileTable {
//     table: Vec<FileDesc>,
// }
//
// /// 文件描述符表的实现
// impl FileTable {
//     /// 创建一个新的文件描述符表
//     pub fn new() -> Self {
//         let res = Self {
//             table: Vec::new(),
//         };
//         // 将标准输入输出错误流添加到文件描述符表中
//         // 由于标准输入输出错误流是在内核中创建的, 因此不需要释放
//         table.push(FileDesc {
//             file: axfs::File::new(0),
//             flags: 0b0000_0001,
//         });
//         table.push(FileDesc {
//             file: axfs::File::new(1),
//             flags: 0b0000_0010,
//         });
//         table.push(FileDesc {
//             file: axfs::File::new(2),
//             flags: 0b0000_0010,
//         });
//         res
//     }
//
//     /// 为文件描述符表添加一个文件描述符
//     pub fn add(&mut self, file: fs::File, flags: u8) -> usize {
//         let fd = FileDesc { file, flags };
//         self.table.push(fd);
//         self.table.len() - 1
//     }
//
//     /// 通过文件描述符获取文件
//     pub fn get(&self, fd: usize) -> Option<&fs::File> {
//         self.table.get(fd).map(|fd| &fd.file)
//     }
//
//     /// 通过文件描述符获取文件的可变引用
//     pub fn get_mut(&mut self, fd: usize) -> Option<&mut fs::File> {
//         self.table.get_mut(fd).map(|fd| &mut fd.file)
//     }
//
//     /// 通过文件描述符获取文件的标志位
//     pub fn get_flags(&self, fd: usize) -> Option<u8> {
//         self.table.get(fd).map(|fd| fd.flags)
//     }
//
//     /// 通过文件描述符获取文件的标志位的可变引用
//     pub fn get_flags_mut(&mut self, fd: usize) -> Option<&mut u8> {
//         self.table.get_mut(fd).map(|fd| &mut fd.flags)
//     }
//
//     /// 通过文件描述符删除文件
//     pub fn remove(&mut self, fd: usize) -> Option<fs::File> {
//         self.table.remove(fd).map(|fd| fd.file)
//     }
//
//     /// 获取标准输入流
//     fn get_stdin(&self) -> &File {
//         let fd = 0;
//         &self.table[fd].file
//     }
//
//     /// 获取标准输出流
//     fn get_stdout(&self) -> &File {
//         let fd = 1;
//         &self.table[fd].file
//     }
//
//
// }
