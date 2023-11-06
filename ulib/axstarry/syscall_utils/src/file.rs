use axerrno::AxResult;
use axfs::api::{File, OpenFlags};

/// 若使用多次new file打开同名文件，那么不同new file之间读写指针不共享，但是修改的内容是共享的
pub fn new_file(path: &str, flags: &OpenFlags) -> AxResult<File> {
    let mut file = File::options();
    file.read(flags.readable());
    file.write(flags.writable());
    file.create(flags.creatable());
    file.create_new(flags.new_creatable());
    file.open(path)
}
/// 在完成一次系统调用之后，恢复全局目录
pub fn init_current_dir() {
    axfs::api::set_current_dir("/").expect("reset current dir failed");
}

pub type FileFlags = OpenFlags;
