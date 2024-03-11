//! Init some files and links for the apps

use alloc::{format, string::ToString};
use axstarry::{create_link, new_file, FileFlags, FilePath};

/// 在执行系统调用前初始化文件系统
///
/// 包括建立软连接，提前准备好一系列的文件与文件夹
///
/// Fat32 filesystem doesn't exists the concept of soft link, so we need to call this function every time we boot the system
pub fn fs_init() {
    #[cfg(target_arch = "riscv64")]
    let libc_so = &"ld-musl-riscv64-sf.so.1";
    #[cfg(target_arch = "riscv64")]
    let libc_so2 = &"ld-musl-riscv64.so.1"; // 另一种名字的 libc.so，非 libc-test 测例库用

    #[cfg(target_arch = "x86_64")]
    let libc_so = &"ld-musl-x86_64-sf.so.1";
    #[cfg(target_arch = "x86_64")]
    let libc_so2 = &"ld-musl-x86_64.so.1"; // 另一种名字的 libc.so，非 libc-test 测例库用

    #[cfg(target_arch = "aarch64")]
    let libc_so = &"ld-musl-aarch64-sf.so.1";
    #[cfg(target_arch = "aarch64")]
    let libc_so2 = &"ld-musl-aarch64.so.1"; // 另一种名字的 libc.so，非 libc-test 测例库用

    create_link(
        &(FilePath::new(("/lib/".to_string() + libc_so).as_str()).unwrap()),
        &(FilePath::new("libc.so").unwrap()),
    );
    create_link(
        &(FilePath::new(("/lib/".to_string() + libc_so2).as_str()).unwrap()),
        &(FilePath::new("libc.so").unwrap()),
    );

    let tls_so = &"tls_get_new-dtv_dso.so";
    create_link(
        &(FilePath::new(("/lib/".to_string() + tls_so).as_str()).unwrap()),
        &(FilePath::new("tls_get_new-dtv_dso.so").unwrap()),
    );

    // 接下来对 busybox 相关的指令建立软链接
    let busybox_arch = ["ls", "mkdir", "touch", "mv", "busybox"];
    for arch in busybox_arch {
        let src_path = "/usr/sbin/".to_string() + arch;
        create_link(
            &(FilePath::new(src_path.as_str()).unwrap()),
            &(FilePath::new("busybox").unwrap()),
        );
        let src_path = "/usr/bin/".to_string() + arch;
        create_link(
            &(FilePath::new(src_path.as_str()).unwrap()),
            &(FilePath::new("busybox").unwrap()),
        );
    }
    create_link(
        &(FilePath::new("/bin/lmbench_all").unwrap()),
        &(FilePath::new("/lmbench_all").unwrap()),
    );
    create_link(
        &(FilePath::new("/bin/iozone").unwrap()),
        &(FilePath::new("/iozone").unwrap()),
    );

    #[cfg(target_arch = "x86_64")]
    {
        let libc_zlm = &"/lib64/ld-linux-x86-64.so.2";
        create_link(
            &(FilePath::new(libc_zlm).unwrap()),
            &(FilePath::new("ld-linux-x86-64.so.2").unwrap()),
        );

        create_link(
            &(FilePath::new("/lib/libssl.so.3").unwrap()),
            &(FilePath::new("libssl.so.3").unwrap()),
        );

        create_link(
            &(FilePath::new("/lib/libcrypto.so.3").unwrap()),
            &(FilePath::new("libcrypto.so.3").unwrap()),
        );

        create_link(
            &(FilePath::new("/lib/libstdc++.so.6").unwrap()),
            &(FilePath::new("libstdc++.so.6").unwrap()),
        );

        create_link(
            &(FilePath::new("/lib/libm.so.6").unwrap()),
            &(FilePath::new("libm.so.6").unwrap()),
        );

        create_link(
            &(FilePath::new("/lib/libgcc_s.so.1").unwrap()),
            &(FilePath::new("libgcc_s.so.1").unwrap()),
        );

        create_link(
            &(FilePath::new("/lib/libc.so.6").unwrap()),
            &(FilePath::new("libc.so.6").unwrap()),
        );
    }

    // create the file for the lmbench testcase
    let _ = new_file("/lat_sig", &(FileFlags::CREATE | FileFlags::RDWR));

    // gcc相关的链接，可以在testcases/gcc/riscv64-linux-musl-native/lib目录下使用ls -al指令查看
    let src_dir = "riscv64-linux-musl-native/lib";
    create_link(
        &FilePath::new(format!("{}/ld-musl-riscv64.so.1", src_dir).as_str()).unwrap(),
        &FilePath::new("/lib/libc.so").unwrap(),
    );
    create_link(
        &FilePath::new(format!("{}/libatomic.so", src_dir).as_str()).unwrap(),
        &FilePath::new(format!("{}/libatomic.so.1.2.0", src_dir).as_str()).unwrap(),
    );
    create_link(
        &FilePath::new(format!("{}/libatomic.so.1", src_dir).as_str()).unwrap(),
        &FilePath::new(format!("{}/libatomic.so.1.2.0", src_dir).as_str()).unwrap(),
    );
    create_link(
        &FilePath::new(format!("{}/libgfortran.so", src_dir).as_str()).unwrap(),
        &FilePath::new(format!("{}/libgfortran.so.5.0.0", src_dir).as_str()).unwrap(),
    );
    create_link(
        &FilePath::new(format!("{}/libgfortran.so.5", src_dir).as_str()).unwrap(),
        &FilePath::new(format!("{}/libgfortran.so.5.0.0", src_dir).as_str()).unwrap(),
    );
    create_link(
        &FilePath::new(format!("{}/libgomp.so", src_dir).as_str()).unwrap(),
        &FilePath::new(format!("{}/libgomp.so.1.0.0", src_dir).as_str()).unwrap(),
    );
    create_link(
        &FilePath::new(format!("{}/libgomp.so.1", src_dir).as_str()).unwrap(),
        &FilePath::new(format!("{}/libgomp.so.1.0.0", src_dir).as_str()).unwrap(),
    );
    create_link(
        &FilePath::new(format!("{}/libssp.so", src_dir).as_str()).unwrap(),
        &FilePath::new(format!("{}/libssp.so.0.0.0", src_dir).as_str()).unwrap(),
    );
    create_link(
        &FilePath::new(format!("{}/libssp.so.0", src_dir).as_str()).unwrap(),
        &FilePath::new(format!("{}/libssp.so.0.0.0", src_dir).as_str()).unwrap(),
    );
    create_link(
        &FilePath::new(format!("{}/libstdc++.so", src_dir).as_str()).unwrap(),
        &FilePath::new(format!("{}/libstdc++.so.6.0.29", src_dir).as_str()).unwrap(),
    );
    create_link(
        &FilePath::new(format!("{}/libstdc++.so.6", src_dir).as_str()).unwrap(),
        &FilePath::new(format!("{}/libstdc++.so.6.0.29", src_dir).as_str()).unwrap(),
    );
}
