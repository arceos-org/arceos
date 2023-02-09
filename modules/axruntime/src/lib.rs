#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate axlog;

#[cfg(not(test))]
mod lang_items;

pub use axlog::{__print_impl, print, println};

const LOGO: &str = r#"
       d8888                            .d88888b.   .d8888b.
      d88888                           d88P" "Y88b d88P  Y88b
     d88P888                           888     888 Y88b.
    d88P 888 888d888  .d8888b  .d88b.  888     888  "Y888b.
   d88P  888 888P"   d88P"    d8P  Y8b 888     888     "Y88b.
  d88P   888 888     888      88888888 888     888       "888
 d8888888888 888     Y88b.    Y8b.     Y88b. .d88P Y88b  d88P
d88P     888 888      "Y8888P  "Y8888   "Y88888P"   "Y8888P"
"#;

extern "Rust" {
    fn main();
}

struct LogIfImpl;

impl axlog::LogIf for LogIfImpl {
    fn console_write_str(&self, s: &str) {
        use axhal::console::putchar;
        for c in s.chars() {
            match c {
                '\n' => {
                    putchar(b'\r');
                    putchar(b'\n');
                }
                _ => putchar(c as u8),
            }
        }
    }
}

#[cfg_attr(not(test), no_mangle)]
pub extern "C" fn rust_main() -> ! {
    axlog::set_interface(&LogIfImpl);
    println!("{}", LOGO);
    println!(
        "\
        arch = {}\n\
        platform = {}\n\
        build_mode = {}\n\
        log_level = {}\n\
        ",
        option_env!("ARCH").unwrap_or(""),
        option_env!("PLATFORM").unwrap_or(""),
        option_env!("MODE").unwrap_or(""),
        option_env!("LOG").unwrap_or(""),
    );

    axlog::init();
    axlog::set_max_level(option_env!("LOG").unwrap_or("")); // no effect if set `log-level-*` features
    info!("Logging is enabled.");

    info!("Found physcial memory regions:");
    for r in axhal::mem::memory_regions() {
        info!(
            "  [0x{:016x}, 0x{:#016x}) {} ({:?})",
            r.paddr,
            r.paddr + r.size,
            r.name,
            r.flags
        );
    }

    #[cfg(feature = "alloc")]
    init_allocator();

    #[cfg(feature = "paging")]
    remap_kernel_memory().expect("remap kernel memoy failed");

    #[cfg(any(feature = "fs", feature = "net"))]
    init_drivers();

    #[cfg(feature = "multitask")]
    init_scheduler();

    unsafe { main() };

    axhal::misc::terminate()
}

#[cfg(feature = "alloc")]
fn init_allocator() {
    use axhal::mem::{memory_regions, phys_to_virt, MemRegionFlags};

    info!("Initialize global memory allocator...");
    let mut max_region_size = 0;
    let mut max_region_paddr = 0.into();
    for r in memory_regions() {
        if r.flags.contains(MemRegionFlags::FREE) && r.size > max_region_size {
            max_region_size = r.size;
            max_region_paddr = r.paddr;
        }
    }
    for r in memory_regions() {
        if r.flags.contains(MemRegionFlags::FREE) && r.paddr == max_region_paddr {
            axalloc::global_init(phys_to_virt(r.paddr).as_usize(), r.size);
            break;
        }
    }
    for r in memory_regions() {
        if r.flags.contains(MemRegionFlags::FREE) && r.paddr != max_region_paddr {
            axalloc::global_add_memory(phys_to_virt(r.paddr).as_usize(), r.size)
                .expect("add heap memory region failed");
        }
    }
}

#[cfg(feature = "paging")]
fn remap_kernel_memory() -> Result<(), axhal::paging::PagingError> {
    use axhal::mem::{memory_regions, phys_to_virt};
    use axhal::paging::PageTable;

    info!("Initialize kernel page table...");
    let mut kernel_page_table = PageTable::try_new()?;
    for r in memory_regions() {
        kernel_page_table.map_region(
            phys_to_virt(r.paddr),
            r.paddr,
            r.size,
            r.flags.into(),
            true,
        )?;
    }

    unsafe { axhal::arch::write_page_table_root(kernel_page_table.root_paddr()) };
    core::mem::forget(kernel_page_table);
    Ok(())
}

#[cfg(any(feature = "fs", feature = "net"))]
fn init_drivers() {
    info!("Initialize drivers...");
    axdriver::init_drivers();
}

#[cfg(feature = "multitask")]
fn init_scheduler() {
    info!("Initialize scheduling...");
    axtask::init_scheduler();
}
