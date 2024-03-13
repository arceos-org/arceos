use axalloc::GlobalPage;
use axerrno::AxResult;
use axhal::{
    mem::{virt_to_phys, PhysAddr, PAGE_SIZE_4K},
    time::current_time,
};

#[allow(dead_code)]
pub struct SharedMem {
    pages: GlobalPage,
    /// The information of the shared memory.
    pub info: SharedMemInfo,
}

impl SharedMem {
    /// Allocate a new shared memory.
    ///
    /// If the allocation fails, return an error.
    pub fn try_new(
        key: i32,
        size: usize,
        pid: u64,
        uid: u32,
        gid: u32,
        mode: u16,
    ) -> AxResult<Self> {
        let num_pages = (size + PAGE_SIZE_4K - 1) / PAGE_SIZE_4K;

        let pages = GlobalPage::alloc_contiguous(num_pages, PAGE_SIZE_4K)?;
        let size = pages.size();

        Ok(Self {
            pages,
            info: SharedMemInfo::new(key, size, pid, uid, gid, mode),
        })
    }

    /// Return the size of the shared memory.
    pub fn size(&self) -> usize {
        self.pages.size()
    }

    /// Return the start physical address of the shared memory.
    pub fn paddr(&self) -> PhysAddr {
        self.pages.start_paddr(virt_to_phys)
    }
}

#[allow(dead_code)]
pub struct SharedMemInfo {
    perm: SharedMemPermInfo,
    size: usize,

    a_time: usize,
    d_time: usize,
    c_time: usize,

    c_pid: u64,
    l_pid: u64,
}

#[allow(dead_code)]
pub struct SharedMemPermInfo {
    key: i32,
    uid: u32,
    gid: u32,
    cuid: u32,
    cgid: u32,
    mode: u16,
}

impl SharedMemInfo {
    /// Allocate a new SharedMem.
    ///
    /// This function should be called by SharedMem::try_new().
    fn new(key: i32, size: usize, pid: u64, uid: u32, gid: u32, mode: u16) -> Self {
        Self {
            perm: SharedMemPermInfo {
                key,
                uid,
                gid,
                cuid: uid,
                cgid: gid,
                mode,
            },
            size,
            a_time: 0,
            d_time: 0,
            c_time: current_time().as_secs() as usize,

            c_pid: pid,
            l_pid: 0,
        }
    }
}
