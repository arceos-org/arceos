use core::ffi::c_char;

use arceos_posix_api::{ctypes::stat, utils::char_ptr_to_str};
use axerrno::AxError;
use log::info;

use crate::err;

#[unsafe(no_mangle)]
pub fn sys_stat(name: *const c_char, stat: *mut stat) -> i32 {
    info!("called sys_stat");
    unsafe { arceos_posix_api::sys_stat(name, stat as _) }
}

#[unsafe(no_mangle)]
pub fn sys_lstat(name: *const c_char, stat: *mut stat) -> i32 {
    info!("called sys_lstat");
    unsafe { arceos_posix_api::sys_lstat(name, stat as _) as _ }
}

#[unsafe(no_mangle)]
pub fn sys_fstat(fd: i32, stat: *mut stat) -> i32 {
    info!("called sys_fstat");
    unsafe { arceos_posix_api::sys_fstat(fd, stat as _) as _ }
}

#[unsafe(no_mangle)]
pub fn sys_unlink(name: *const c_char) -> i32 {
    info!("called sys_unlink");
    // get name as &str
    let name = match char_ptr_to_str(name) {
        Ok(s) => s,
        Err(e) => return err(e),
    };

    if let Err(e) = arceos_api::fs::ax_remove_file(name) {
        if e != AxError::IsADirectory {
            return err(e.into());
        }
        if let Err(e) = arceos_api::fs::ax_remove_dir(name) {
            return err(e.into());
        }
    }
    0
}

#[unsafe(no_mangle)]
pub fn sys_mkdir(name: *const c_char, _mode: u32) -> i32 {
    let name = match char_ptr_to_str(name) {
        Ok(s) => s,
        Err(e) => return err(e),
    };
    if let Err(e) = arceos_api::fs::ax_create_dir(name) {
        return err(e.into());
    }
    0
}
