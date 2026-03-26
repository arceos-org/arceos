// Copyright 2025 The Axvisor Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Runtime library of [ArceOS](https://github.com/arceos-org/arceos).
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
#![allow(missing_abi)]

#[macro_use]
extern crate axlog;

#[cfg(all(target_os = "none", not(test)))]
mod lang_items;

#[cfg(feature = "smp")]
mod mp;

#[cfg(feature = "paging")]
mod klib;

#[cfg(feature = "smp")]
pub use self::mp::rust_main_secondary;

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

unsafe extern "C" {
    /// Application's entry point.
    fn main();
}

struct LogIfImpl;

#[crate_interface::impl_interface]
impl axlog::LogIf for LogIfImpl {
    fn console_write_str(s: &str) {
        axhal::console::write_bytes(s.as_bytes());
    }

    fn current_time() -> core::time::Duration {
        axhal::time::monotonic_time()
    }

    fn current_cpu_id() -> Option<usize> {
        #[cfg(feature = "smp")]
        if is_init_ok() {
            Some(axhal::percpu::this_cpu_id())
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

/// Number of CPUs that have completed initialization.
static INITED_CPUS: AtomicUsize = AtomicUsize::new(0);

fn is_init_ok() -> bool {
    INITED_CPUS.load(Ordering::Acquire) == axhal::cpu_num()
}

/// The main entry point of the ArceOS runtime.
///
/// It is called from the bootstrapping code in the specific platform crate (see
/// [`axplat::main`]).
///
/// `cpu_id` is the logic ID of the current CPU, and `arg` is passed from the
/// bootloader (typically the device tree blob address).
///
/// In multi-core environment, this function is called on the primary core, and
/// secondary cores call [`rust_main_secondary`].
#[cfg_attr(not(test), axplat::main)]
pub fn rust_main(cpu_id: usize, arg: usize) -> ! {
    #[cfg(not(feature = "plat-dyn"))]
    unsafe {
        axhal::mem::clear_bss()
    };
    axhal::percpu::init_primary(cpu_id);
    axhal::init_early(cpu_id, arg);
    let log_level = option_env!("AX_LOG").unwrap_or("info");

    ax_println!("{}", LOGO);
    ax_println!(
        indoc::indoc! {"
            arch = {}
            platform = {}
            target = {}
            build_mode = {}
            log_level = {}
            backtrace = {}
            smp = {}
        "},
        axconfig::ARCH,
        axconfig::PLATFORM,
        option_env!("AX_TARGET").unwrap_or(""),
        option_env!("AX_MODE").unwrap_or(""),
        log_level,
        axbacktrace::is_enabled(),
        axhal::cpu_num()
    );

    #[cfg(feature = "rtc")]
    ax_println!(
        "Boot at {}\n",
        chrono::DateTime::from_timestamp_nanos(axhal::time::wall_time_nanos() as _),
    );

    axlog::init();
    axlog::set_max_level(log_level); // no effect if set `log-level-*` features
    info!("Logging is enabled.");
    info!("Primary CPU {cpu_id} started, arg = {arg:#x}.");

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
    init_allocator();

    {
        use core::ops::Range;

        unsafe extern "C" {
            safe static _stext: [u8; 0];
            safe static _etext: [u8; 0];
            safe static _edata: [u8; 0];
        }

        axbacktrace::init(
            Range {
                start: _stext.as_ptr() as usize,
                end: _etext.as_ptr() as usize,
            },
            Range {
                start: _edata.as_ptr() as usize,
                end: usize::MAX,
            },
        );
    }

    let (kernel_space_start, kernel_space_size) = axhal::mem::kernel_aspace();

    info!(
        "kernel aspace: [{:#x?}, {:#x?})",
        kernel_space_start,
        kernel_space_start + kernel_space_size,
    );

    #[cfg(feature = "paging")]
    axmm::init_memory_management();

    // #[cfg(feature = "plat-dyn")]
    // axdriver::setup(arg);

    info!("Initialize platform devices...");
    axhal::init_later(cpu_id, arg);

    #[cfg(feature = "multitask")]
    axtask::init_scheduler();

    #[cfg(feature = "axdriver")]
    {
        #[allow(unused_variables)]
        let all_devices = axdriver::init_drivers();

        cfg_if::cfg_if! {
            if #[cfg(feature = "fs-ng")] {
                axfs_ng::init_filesystems(all_devices.block);
            } else
            if #[cfg(feature = "fs")] {
                axfs::init_filesystems(all_devices.block, axhal::dtb::get_chosen_bootargs());
            }
        }

        cfg_if::cfg_if! {
            if #[cfg(feature = "net-ng")] {
                axnet_ng::init_network(all_devices.net);

                #[cfg(feature = "vsock")]
                axnet_ng::init_vsock(all_devices.vsock);
            } else if #[cfg(feature = "net")] {
                axnet::init_network(all_devices.net);
            }
        }

        #[cfg(feature = "display")]
        axdisplay::init_display(all_devices.display);

        #[cfg(feature = "input")]
        axinput::init_input(all_devices.input);
    }

