#![cfg(feature = "myfs")]

mod test_common;

use std::sync::Arc;

use axdriver::AxDeviceContainer;
use axdriver_block::ramdisk::RamDisk;
use axfs::api::{self as fs, File};
use axfs::fops::{Disk, MyFileSystemIf};
use axfs_ramfs::RamFileSystem;
use axfs_vfs::VfsOps;
use axio::{Result, Write};

struct MyFileSystemIfImpl;

#[crate_interface::impl_interface]
impl MyFileSystemIf for MyFileSystemIfImpl {
    fn new_myfs(_disk: Disk) -> Arc<dyn VfsOps> {
        Arc::new(RamFileSystem::new())
    }
}

fn create_init_files() -> Result<()> {
    fs::write("./short.txt", "Rust is cool!\n")?;
    let mut file = File::create_new("/long.txt")?;
    for _ in 0..100 {
        file.write_fmt(format_args!("Rust is cool!\n"))?;
    }

    fs::create_dir("very-long-dir-name")?;
    fs::write(
        "very-long-dir-name/very-long-file-name.txt",
        "Rust is cool!\n",
    )?;

    fs::create_dir("very")?;
    fs::create_dir("//very/long")?;
    fs::create_dir("/./very/long/path")?;
    fs::write(".//very/long/path/test.txt", "Rust is cool!\n")?;
    Ok(())
}

mod axns_imp {
    use axns::{AxNamespace, AxNamespaceIf};
    use lazyinit::LazyInit;

    thread_local! {
        static NS: LazyInit<AxNamespace> = LazyInit::new();
    }
    struct AxNamespaceImpl;

    #[crate_interface::impl_interface]
    impl AxNamespaceIf for AxNamespaceImpl {
        fn current_namespace_base() -> *mut u8 {
            NS.with(|ns| ns.base())
        }
    }

    pub(crate) fn thread_init_namespace() {
        NS.with(|ns| {
            ns.init_once(AxNamespace::global());
        });
    }
}

#[test]
fn test_ramfs() {
    println!("Testing ramfs ...");
    axns_imp::thread_init_namespace();
    axtask::init_scheduler(); // call this to use `axsync::Mutex`.
    axfs::init_filesystems(AxDeviceContainer::from_one(RamDisk::default())); // dummy disk, actually not used.

    if let Err(e) = create_init_files() {
        log::warn!("failed to create init files: {:?}", e);
    }

    test_common::test_all();
}
