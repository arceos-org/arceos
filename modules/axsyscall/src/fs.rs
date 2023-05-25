use alloc::format;
use alloc::string::ToString;
use alloc::sync::Arc;
use core::mem::transmute;
use core::ptr::copy_nonoverlapping;
use log::debug;
use axfs::api;
use axfs_os::{FilePath, new_fd, new_dir, DirEnt, DirEntType};
use axfs_os::flags::OpenFlags;
use axfs_os::link::{create_link, remove_link};
use axfs_os::mount::{check_mounted, mount_fat_fs, umount_fat_fs};
use axfs_os::pipe::make_pipe;
use axfs_os::types::Kstat;
use axprocess::process::current_process;


#[allow(unused)]
const AT_FDCWD: usize = -100isize as usize;
// Special value used to indicate openat should use the current working directory.
const AT_REMOVEDIR: usize = 0x200;        // Remove directory instead of unlinking file.

// const STDIN: usize = 0;
// const STDOUT: usize = 1;
// const STDERR: usize = 2;

/// 辅助函数：处理路径，返回一个FilePath结构体
///
/// 输入：
///    - dir_fd：文件描述符
///    - path_addr：路径地址
///    - force_dir：是否强制为目录
///
/// 一般情况下, 传入path末尾是`/`的话, 生成的FilePath是一个目录，否则是一个文件；但如果force_dir为true, 则生成的FilePath一定是一个目录(自动补充`/`)
fn deal_with_path(dir_fd: usize, path_addr: Option<*const u8>, force_dir: bool) -> Option<FilePath> {
    let process = current_process();
    let process_inner = process.inner.lock();
    let mut path = "".to_string();
    if let Some(path_addr) = path_addr {
        if path_addr as usize == 0 {
            debug!("path address is null");
            return None;
        }
        path = process_inner.memory_set.lock().translate_str(path_addr);
    }

    if force_dir {
        path = format!("{}/", path);
    }
    if path.ends_with('.') {     // 如果path以.或..结尾, 则加上/告诉FilePath::new它是一个目录
        path = format!("{}/", path);
    }
    debug!("path: {}", path);

    if !path.starts_with('/') && dir_fd != AT_FDCWD {   // 如果不是绝对路径, 且dir_fd不是AT_FDCWD, 则需要将dir_fd和path拼接起来
        if dir_fd >= process_inner.fd_table.len() {
            debug!("fd index out of range");
            return None;
        }
        match process_inner.fd_table[dir_fd].as_ref() {
            Some(dir) => {
                if dir.get_type() != "DirDesc" {
                    debug!("selected fd is not a dir");
                    return None;
                }
                let dir = dir.clone();
                path = format!("{}/{}", dir.get_path(), path);
                debug!("handled_path: {}", path);
            }
            None => {
                debug!("fd not exist");
                return None;
            }
        }
    }

    Some(FilePath::new(&path))
}


/// 功能：从一个文件描述符中读取；
/// 输入：
///     - fd：要读取文件的文件描述符。
///     - buf：一个缓存区，用于存放读取的内容。
///     - count：要读取的字节数。
/// 返回值：成功执行，返回读取的字节数。如为0，表示文件结束。错误，则返回-1。
pub fn syscall_read(fd: usize, buf: *mut u8, count: usize) -> isize {
    debug!("Into syscall_read. fd: {}, buf: {:?}, len: {}", fd, buf as usize, count);
    let process = current_process();
    let process_inner = process.inner.lock();
    if fd >= process_inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = process_inner.fd_table[fd].as_ref() {
        if file.get_type() == "DirDesc" {
            debug!("fd is a dir");
            return -1;
        }
        if !file.readable() {
            return -1;
        }
        let file = file.clone();

        // // debug
        // file.print_content();

        drop(process_inner); // release current inner manually to avoid multi-borrow
        let read_size = file.read(unsafe { core::slice::from_raw_parts_mut(buf, count) })
            .unwrap() as isize;
        debug!("read_size: {}", read_size);
        read_size as isize
    } else {
        -1
    }
}


