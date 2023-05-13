//! Runtime library of [ArceOS](https://github.com/rcore-os/arceos).
//!
//! Any application uses ArceOS should link this library. It does some
//! initialization work before entering the application's `main` function.
//!
//! # Cargo Features
//!
//! - `alloc`: Enable global memory allocator.
//! - `paging`: Enable page table manipulation support.
//! - `irq`: Enable interrupt handling support.
//! - `multitask`: Enable multi-threading support.
//! - `smp`: Enable SMP (symmetric multiprocessing) support.
//! - `fs`: Enable filesystem support.
//! - `net`: Enable networking support.
//! - `display`: Enable graphics support.
//!
//! All the features are optional and disabled by default.

#![cfg_attr(not(test), no_std)]
#![feature(doc_auto_cfg)]

#[macro_use]
extern crate axlog;

#[cfg(all(target_os = "none", not(test)))]
mod lang_items;
mod trap;

#[cfg(feature = "smp")]
mod mp;

#[cfg(feature = "user")]
mod syscall;

#[cfg(feature = "user")]
use axmem::{USER_START, USTACK_SIZE, USTACK_START};
#[cfg(feature = "smp")]
pub use self::mp::rust_main_secondary;
#[cfg(feature = "scheme")]
mod scheme;

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

extern "C" {
    fn main();
}

struct LogIfImpl;

#[crate_interface::impl_interface]
impl axlog::LogIf for LogIfImpl {
    fn console_write_str(s: &str) {
        axhal::console::write_bytes(s.as_bytes());
    }

    fn current_time() -> core::time::Duration {
        axhal::time::current_time()
    }

    fn current_cpu_id() -> Option<usize> {
        #[cfg(feature = "smp")]
        if is_init_ok() {
            Some(axhal::cpu::this_cpu_id())
        } else {
            None
        }
        #[cfg(not(feature = "smp"))]
        Some(0)
    }

    fn current_task_id() -> Option<u64> {
        if is_init_ok() {
            #[cfg(feature = "multitask")]
            {
                axtask::current_may_uninit().map(|curr| curr.id().as_u64())
            }
            #[cfg(not(feature = "multitask"))]
            None
        } else {
            None
        }
    }
}

use core::sync::atomic::{AtomicUsize, Ordering};

static INITED_CPUS: AtomicUsize = AtomicUsize::new(0);

fn is_init_ok() -> bool {
    INITED_CPUS.load(Ordering::Acquire) == axconfig::SMP
}

/// The main entry point of the ArceOS runtime.
///
/// It is called from the bootstrapping code in [axhal]. `cpu_id` is the ID of
/// the current CPU, and `dtb` is the address of the device tree blob. It
/// finally calls the application's `main` function after all initialization
/// work is done.
///
/// In multi-core environment, this function is called on the primary CPU,
/// and the secondary CPUs call [`rust_main_secondary`].
#[cfg_attr(not(test), no_mangle)]
pub extern "C" fn rust_main(cpu_id: usize, dtb: usize) -> ! {
    ax_println!("{}", LOGO);
    ax_println!(
        "\
        arch = {}\n\
        platform = {}\n\
        smp = {}\n\
        build_mode = {}\n\
        log_level = {}\n\
        ",
        option_env!("ARCH").unwrap_or(""),
        option_env!("PLATFORM").unwrap_or(""),
        option_env!("SMP").unwrap_or(""),
        option_env!("MODE").unwrap_or(""),
        option_env!("LOG").unwrap_or(""),
    );

    axlog::init();
    axlog::set_max_level(option_env!("LOG").unwrap_or("")); // no effect if set `log-level-*` features
    info!("Logging is enabled.");
    info!("Primary CPU {} started, dtb = {:#x}.", cpu_id, dtb);

    info!("Found physcial memory regions:");
    for r in axhal::mem::memory_regions() {
        info!(
            "  [{:x?}, {:x?}) {} ({:?})",
            r.paddr,
            r.paddr + r.size,
            r.name,
            r.flags
        );
    }

    #[cfg(feature = "alloc")]
    {
        info!("Initialize global memory allocator...");
        init_allocator();
    }

    #[cfg(feature = "paging")]
    {
        info!("Initialize kernel page table...");
        remap_kernel_memory().expect("remap kernel memoy failed");
        #[cfg(feature = "user-paging")]
        axmem::init_global_addr_space();
    }

    info!("Initialize platform devices...");
    axhal::platform_init();

    #[cfg(feature = "multitask")]
    axtask::init_scheduler();

    #[cfg(any(feature = "fs", feature = "net", feature = "display"))]
    {
        #[allow(unused_variables)]
        let all_devices = axdriver::init_drivers();

        #[cfg(feature = "fs")]
        axfs::init_filesystems(all_devices.block.0);

        #[cfg(feature = "net")]
        axnet::init_network(all_devices.net);

        #[cfg(feature = "display")]
        axdisplay::init_display(all_devices.display);
    }

    #[cfg(feature = "smp")]
    self::mp::start_secondary_cpus(cpu_id);

    #[cfg(feature = "irq")]
    {
        info!("Initialize interrupt handlers...");
        init_interrupt();
    }

    #[cfg(feature = "futex")]
    {
        info!("Initialize futex...");
        axsync::futex::init();
    }

    info!("Primary CPU {} init OK.", cpu_id);
    INITED_CPUS.fetch_add(1, Ordering::Relaxed);

    while !is_init_ok() {
        core::hint::spin_loop();
    }

    #[cfg(feature = "user")]
    trap::user_space_entry();


    #[cfg(not(feature = "user"))]
    {
        extern "Rust" {
            fn main();
        }
        unsafe { main() };        
        #[cfg(feature = "multitask")]
        axtask::exit(0);
        #[cfg(not(feature = "multitask"))]
        {
            debug!("main task exited: exit_code={}", 0);
            axhal::misc::terminate();
        }
    }
}

