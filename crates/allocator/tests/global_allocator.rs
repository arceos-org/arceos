use allocator::{AllocResult, BaseAllocator, ByteAllocator};
use allocator::{
    BasicAllocator, BuddyByteAllocator, MiAllocator, SlabByteAllocator, TLSFAllocator,
    TLSFCAllocator,
};
use std::alloc::{GlobalAlloc, Layout, System};
use std::mem::size_of;

use core::panic;
use std::sync::Mutex;

pub enum AllocType {
    SystemAlloc,
    BasicAlloc,
    BuddyAlloc,
    SlabAlloc,
    TLSFCAlloc,
    TLSFRustAlloc,
    MiAlloc,
}

pub struct GlobalAllocator {
    basic_alloc: Mutex<BasicAllocator<0>>,
    buddy_alloc: Mutex<BuddyByteAllocator>,
    slab_alloc: Mutex<SlabByteAllocator>,
    tlsf_c_alloc: Mutex<TLSFCAllocator>,
    tlsf_rust_alloc: Mutex<TLSFAllocator>,
    mi_alloc: Mutex<MiAllocator>,
    alloc_type: AllocType,
    heap_arddress: usize,
    heap_size: usize,
}

const PAGE_SIZE: usize = 1 << 22; // need 4MB aligned (prepared for mimalloc)
const HEAP_SIZE: usize = 1 << 26; // 512MB
/// The memory heap in this test
static mut HEAP: [usize; HEAP_SIZE + PAGE_SIZE] = [0; HEAP_SIZE + PAGE_SIZE];

/// if FLAG=ture, always uses system_alloc
static mut FLAG: bool = false;

/// The global allocator to test alloc in user mode.
/// The alloc mode supported: system_alloc, basic_alloc, buddy_alloc,
/// slab_alloc, tlsf_c_alloc and tlsf_rust_alloc.
/// The basic_alloc can choose strategy: first_fit, best_fit and worst_fit.
impl GlobalAllocator {
    pub const fn new() -> Self {
        Self {
            basic_alloc: Mutex::new(BasicAllocator::new()),
            buddy_alloc: Mutex::new(BuddyByteAllocator::new()),
            slab_alloc: Mutex::new(SlabByteAllocator::new()),
            tlsf_c_alloc: Mutex::new(TLSFCAllocator::new()),
            tlsf_rust_alloc: Mutex::new(TLSFAllocator::new()),
            mi_alloc: Mutex::new(MiAllocator::new()),
            alloc_type: AllocType::SystemAlloc,
            heap_arddress: 0,
            heap_size: 0,
        }
    }