/// 功能：从一个文件描述符中写入；
/// 输入：
///     - fd：要写入文件的文件描述符。
///     - buf：一个缓存区，用于存放要写入的内容。
///     - count：要写入的字节数。
/// 返回值：成功执行，返回写入的字节数。错误，则返回-1。
pub fn syscall_write(fd: usize, buf: *const u8, count: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner.lock();
    if fd >= process_inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = process_inner.fd_table[fd].as_ref() {
        if file.get_type() == "DirDesc" {
            debug!("fd is a dir");
            return -1;
        }
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        drop(process_inner); // release current inner manually to avoid multi-borrow
        // file.write("Test SysWrite\n".as_bytes()).unwrap();
        file.write(unsafe { core::slice::from_raw_parts(buf, count) })
            .unwrap() as isize
    } else {
        -1
    }
}

/// 功能：打开或创建一个文件；
/// 输入：
///     - fd：文件所在目录的文件描述符。
///     - filename：要打开或创建的文件名。如为绝对路径，则忽略fd。如为相对路径，且fd是AT_FDCWD，则filename是相对于当前工作目录来说的。如为相对路径，且fd是一个文件描述符，则filename是相对于fd所指向的目录来说的。
///     - flags：必须包含如下访问模式的其中一种：O_RDONLY，O_WRONLY，O_RDWR。还可以包含文件创建标志和文件状态标志。
///     - mode：文件的所有权描述。详见`man 7 inode `。
/// 返回值：成功执行，返回新的文件描述符。失败，返回-1。
///
/// 说明：如果打开的是一个目录，那么返回的文件描述符指向的是该目录的描述符。(后面会用到针对目录的文件描述符)
/// flags: O_RDONLY: 0, O_WRONLY: 1, O_RDWR: 2, O_CREAT: 64, O_DIRECTORY: 65536
pub fn syscall_openat(fd: usize, path: *const u8, flags: usize, _mode: u8) -> isize {
    let force_dir = OpenFlags::from(flags).is_dir();
    let path = deal_with_path(fd, Some(path), force_dir).unwrap();
    let process = current_process();
    let mut process_inner = process.inner.lock();
    let fd_num = process_inner.alloc_fd();
    debug!("allocated fd_num: {}", fd_num);
    // 如果是DIR
    if path.is_dir() {
        debug!("open dir");
        if let Ok(dir) = new_dir(path.path().to_string(), flags.into()) {
            debug!("new dir_desc successfully allocated: {}", path.path());
            process_inner.fd_table[fd_num] = Some(Arc::new(dir));
            fd_num as isize
        } else {
            debug!("open dir failed");
            -1
        }
    }
    // 如果是FILE，注意若创建了新文件，需要添加链接
    else {
        debug!("open file");
        if let Ok(file) = new_fd(path.path().to_string(), flags.into()) {
            debug!("new file_desc successfully allocated");
            process_inner.fd_table[fd_num] = Some(Arc::new(file));
            let _ = create_link(&path, &path);  // 不需要检查是否成功，因为如果成功，说明是新建的文件，如果失败，说明已经存在了
            fd_num as isize
        } else {
            debug!("open file failed");
            -1
        }
    }
}

/// 功能：关闭一个文件描述符；
/// 输入：
///     - fd：要关闭的文件描述符。
/// 返回值：成功执行，返回0。失败，返回-1。
pub fn syscall_close(fd: usize) -> isize {
    debug!("Into syscall_close. fd: {}", fd);

    let process = current_process();
    let mut process_inner = process.inner.lock();

    if fd >= process_inner.fd_table.len() {
        debug!("fd {} is out of range", fd);
        return -1;
    }
    // if fd == 3 {
    //     debug!("fd {} is reserved for cwd", fd);
    //     return -1;
    // }
    if process_inner.fd_table[fd].is_none() {
        debug!("fd {} is none", fd);
        return -1;
    }
    process_inner.fd_table[fd].take();

    // for i in 0..process_inner.fd_table.len() {
    //     if let Some(file) = process_inner.fd_table[i].as_ref() {
    //         debug!("fd: {} has file", i);
    //     }
    // }

    0
}