#[cfg(feature = "alloc")]
fn init_allocator() {
    use axhal::mem::{memory_regions, phys_to_virt, MemRegionFlags};

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
use axhal::paging::PageTable;
#[cfg(feature = "paging")]
fn remap_kernel_memory() -> Result<(), axhal::paging::PagingError> {
    use axhal::{
        mem::{memory_regions, phys_to_virt, virt_to_phys},
        paging::MappingFlags,
    };

    use lazy_init::LazyInit;

    static KERNEL_PAGE_TABLE: LazyInit<PageTable> = LazyInit::new();

    if axhal::cpu::this_cpu_is_bsp() {
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

        #[cfg(all(feature = "user", not(feature = "user-paging")))]
        init_user_space(&mut kernel_page_table)?;

        #[cfg(feature = "user-paging")]
        {
            extern "C" {
                fn strampoline();
            }
            kernel_page_table.map_region(
                axmem::TRAMPOLINE_START.into(),
                virt_to_phys((strampoline as usize).into()),
                axhal::mem::PAGE_SIZE_4K,
                MappingFlags::READ | MappingFlags::EXECUTE,
                false,
            )?;
        }

        KERNEL_PAGE_TABLE.init_by(kernel_page_table);
    }

    unsafe { axhal::arch::write_page_table_root(KERNEL_PAGE_TABLE.root_paddr()) };
    Ok(())
}

#[cfg(feature = "irq")]
fn init_interrupt() {
    use axhal::time::TIMER_IRQ_NUM;

    // Setup timer interrupt handler
    const PERIODIC_INTERVAL_NANOS: u64 =
        axhal::time::NANOS_PER_SEC / axconfig::TICKS_PER_SEC as u64;

    #[percpu::def_percpu]
    static NEXT_DEADLINE: u64 = 0;

    fn update_timer() {
        let now_ns = axhal::time::current_time_nanos();
        // Safety: we have disabled preemption in IRQ handler.
        let mut deadline = unsafe { NEXT_DEADLINE.read_current_raw() };
        if now_ns >= deadline {
            deadline = now_ns + PERIODIC_INTERVAL_NANOS;
        }
        unsafe { NEXT_DEADLINE.write_current_raw(deadline + PERIODIC_INTERVAL_NANOS) };
        axhal::time::set_oneshot_timer(deadline);
    }

    axhal::irq::register_handler(TIMER_IRQ_NUM, || {
        update_timer();
        #[cfg(feature = "multitask")]
        axtask::on_timer_tick();
    });

    // Enable IRQs before starting app
    axhal::arch::enable_irqs();
}

#[cfg(all(feature = "user", not(feature = "user-paging")))]
/// Set up user state memory
fn init_user_space(page_table: &mut PageTable) -> Result<(), axhal::paging::PagingError> {
    use axalloc::GlobalPage;
    use axhal::mem::{virt_to_phys, PAGE_SIZE_4K};
    use axhal::paging::MappingFlags;
    use axmem::AddrSpace;
    extern crate alloc;

    extern "C" {
        fn ustart();
        fn uend();
    }

    let user_elf: &[u8] = unsafe {
        let len = (uend as usize) - (ustart as usize);
        core::slice::from_raw_parts(ustart as *const _, len)
    };

    debug!("{:x} {:x}", ustart as usize, user_elf.len());

    let segments = elf_loader::SegmentEntry::new(user_elf).expect("Corrupted elf file!");

    fn align_up(size: usize) -> usize {
        (size + PAGE_SIZE_4K - 1) / PAGE_SIZE_4K
    }

    let mut phy_pages: alloc::vec::Vec<GlobalPage> = alloc::vec![];

    for segment in &segments {
        let mut user_phy_page = GlobalPage::alloc_contiguous(align_up(segment.size), PAGE_SIZE_4K)
            .expect("Alloc page error!");
        // init
        user_phy_page.zero();

        // copy user content
        user_phy_page.as_slice_mut()[..segment.data.len()].copy_from_slice(segment.data);
        debug!(
            "{:x} {:x}",
            user_phy_page.as_slice()[0],
            user_phy_page.as_slice()[1]
        );

        page_table.map_region(
            segment.start_addr,
            user_phy_page.start_paddr(virt_to_phys),
            user_phy_page.size(),
            segment.flags | MappingFlags::USER,
            false,
        )?;
        phy_pages.push(phy_pages);
    }

    // stack allocation
    assert!(USTACK_SIZE % PAGE_SIZE_4K == 0);
    let user_stack_page = GlobalPage::alloc_contiguous(USTACK_SIZE / PAGE_SIZE_4K, PAGE_SIZE_4K)
        .expect("Alloc page error!");
    debug!("{:?}", user_stack_page);

    page_table.map_region(
        USTACK_START.into(),
        user_stack_page.start_paddr(virt_to_phys),
        user_stack_page.size(),
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER,
        false,
    )?;
    phy_pages.push(phy_pages);
    Ok(())
}