    #[cfg(feature = "smp")]
    self::mp::start_secondary_cpus(cpu_id);

    #[cfg(feature = "irq")]
    {
        info!("Initialize interrupt handlers...");
        init_interrupt();
    }

    #[cfg(all(feature = "tls", not(feature = "multitask")))]
    {
        info!("Initialize thread local storage...");
        init_tls();
    }

    ctor_bare::call_ctors();

    info!("Primary CPU {cpu_id} init OK.");
    INITED_CPUS.fetch_add(1, Ordering::Release);

    while !is_init_ok() {
        core::hint::spin_loop();
    }

    unsafe { main() };

    #[cfg(feature = "multitask")]
    axtask::exit(0);
    #[cfg(not(feature = "multitask"))]
    {
        debug!("main task exited: exit_code={}", 0);
        axhal::power::system_off();
    }
}

#[cfg(feature = "alloc")]
fn init_allocator() {
    use axhal::mem::{MemRegionFlags, memory_regions, phys_to_virt};

    info!("Initialize global memory allocator...");
    info!("  use {} allocator.", axalloc::global_allocator().name());

    let mut max_region_size = 0;
    let mut max_region_paddr = 0.into();
    let mut use_next_free = false;

    for r in memory_regions() {
        if r.name == ".bss" {
            use_next_free = true;
        } else if r.flags.contains(MemRegionFlags::FREE) {
            if use_next_free {
                max_region_paddr = r.paddr;
                break;
            } else if r.size > max_region_size {
                max_region_size = r.size;
                max_region_paddr = r.paddr;
            }
        }
    }

    #[cfg(feature = "buddy-slab")]
    {
        struct AddrTranslatorImpl;
        impl axalloc::AddrTranslator for AddrTranslatorImpl {
            fn virt_to_phys(&self, va: usize) -> Option<usize> {
                Some(axhal::mem::virt_to_phys(va.into()).as_usize())
            }
        }

        static TRANSLATOR: AddrTranslatorImpl = AddrTranslatorImpl;

        for r in memory_regions() {
            if r.flags.contains(MemRegionFlags::FREE) && r.paddr == max_region_paddr {
                axalloc::global_init(phys_to_virt(r.paddr).as_usize(), r.size, &TRANSLATOR);
                break;
            }
        }
    }

    #[cfg(not(feature = "buddy-slab"))]
    {
        for r in memory_regions() {
            if r.flags.contains(MemRegionFlags::FREE) && r.paddr == max_region_paddr {
                axalloc::global_init(phys_to_virt(r.paddr).as_usize(), r.size);
                break;
            }
        }
    }

    for r in memory_regions() {
        if r.flags.contains(MemRegionFlags::FREE) && r.paddr != max_region_paddr {
            axalloc::global_add_memory(phys_to_virt(r.paddr).as_usize(), r.size)
                .expect("add heap memory region failed");
        }
    }
}

#[cfg(feature = "irq")]
fn init_interrupt() {
    // Setup timer interrupt handler
    const PERIODIC_INTERVAL_NANOS: u64 =
        axhal::time::NANOS_PER_SEC / axconfig::TICKS_PER_SEC as u64;

    #[percpu::def_percpu]
    static NEXT_DEADLINE: u64 = 0;

    fn update_timer() {
        let now_ns = axhal::time::monotonic_time_nanos();
        // Safety: we have disabled preemption in IRQ handler.
        let mut deadline = unsafe { NEXT_DEADLINE.read_current_raw() };
        if now_ns >= deadline {
            deadline = now_ns + PERIODIC_INTERVAL_NANOS;
        }
        unsafe { NEXT_DEADLINE.write_current_raw(deadline + PERIODIC_INTERVAL_NANOS) };
        axhal::time::set_oneshot_timer(deadline);
    }

    axhal::irq::register(axhal::time::irq_num(), || {
        update_timer();
        #[cfg(feature = "multitask")]
        axtask::on_timer_tick();
    });

    #[cfg(feature = "ipi")]
    axhal::irq::register(axhal::irq::IPI_IRQ, || {
        axipi::ipi_handler();
    });

    // Enable IRQs before starting app
    axhal::asm::enable_irqs();
}

#[cfg(all(feature = "tls", not(feature = "multitask")))]
fn init_tls() {
    let main_tls = axhal::tls::TlsArea::alloc();
    unsafe { axhal::asm::write_thread_pointer(main_tls.tls_ptr() as usize) };
    core::mem::forget(main_tls);
}