/// 功能：获取当前工作目录；
/// 输入：
///     - char *buf：一块缓存区，用于保存当前工作目录的字符串。当buf设为NULL，由系统来分配缓存区。
///     - size：buf缓存区的大小。
/// 返回值：成功执行，则返回当前工作目录的字符串的指针。失败，则返回NULL。
///  暂时：成功执行，则返回当前工作目录的字符串的指针 as isize。失败，则返回0。
pub fn syscall_getcwd(buf: *mut u8, len: usize) -> isize {
    debug!("Into syscall_getcwd. buf: {}, len: {}", buf as usize, len);
    let cwd = api::current_dir().unwrap();

    // todo: 如果buf为NULL,则系统分配缓存区
    // let process = current_process();
    // let process_inner = process.inner.lock();
    // if buf.is_null() {
    //     buf = allocate_buffer(cwd.len());   // 分配缓存区 allocate_buffer
    // }

    let cwd = cwd.as_bytes();

    return if len >= cwd.len() {
        unsafe {
            core::ptr::copy_nonoverlapping(cwd.as_ptr(), buf, cwd.len());
        }
        buf as isize
    } else {
        debug!("getcwd: buf size is too small");
        0
    };
}

/// 功能：创建管道；
/// 输入：
///     - fd[2]：用于保存2个文件描述符。其中，fd[0]为管道的读出端，fd[1]为管道的写入端。
/// 返回值：成功执行，返回0。失败，返回-1。
///
/// 注意：fd[2]是32位数组，所以这里的 fd 是 u32 类型的指针，而不是 usize 类型的指针。
pub fn syscall_pipe2(fd: *mut u32) -> isize {
    debug!("Into syscall_pipe2. fd: {}", fd as usize);
    let process = current_process();
    let mut process_inner = process.inner.lock();

    let (read, write) = make_pipe();

    let fd_num = process_inner.alloc_fd();
    process_inner.fd_table[fd_num] = Some(read);
    let fd_num2 = process_inner.alloc_fd();
    process_inner.fd_table[fd_num2] = Some(write);


    debug!("fd_num: {}, fd_num2: {}", fd_num, fd_num2);

    unsafe {
        core::ptr::write(fd, fd_num as u32);
        core::ptr::write(fd.offset(1), fd_num2 as u32);
    }
    0
}

/// 功能：复制文件描述符；
/// 输入：
///     - fd：被复制的文件描述符。
/// 返回值：成功执行，返回新的文件描述符。失败，返回-1。
pub fn syscall_dup(fd: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner.lock();

    if fd >= process_inner.fd_table.len() {
        debug!("fd {} is out of range", fd);
        return -1;
    }
    if process_inner.fd_table[fd].is_none() {
        debug!("fd {} is a closed fd", fd);
        return -1;
    }

    let fd_num = process_inner.alloc_fd();
    process_inner.fd_table[fd_num] = process_inner.fd_table[fd].clone();

    fd_num as isize
}

/// 功能：复制文件描述符，并指定了新的文件描述符；
/// 输入：
///     - old：被复制的文件描述符。
///     - new：新的文件描述符。
/// 返回值：成功执行，返回新的文件描述符。失败，返回-1。
pub fn syscall_dup3(fd: usize, new_fd: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner.lock();

    if fd >= process_inner.fd_table.len() {
        debug!("fd {} is out of range", fd);
        return -1;
    }
    if process_inner.fd_table[fd].is_none() {
        debug!("fd {} is not opened", fd);
        return -1;
    }
    if new_fd >= process_inner.fd_table.len() {
        for _i in process_inner.fd_table.len()..new_fd + 1 {
            process_inner.fd_table.push(None);
        }
    }
    if process_inner.fd_table[new_fd].is_some() {
        debug!("new_fd {} is already opened", new_fd);
        return -1;
    }
    process_inner.fd_table[new_fd] = process_inner.fd_table[fd].clone();

    new_fd as isize
}

