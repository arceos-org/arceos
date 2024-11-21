#![cfg(not(feature = "myfs"))]

mod test_common;

use axdriver::AxDeviceContainer;
use axdriver_block::ramdisk::RamDisk;

const IMG_PATH: &str = "resources/fat16.img";

fn make_disk() -> std::io::Result<RamDisk> {
    let path = std::env::current_dir()?.join(IMG_PATH);
    println!("Loading disk image from {:?} ...", path);
    let data = std::fs::read(path)?;
    println!("size = {} bytes", data.len());
    Ok(RamDisk::from(&data))
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
fn test_fatfs() {
    println!("Testing fatfs with ramdisk ...");
    axns_imp::thread_init_namespace();
    let disk = make_disk().expect("failed to load disk image");
    axtask::init_scheduler(); // call this to use `axsync::Mutex`.
    axfs::init_filesystems(AxDeviceContainer::from_one(disk));

    test_common::test_all();
}
