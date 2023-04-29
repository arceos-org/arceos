extern crate alloc;
extern crate allocator;
use core::alloc::{GlobalAlloc, Layout};

use alloc::alloc::handle_alloc_error;
use allocator::{BaseAllocator, ByteAllocator, SlabByteAllocator};
use spinlock::SpinRaw;

use crate::task::sbrk;

struct UserAllocator(SpinRaw<SlabByteAllocator>);

#[global_allocator]
static GLOBAL_ALLOCATOR: UserAllocator = UserAllocator(SpinRaw::new(SlabByteAllocator::new()));

unsafe impl GlobalAlloc for UserAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.0.lock();
        //println!("alloc {:?}!", layout);
        loop {
            if let Ok(ptr) = allocator.alloc(layout.size(), layout.align()) {
                return ptr as _;
            } else {
                let size = allocator
                    .total_bytes()
                    .max(layout.size())
                    .next_power_of_two()
                    .max(4096);
                let res = sbrk(size as isize);
                if res < 0 {
                    handle_alloc_error(layout);
                }
                println!("sbrked {:x}, {:x}", res, size);
                allocator.add_memory(res as usize, size).unwrap();
                println!("size {:x}", allocator.total_bytes());
            }
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.0.lock();
        allocator.dealloc(ptr as _, layout.size(), layout.align());
    }
}

pub(crate) fn init() {
    let start_addr = crate::syscall::task::sbrk(0x8000);
    if start_addr < 0 {
        panic!("Error when preparing memory")
    }
    GLOBAL_ALLOCATOR.0.lock().init(start_addr as usize, 0x8000);
}