/// 功能：创建目录；
/// 输入：
///     - dirfd：要创建的目录所在的目录的文件描述符。
///     - path：要创建的目录的名称。如果path是相对路径，则它是相对于dirfd目录而言的。如果path是相对路径，且dirfd的值为AT_FDCWD，则它是相对于当前路径而言的。如果path是绝对路径，则dirfd被忽略。
///     - mode：文件的所有权描述。详见`man 7 inode `。
/// 返回值：成功执行，返回0。失败，返回-1。
pub fn syscall_mkdirat(dir_fd: usize, path: *const u8, mode: u32) -> isize {
    let path = deal_with_path(dir_fd, Some(path), true).unwrap();
    debug!("Into syscall_mkdirat. dirfd: {}, path: {:?}, mode: {}", dir_fd, path.path(), mode);
    if let Ok(_res) = api::create_dir(path.path()) {
        0
    } else {
        -1
    }
}


/// 功能：切换工作目录；
/// 输入：
///     - path：需要切换到的目录。
/// 返回值：成功执行，返回0。失败，返回-1。
pub fn syscall_chdir(path: *const u8) -> isize {
    // 从path中读取字符串
    let path = deal_with_path(AT_FDCWD, Some(path), true).unwrap();
    debug!("Into syscall_chdir. path: {:?}", path.path());
    match api::set_current_dir(path.path()) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// 功能：获取目录的条目;
/// 参数：
///     -fd：所要读取目录的文件描述符。
///     -buf：一个缓存区，用于保存所读取目录的信息。缓存区的结构如下
///     -len：buf的大小。
/// 返回值：成功执行，返回读取的字节数。当到目录结尾，则返回0。失败，则返回-1。
///  struct dirent {
///      uint64 d_ino;	// 索引结点号
///      int64 d_off;	// 到下一个dirent的偏移
///      unsigned short d_reclen;	// 当前dirent的长度
///      unsigned char d_type;	// 文件类型 0:
///      char d_name[];	//文件名
///  };
///  1. 内存布局：
///       0x61fef8
///       0x61fef8 0x61ff00 0x61ff08 0x61ff0a 0x61ff0b
///       实测结果在我的电脑上是这样的，没有按最大对齐方式8字节对齐
///  2. d_off 和 d_reclen 同时存在的原因：
///       不同的dirent可以不按照顺序紧密排列
pub fn syscall_getdents64(fd: usize, buf: *mut u8, len: usize) -> isize {
    let path = deal_with_path(fd, None, true).unwrap();

    let buf = unsafe { core::slice::from_raw_parts_mut(buf, len) };
    let dir_iter = api::read_dir(path.path()).unwrap();
    let mut count = 0;     // buf中已经写入的字节数

    for (i, entry) in dir_iter.enumerate() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        let name = name.as_bytes();
        let name_len = name.len();
        let file_type = entry.file_type();
        let entry_size = DirEnt::fixed_size() + name_len + 1;

        // buf不够大，写不下新的entry
        if count + entry_size > len {
            debug!("buf not big enough");
            return count as isize;
        }
        // 转换为DirEnt
        let dirent: &mut DirEnt = unsafe { transmute(buf.as_mut_ptr().offset(count as isize)) };
        // 设置定长部分
        if file_type.is_dir() {
            dirent.set_fixed_part(i as u64, entry_size as i64, entry_size, DirEntType::DIR);
        } else if file_type.is_file() {
            dirent.set_fixed_part(i as u64, entry_size as i64, entry_size, DirEntType::REG);
        } else {
            dirent.set_fixed_part(i as u64, entry_size as i64, entry_size, DirEntType::UNKNOWN);
        }

        // 写入文件名
        unsafe { copy_nonoverlapping(name.as_ptr(), dirent.d_name.as_mut_ptr(), name_len + 1) };

        count += entry_size;
    }
    0
}

