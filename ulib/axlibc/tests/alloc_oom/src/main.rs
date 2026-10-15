extern crate alloc;

use std::alloc::{GlobalAlloc, Layout, System};

#[allow(non_camel_case_types)]
mod ctypes {
    pub type size_t = usize;
}

mod errno {
    use std::sync::atomic::{AtomicI32, Ordering};
    static ERRNO: AtomicI32 = AtomicI32::new(0);

    pub fn set_errno(code: i32) {
        ERRNO.store(code, Ordering::Relaxed);
    }

    pub fn get_errno() -> i32 {
        ERRNO.load(Ordering::Relaxed)
    }
}

mod upstream {
    include!(concat!(env!("OUT_DIR"), "/upstream_malloc.rs"));
}

const HEAP_SIZE: usize = 256 * 1024;
const REQUEST: usize = 1 << 30;
const HEADER_SIZE: usize = 8;

#[repr(align(4096))]
#[allow(dead_code)]
struct Heap([u8; HEAP_SIZE]);

static mut HEAP: Heap = Heap([0; HEAP_SIZE]);

struct HostAllocator;

// Diagnostics use System. Oversized requests from upstream malloc are
// forwarded to the real axalloc GlobalAlloc method.
// The failure is therefore not a mocked NULL and cannot exhaust the host.
unsafe impl GlobalAlloc for HostAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() >= REQUEST {
            unsafe { GlobalAlloc::alloc(axalloc::global_allocator(), layout) }
        } else {
            unsafe { System.alloc(layout) }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let start = core::ptr::addr_of_mut!(HEAP).cast::<u8>() as usize;
        if (start..start + HEAP_SIZE).contains(&(ptr as usize)) {
            unsafe { GlobalAlloc::dealloc(axalloc::global_allocator(), ptr, layout) }
        } else {
            unsafe { System.dealloc(ptr, layout) }
        }
    }
}

#[global_allocator]
static HOST_ALLOCATOR: HostAllocator = HostAllocator;

fn main() {
    let start = core::ptr::addr_of_mut!(HEAP).cast::<u8>() as usize;
    axalloc::global_init(start, HEAP_SIZE);
    let mode = std::env::args().nth(1).expect("mode required");
    match mode.as_str() {
        "fallible" => {
            let allocator = axalloc::global_allocator();
            let layout = Layout::from_size_align(REQUEST + HEADER_SIZE, 8).unwrap();
            assert!(matches!(
                allocator.alloc(layout),
                Err(axallocator::AllocError::NoMemory)
            ));
            let small = Layout::from_size_align(64, 8).unwrap();
            let ptr = allocator.alloc(small).expect("recovery allocation");
            unsafe {
                ptr.as_ptr().write_bytes(0xa5, small.size());
                assert!((0..small.size()).all(|i| *ptr.as_ptr().add(i) == 0xa5));
            }
            allocator.dealloc(ptr, small);
            println!("PASS: real axalloc returned Err; small allocation/write/free survived");
        }
        "trait-oom" => {
            let layout = Layout::from_size_align(REQUEST + HEADER_SIZE, 8).unwrap();
            eprintln!("BEFORE: real axalloc GlobalAlloc::alloc, 1 GiB + 8 bytes");
            let ptr = unsafe { GlobalAlloc::alloc(axalloc::global_allocator(), layout) };
            assert!(ptr.is_null());
            println!("PASS: GlobalAlloc returned NULL");
        }
        "malloc-oom" => {
            eprintln!("BEFORE: unchanged axlibc malloc -> Rust alloc -> real axalloc");
            let ptr = unsafe { upstream::probe_malloc(REQUEST) };
            assert!(ptr.is_null());
            assert_eq!(errno::get_errno(), axerrno::LinuxError::ENOMEM as i32);
            println!("PASS: malloc returned NULL");
        }
        "rust-vec-oom" => {
            eprintln!("BEFORE: Rust Vec -> real axalloc; Rust infallible OOM must still abort");
            let mut data = Vec::<u8>::with_capacity(REQUEST + HEADER_SIZE);
            data.push(1);
            std::hint::black_box(data);
            panic!("unexpectedly allocated oversized Vec");
        }
        "size-max" | "layout-limit" | "max-valid" => {
            let size = if mode == "size-max" {
                usize::MAX
            } else if mode == "max-valid" {
                isize::MAX as usize - HEADER_SIZE - 7
            } else {
                isize::MAX as usize
            };
            eprintln!("BEFORE: unchanged axlibc malloc({size})");
            let ptr = unsafe { upstream::probe_malloc(size) };
            assert!(ptr.is_null());
            assert_eq!(errno::get_errno(), axerrno::LinuxError::ENOMEM as i32);
            println!("PASS: boundary request returned NULL");
        }
        "small" => unsafe {
            let ptr = upstream::probe_malloc(64).cast::<u8>();
            assert!(!ptr.is_null());
            ptr.write_bytes(0x5a, 64);
            assert!((0..64).all(|i| *ptr.add(i) == 0x5a));
            upstream::probe_free(ptr.cast());
            upstream::probe_free(core::ptr::null_mut());
            let zero = upstream::probe_malloc(0);
            upstream::probe_free(zero);
            println!("PASS: axlibc small allocation, free(NULL), malloc(0)/free");
        },
        _ => panic!("unknown mode: {mode}"),
    }
}