    pub unsafe fn init_heap(&mut self) {
        self.heap_arddress = (HEAP.as_ptr() as usize + PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE;
        self.heap_size = HEAP_SIZE * size_of::<usize>();
    }

    pub unsafe fn init_system(&mut self) {
        self.alloc_type = AllocType::SystemAlloc;
    }

    pub unsafe fn init_basic(&mut self, strategy: &str) {
        self.basic_alloc
            .lock()
            .unwrap()
            .init(self.heap_arddress, self.heap_size);
        self.basic_alloc.lock().unwrap().set_strategy(strategy);
        self.alloc_type = AllocType::BasicAlloc;
    }

    pub unsafe fn init_buddy(&mut self) {
        self.buddy_alloc
            .lock()
            .unwrap()
            .init(self.heap_arddress, self.heap_size);
        self.alloc_type = AllocType::BuddyAlloc;
    }

    pub unsafe fn init_slab(&mut self) {
        self.slab_alloc
            .lock()
            .unwrap()
            .init(self.heap_arddress, self.heap_size);
        self.alloc_type = AllocType::SlabAlloc;
    }

    pub unsafe fn init_tlsf_c(&mut self) {
        self.tlsf_c_alloc
            .lock()
            .unwrap()
            .init(self.heap_arddress, self.heap_size);
        self.alloc_type = AllocType::TLSFCAlloc;
    }

    pub unsafe fn init_tlsf_rust(&mut self) {
        self.tlsf_rust_alloc
            .lock()
            .unwrap()
            .init(self.heap_arddress, self.heap_size);
        self.alloc_type = AllocType::TLSFRustAlloc;
    }

    pub unsafe fn init_mi(&mut self) {
        self.mi_alloc
            .lock()
            .unwrap()
            .init(self.heap_arddress, self.heap_size);
        self.alloc_type = AllocType::MiAlloc;
    }

    pub unsafe fn alloc(&self, layout: Layout) -> AllocResult<usize> {
        let size: usize = layout.size();
        let align_pow2: usize = layout.align();
        if FLAG {
            let ptr = System.alloc(layout);
            return Ok(ptr as usize);
        }

        match self.alloc_type {
            AllocType::SystemAlloc => {
                let ptr = System.alloc(layout);
                return Ok(ptr as usize);
            }
            AllocType::BasicAlloc => {
                FLAG = true;
                if let Ok(ptr) = self.basic_alloc.lock().unwrap().alloc(size, align_pow2) {
                    FLAG = false;
                    return Ok(ptr);
                } else {
                    panic!("alloc err: no memery.");
                }
            }
            AllocType::BuddyAlloc => {
                FLAG = true;
                if let Ok(ptr) = self.buddy_alloc.lock().unwrap().alloc(size, align_pow2) {
                    FLAG = false;
                    return Ok(ptr);
                } else {
                    panic!("alloc err: no memery.");
                }
            }
            AllocType::SlabAlloc => {
                FLAG = true;
                if let Ok(ptr) = self.slab_alloc.lock().unwrap().alloc(size, align_pow2) {
                    FLAG = false;
                    return Ok(ptr);
                } else {
                    panic!("alloc err: no memery.");
                }
            }
            AllocType::TLSFCAlloc => {
                FLAG = true;
                if let Ok(ptr) = self.tlsf_c_alloc.lock().unwrap().alloc(size, align_pow2) {
                    FLAG = false;
                    return Ok(ptr);
                } else {
                    panic!("alloc err: no memery.");
                }
            }
            AllocType::TLSFRustAlloc => {
                FLAG = true;
                if let Ok(ptr) = self.tlsf_rust_alloc.lock().unwrap().alloc(size, align_pow2) {
                    FLAG = false;
                    return Ok(ptr);
                } else {
                    panic!("alloc err: no memery.");
                }
            }
            AllocType::MiAlloc => {
                FLAG = true;
                if let Ok(ptr) = self.mi_alloc.lock().unwrap().alloc(size, align_pow2) {
                    FLAG = false;
                    return Ok(ptr);
                } else {
                    panic!("alloc err: no memery.");
                }
            }
        }
    }

    pub unsafe fn dealloc(&self, pos: usize, layout: Layout) {
        let size: usize = layout.size();
        let align_pow2: usize = layout.align();
        if FLAG {
            System.dealloc(pos as *mut u8, layout);
            return;
        }

        match self.alloc_type {
            AllocType::SystemAlloc => {
                System.dealloc(pos as *mut u8, layout);
            }
            AllocType::BasicAlloc => {
                FLAG = true;
                self.basic_alloc
                    .lock()
                    .unwrap()
                    .dealloc(pos, size, align_pow2);
                FLAG = false;
            }
            AllocType::BuddyAlloc => {
                FLAG = true;
                self.buddy_alloc
                    .lock()
                    .unwrap()
                    .dealloc(pos, size, align_pow2);
                FLAG = false;
            }
            AllocType::SlabAlloc => {
                FLAG = true;
                self.slab_alloc
                    .lock()
                    .unwrap()
                    .dealloc(pos, size, align_pow2);
                FLAG = false;
            }
            AllocType::TLSFCAlloc => {
                FLAG = true;
                self.tlsf_c_alloc
                    .lock()
                    .unwrap()
                    .dealloc(pos, size, align_pow2);
                FLAG = false;
            }
            AllocType::TLSFRustAlloc => {
                FLAG = true;
                self.tlsf_rust_alloc
                    .lock()
                    .unwrap()
                    .dealloc(pos, size, align_pow2);
                FLAG = false;
            }
            AllocType::MiAlloc => {
                FLAG = true;
                self.mi_alloc.lock().unwrap().dealloc(pos, size, align_pow2);
                FLAG = false;
            }
        }
    }

    pub fn total_bytes(&self) -> usize {
        match self.alloc_type {
            AllocType::SystemAlloc => 0,
            AllocType::BasicAlloc => self.basic_alloc.lock().unwrap().total_bytes(),
            AllocType::BuddyAlloc => self.buddy_alloc.lock().unwrap().total_bytes(),
            AllocType::SlabAlloc => self.slab_alloc.lock().unwrap().total_bytes(),
            AllocType::TLSFCAlloc => self.tlsf_c_alloc.lock().unwrap().total_bytes(),
            AllocType::TLSFRustAlloc => self.tlsf_rust_alloc.lock().unwrap().total_bytes(),
            AllocType::MiAlloc => self.mi_alloc.lock().unwrap().total_bytes(),
        }
    }

    pub fn used_bytes(&self) -> usize {
        match self.alloc_type {
            AllocType::SystemAlloc => 0,
            AllocType::BasicAlloc => self.basic_alloc.lock().unwrap().used_bytes(),
            AllocType::BuddyAlloc => self.buddy_alloc.lock().unwrap().used_bytes(),
            AllocType::SlabAlloc => self.slab_alloc.lock().unwrap().used_bytes(),
            AllocType::TLSFCAlloc => self.tlsf_c_alloc.lock().unwrap().used_bytes(),
            AllocType::TLSFRustAlloc => self.tlsf_rust_alloc.lock().unwrap().used_bytes(),
            AllocType::MiAlloc => self.mi_alloc.lock().unwrap().used_bytes(),
        }
    }

    pub fn available_bytes(&self) -> usize {
        match self.alloc_type {
            AllocType::SystemAlloc => 0,
            AllocType::BasicAlloc => self.basic_alloc.lock().unwrap().available_bytes(),
            AllocType::BuddyAlloc => self.buddy_alloc.lock().unwrap().available_bytes(),
            AllocType::SlabAlloc => self.slab_alloc.lock().unwrap().available_bytes(),
            AllocType::TLSFCAlloc => self.tlsf_c_alloc.lock().unwrap().available_bytes(),
            AllocType::TLSFRustAlloc => self.tlsf_rust_alloc.lock().unwrap().available_bytes(),
            AllocType::MiAlloc => self.mi_alloc.lock().unwrap().available_bytes(),
        }
    }
}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Ok(ptr) = GlobalAllocator::alloc(self, layout) {
            ptr as _
        } else {
            panic!("alloc err.");
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        GlobalAllocator::dealloc(self, ptr as _, layout)
    }
}

#[global_allocator]
pub static mut GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator::new();