/// 功能：创建文件的链接；
/// 输入：
///     - old_dir_fd：原来的文件所在目录的文件描述符。
///     - old_path：文件原来的名字。如果old_path是相对路径，则它是相对于old_dir_fd目录而言的。如果old_path是相对路径，且old_dir_fd的值为AT_FDCWD，则它是相对于当前路径而言的。如果old_path是绝对路径，则old_dir_fd被忽略。
///     - new_dir_fd：新文件名所在的目录。
///     - new_path：文件的新名字。new_path的使用规则同old_path。
///     - flags：在2.6.18内核之前，应置为0。其它的值详见`man 2 linkat`。
/// 返回值：成功执行，返回0。失败，返回-1。
#[allow(dead_code)]
pub fn sys_linkat(old_dir_fd: usize, old_path: *const u8, new_dir_fd: usize, new_path: *const u8, _flags: usize) -> isize {
    let old_path = deal_with_path(old_dir_fd, Some(old_path), false).unwrap();
    let new_path = deal_with_path(new_dir_fd, Some(new_path), false).unwrap();
    if create_link(&old_path, &new_path) {
        0
    } else {
        -1
    }
}

/// 功能：移除指定文件的链接(可用于删除文件)；
/// 输入：
///     - dir_fd：要删除的链接所在的目录。
///     - path：要删除的链接的名字。如果path是相对路径，则它是相对于dir_fd目录而言的。如果path是相对路径，且dir_fd的值为AT_FDCWD，则它是相对于当前路径而言的。如果path是绝对路径，则dir_fd被忽略。
///     - flags：可设置为0或AT_REMOVEDIR。
/// 返回值：成功执行，返回0。失败，返回-1。
pub fn syscall_unlinkat(dir_fd: usize, path: *const u8, flags: usize) -> isize {
    let path = deal_with_path(dir_fd, Some(path), false).unwrap();

    // unlink file
    if flags == 0 {
        if let None = remove_link(&path) {
            debug!("unlink file error");
            return -1;
        }
    }
    // remove dir
    else if flags == AT_REMOVEDIR {
        if let Err(e) = api::remove_dir(path.path()) {
            debug!("rmdir error: {:?}", e);
            return -1;
        }
    }
    // flags error
    else {
        debug!("flags error");
        return -1;
    }
    0
}

/// 功能：挂载文件系统；
/// 输入：
///   - special: 挂载设备；
///   - dir: 挂载点；       经过实测，发现dir可以是绝对路径，也可以是相对路径，甚至可以是 . 或 ..
///   - fs_type: 挂载的文件系统类型；
///   - flags: 挂载参数；
///   - data: 传递给文件系统的字符串参数，可为NULL；
/// 返回值：成功返回0，失败返回-1
pub fn syscall_mount(
    special: *const u8,
    dir: *const u8,
    fs_type: *const u8,
    _flags: usize,
    _data: *const u8,
) -> isize {
    let device_path = deal_with_path(AT_FDCWD, Some(special), false).unwrap();
    // 这里dir必须以"/"结尾，但在shell中输入时，不需要以"/"结尾
    let mount_path = deal_with_path(AT_FDCWD, Some(dir), true).unwrap();
    let process = current_process();
    let process_inner = process.inner.lock();
    let memory_set = process_inner.memory_set.lock();
    let fs_type = memory_set.translate_str(fs_type);
    let mut _data_str = "".to_string();
    if !_data.is_null() {   // data可以为NULL, 必须判断, 否则会panic, 发生LoadPageFault
        _data_str = memory_set.translate_str(_data);
    }
    if device_path.is_dir() {
        debug!("device_path should not be a dir");
        return -1;
    }
    if !mount_path.is_dir() {
        debug!("mount_path should be a dir");
        return -1;
    }
    if fs_type != "vfat" {
        debug!("fs_type can only be vfat.");
        return -1;
    }
    // 检查挂载点路径是否存在
    if !api::path_exists(mount_path.path()) {
        debug!("mount path not exist");
        return -1;
    }
    // 查挂载点是否已经被挂载
    if check_mounted(&mount_path) {
        debug!("mount path includes mounted fs");
        return -1;
    }
    // 挂载
    if !mount_fat_fs(&device_path, &mount_path) {
        debug!("mount error");
        return -1;
    }

    0
}


