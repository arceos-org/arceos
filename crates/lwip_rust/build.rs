use std::path::PathBuf;

fn main() {
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    compile_lwip(&arch);
    generate_lwip_bindings();
}

fn generate_lwip_bindings() {
    println!("cargo:rustc-link-lib=lwip");
    println!("cargo:include=depend/lwip/src/include/");

    let bindings = bindgen::Builder::default()
        .use_core()
        .header("wrapper.h")
        .clang_arg("-I./depend/lwip/src/include")
        .clang_arg("-I./custom")
        .clang_arg("-I../../ulib/c_libax/include")
        .clang_arg("-Wno-everything")
        .layout_tests(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from("src");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn compile_lwip(arch: &str) {
    let mut base_config = cc::Build::new();

    match arch {
        "riscv64" => {
            base_config.compiler("riscv64-linux-musl-gcc");
            base_config.flag("-mabi=lp64d");
        }
        "aarch64" => {
            base_config.compiler("aarch64-linux-musl-gcc");
            base_config.flag("-mgeneral-regs-only");
        }
        "x86_64" => {
            base_config.compiler("x86_64-linux-musl-gcc");
            base_config.flag("-mno-sse");
        }
        _ => {
            panic!("Unsupported arch: {}", arch);
        }
    }

    base_config
        .include("depend/lwip/src/include")
        .include("custom")
        .include("../../ulib/c_libax/include");

    base_config
        .file("depend/lwip/src/api/err.c")
        .file("depend/lwip/src/core/init.c")
        .file("depend/lwip/src/core/def.c")
        .file("depend/lwip/src/core/dns.c")
        .file("depend/lwip/src/core/inet_chksum.c")
        .file("depend/lwip/src/core/ip.c")
        .file("depend/lwip/src/core/mem.c")
        .file("depend/lwip/src/core/memp.c")
        .file("depend/lwip/src/core/netif.c")
        .file("depend/lwip/src/core/pbuf.c")
        .file("depend/lwip/src/core/raw.c")
        .file("depend/lwip/src/core/stats.c")
        .file("depend/lwip/src/core/sys.c")
        .file("depend/lwip/src/core/altcp.c")
        .file("depend/lwip/src/core/altcp_alloc.c")
        .file("depend/lwip/src/core/altcp_tcp.c")
        .file("depend/lwip/src/core/tcp.c")
        .file("depend/lwip/src/core/tcp_in.c")
        .file("depend/lwip/src/core/tcp_out.c")
        .file("depend/lwip/src/core/timeouts.c")
        .file("depend/lwip/src/core/udp.c")
        .file("depend/lwip/src/core/ipv4/acd.c")
        .file("depend/lwip/src/core/ipv4/autoip.c")
        .file("depend/lwip/src/core/ipv4/dhcp.c")
        .file("depend/lwip/src/core/ipv4/etharp.c")
        .file("depend/lwip/src/core/ipv4/icmp.c")
        .file("depend/lwip/src/core/ipv4/igmp.c")
        .file("depend/lwip/src/core/ipv4/ip4_frag.c")
        .file("depend/lwip/src/core/ipv4/ip4.c")
        .file("depend/lwip/src/core/ipv4/ip4_addr.c")
        .file("depend/lwip/src/core/ipv6/dhcp6.c")
        .file("depend/lwip/src/core/ipv6/ethip6.c")
        .file("depend/lwip/src/core/ipv6/icmp6.c")
        .file("depend/lwip/src/core/ipv6/inet6.c")
        .file("depend/lwip/src/core/ipv6/ip6.c")
        .file("depend/lwip/src/core/ipv6/ip6_addr.c")
        .file("depend/lwip/src/core/ipv6/ip6_frag.c")
        .file("depend/lwip/src/core/ipv6/mld6.c")
        .file("depend/lwip/src/core/ipv6/nd6.c")
        .file("depend/lwip/src/netif/ethernet.c")
        .file("depend/lwip/src/apps/lwiperf/lwiperf.c")
        .file("../../ulib/c_libax/src/string.c")
        .file("custom/sys_arch.c");

    base_config
        .warnings(false)
        .flag("-static")
        .flag("-no-pie")
        .flag("-fno-builtin")
        .flag("-ffreestanding")
        .flag("-nostdinc")
        .compile("liblwip.a");
}