/// 功能：卸载文件系统；
/// 输入：指定卸载目录，卸载参数；
/// 返回值：成功返回0，失败返回-1；
pub fn syscall_umount(dir: *const u8, flags: usize) -> isize {
    let mount_path = deal_with_path(AT_FDCWD, Some(dir), true).unwrap();

    if flags != 0 {
        debug!("flags unimplemented");
        return -1;
    }

    // 检查挂载点路径是否存在
    if !api::path_exists(mount_path.path()) {
        debug!("mount path not exist");
        return -1;
    }
    // 从挂载点中删除
    if !umount_fat_fs(&mount_path) {
        debug!("umount error");
        return -1;
    }

    0
}


/// 功能：获取文件状态；
/// 输入：
///     - fd: 文件句柄；
///     - kst: 接收保存文件状态的指针；
/// 返回值：成功返回0，失败返回-1；
/// struct kstat {
/// 	dev_t st_dev;
/// 	ino_t st_ino;
/// 	mode_t st_mode;
/// 	nlink_t st_nlink;
/// 	uid_t st_uid;
/// 	gid_t st_gid;
/// 	dev_t st_rdev;
/// 	unsigned long __pad;
/// 	off_t st_size;
/// 	blksize_t st_blksize;
/// 	int __pad2;
/// 	blkcnt_t st_blocks;
/// 	long st_atime_sec;
/// 	long st_atime_nsec;
/// 	long st_mtime_sec;
/// 	long st_mtime_nsec;
/// 	long st_ctime_sec;
/// 	long st_ctime_nsec;
/// 	unsigned __unused[2];
/// };
pub fn syscall_fstat(fd: usize, kst: *mut Kstat) -> isize {
    let process = current_process();
    let process_inner = process.inner.lock();

    if fd >= process_inner.fd_table.len() || fd < 3 {
        debug!("fd {} is out of range", fd);
        return -1;
    }
    if process_inner.fd_table[fd].is_none() {
        debug!("fd {} is none", fd);
        return -1;
    }
    let file = process_inner.fd_table[fd].clone().unwrap();
    if file.get_type() != "FileDesc" {
        debug!("fd {} is not a file", fd);
        return -1;
    }

    match file.get_stat() {
        Ok(stat) => {
            let kstat = unsafe { &mut *kst };
            kstat.st_dev = stat.st_dev;
            kstat.st_ino = stat.st_ino;
            kstat.st_mode = stat.st_mode;
            kstat.st_nlink = stat.st_nlink;
            kstat.st_uid = stat.st_uid;
            kstat.st_gid = stat.st_gid;
            kstat.st_rdev = stat.st_rdev;
            kstat.st_size = stat.st_size;
            kstat.st_blksize = stat.st_blksize;
            kstat.st_blocks = stat.st_blocks;
            kstat.st_atime_sec = stat.st_atime_sec;
            kstat.st_atime_nsec = stat.st_atime_nsec;
            kstat.st_mtime_sec = stat.st_mtime_sec;
            kstat.st_mtime_nsec = stat.st_mtime_nsec;
            kstat.st_ctime_sec = stat.st_ctime_sec;
            kstat.st_ctime_nsec = stat.st_ctime_nsec;
            0
        }
        Err(e) => {
            debug!("get stat error: {:?}", e);
            -1
        }
    }
}

// // ## sys_renameat2()
// //
// // sys_renameat2()是Linux内核中的一个系统调用,用于重命名文件或目录。与renameat()系统调用不同,它允许指定标志位来控制文件重命名的行为。它的原型如下:
// //
// // ```c
// // int sys_renameat2(int olddirfd, const char *oldpath, int newdirfd, const char *newpath, unsigned int flags)
// // ```
// //
// // \- olddirfd: 源文件路径名的目录文件描述符
// // \- oldpath: 源文件路径名
// // \- newdirfd: 目标文件路径名的目录文件描述符
// // \- newpath: 目标文件路径名
// // \- flags: 标志位,控制重命名行为flags可以设置以下值:- RENAME_NOREPLACE: 重命名失败,如果目标文件已经存在
// // \- RENAME_EXCHANGE: 重命名交换两个文件
// // \- RENAME_WHITEOUT: 创建目标文件并插入whiteout标记如果newpath已经存在,并且未指定RENAME_NOREPLACE标志,则新文件会替换旧文件。成功重命名文件后,sys_renameat2()返回0。失败时返回-1,并设置错误码。举个例子:
// //
// // ```c
// // //重命名/tmp/file1为/tmp/file2,如果file2已存在则失败
// // sys_renameat2(AT_FDCWD, "/tmp/file1", AT_FDCWD, "/tmp/file2", RENAME_NOREPLACE);
// //
// // //交换/tmp/file1和/tmp/file2
// // sys_renameat2(AT_FDCWD, "/tmp/file1", AT_FDCWD, "/tmp/file2", RENAME_EXCHANGE);
// // ```
// //
// // 这个示例演示了如何使用RENAME_NOREPLACE防止目标文件被替换,和使用RENAME_EXCHANGE交换两个文件的重命名。sys_renameat2()提供了更加灵活的文件重命名机制,通过标志位可以实现只有当目标文件不存在时才重命名、交换两个文件的重命名等功能。
// pub fn syscall_renameat2(old_dirfd: usize, old_path: *const u8, new_dirfd: usize, new_path: *const u8, flags: usize) -> isize {
//     let process = current_process();
//     let mut process_inner = process.inner.lock();
//
//     let old_path = process_inner.memory_set.lock().translate_str(old_path);
//     let new_path = process_inner.memory_set.lock().translate_str(new_path);
//
//     // 处理path
//     let mut old_path_ = "".to_string();
//     if !old_path.starts_with('/') {
//         if old_dirfd == AT_FDCWD {
//             old_path_ = api::canonicalize(old_path.as_str()).unwrap();
//         }else{
//             if old_dirfd >= process_inner.fd_table.len() || old_dirfd < 0 {
//                 debug!("old_dirfd index out of range");
//                 return -1;
//             }
//             if let Some(dir) = process_inner.fd_table[old_dirfd].as_ref() {
//                 let dir = dir.clone();
//                 old_path_ = format!("{}/{}", dir.get_path(), old_path);
//             } else {
//                 debug!("old_dirfd not exist");
//                 return -1;
//             }
//         }
//     }
//     let mut new_path_ = "".to_string();
//     if !new_path.starts_with('/') {
//         if new_dirfd == AT_FDCWD {
//             new_path_ = api::canonicalize(new_path.as_str()).unwrap();
//         }else{
//             if new_dirfd >= process_inner.fd_table.len() || new_dirfd < 0 {
//                 debug!("old_dirfd index out of range");
//                 return -1;
//             }
//             if let Some(dir) = process_inner.fd_table[new_dirfd].as_ref() {
//                 let dir = dir.clone();
//                 new_path_ = format!("{}/{}", dir.get_path(), new_path);
//             } else {
//                 debug!("old_dirfd not exist");
//                 return -1;
//             }
//         }
//     }
//
//     if flags == RENAME_NOREPLACE {
//         if api::metadata(new_path_.as_str()).is_ok() {
//             debug!("new_path_ already exist");
//             return -1;
//         }
//     }
//
//     if flags == RENAME_EXCHANGE {
//         let old_metadata = api::metadata(old_path_.as_str()).unwrap();
//         let new_metadata = api::metadata(new_path_.as_str()).unwrap();
//         if old_metadata.is_dir() != new_metadata.is_dir() {
//             debug!("old_path_ and new_path_ is not the same type");
//             return -1;
//         }
//     }
//
//     if flags == RENAME_WHITEOUT {
//         let new_metadata = api::metadata(new_path_.as_str()).unwrap();
//         if new_metadata.is_dir() {
//             debug!("new_path_ is a directory");
//             return -1;
//         }
//     }
//
//     if flags != RENAME_NOREPLACE && flags != RENAME_EXCHANGE && flags != RENAME_WHITEOUT {
//         debug!("flags is not valid");
//         return -1;
//     }
//
//     if api::rename(old_path_.as_str(), new_path_.as_str()).is_err() {
//         debug!("rename failed");
//         return -1;
//     }
//
//     0
// }