
axmem 模块中提供了关于地址空间、内存段等结构的抽象。并负责动态加载应用程序、进程的 fork() / spawn()、内存管理（mmap、shmat）等基本任务。

### MemorySet

MemorySet 代表一个进程所拥有（或多个线程共有）的地址空间。从一个用户程序“生命周期”的角度考虑。内核使用`MemorySet::map_elf()`加载 elf 文件中的内容，通过 clone()、mmap()、munmap()、add_shared_mem() 等函数执行由 syscall 传递而来的各种操作，最终在 drop() 中回收资源。

地址空间中最为核心的数据结构是页表（page_table）。使用RAII思想，页表（PageTable 类型）拥有页表本身使用的物理页。

除页表外，owned_mem 记录了在通过 mmap 以及应用程序加载时所获得的全部虚拟内存段。private_mem 和 sttached_mem 分别记录了进程所拥有的 System V Shared Memory。

以及，MemorySet 会记录加载自 ELF 文件的 entry 地址。

```rust
/// PageTable + MemoryArea for a process (task)
pub struct MemorySet {
    page_table: PageTable,
    owned_mem: BTreeMap<usize, MapArea>,

    private_mem: BTreeMap<i32, Arc<SharedMem>>,
    attached_mem: Vec<(VirtAddr, MappingFlags, Arc<SharedMem>)>,

    pub entry: usize,
}
```

### SharedMem

本数据结构代表用户程序通过 shmat() syscall 申请的共享内存。同样使用 RAII 思想，SharedMem::pages 中存放共享内存段所实际分配的物理页。这使得当我们 drop SharedMem 时，对应的物理内存也会自动释放。
SharedMem::info 记录了该段共享内存的属性信息。

```rust
pub struct SharedMem {
    pages: GlobalPage,
    pub info: SharedMemInfo,
}
```

对于进程私有的共享内存，SharedMem 会被放入进程的 MemorySet::private_mem 中保存。对于公开的跨进程共享内存，则会存储在全局数据结构 SHARED_MEMS 中。

```rust
/// This struct only hold SharedMem that are not IPC_PRIVATE. IPC_PRIVATE SharedMem will be stored
/// in MemorySet::detached_mem.
///
/// This is the only place we can query a SharedMem using its shmid.
///
/// It holds an Arc to the SharedMem. If the Arc::strong_count() is 1, SharedMem will be dropped.
pub static SHARED_MEMS: SpinNoIrq<BTreeMap<i32, Arc<SharedMem>>> = SpinNoIrq::new(BTreeMap::new());
pub static KEY_TO_SHMID: SpinNoIrq<BTreeMap<i32, i32>> = SpinNoIrq::new(BTreeMap::new());
```

当进程“挂载（映射）”某一共享内存段时，SharedMem 也会被加入 MemorySet::sttached_mem 中，便于执行进程的其他操作时访问。使用 Arc 数据结构可以得到“一段共享内存被多少个进程加载了”的信息，以便及时回收内存。

### MapArea

MapArea 代表一段虚拟内存。在这个数据结构中需要记录虚拟内存的起始地址、属性信息、对应的物理页与文件后端。

```rust
pub struct MapArea {
    pub pages: Vec<Option<PhysPage>>,
    /// 起始虚拟地址
    pub vaddr: VirtAddr,
    pub flags: MappingFlags,
    pub backend: Option<MemBackend>,
}
```

这里的 pages 使用了 Vec 存储若干单独的物理页，而非一整个物理页段。这是因为连续的一段虚拟内存对应的物理内存可能是不连续的。同时，Option 意味着物理内存可能尚未分配。这是本 OS 具有的 Lazy Load 功能。
用户态中连续已分配的内存区间可能实际上没有分配，当用户使用到未分配区段时，将触发 PageFault，此时将交由 MapArea::handle_page_fault() 处理，分配实际的物理内存。

为了实现 mmap 映射文件的 Lazy Load 功能，MapArea 中记录了一个可选的文件后端 MemBackend。处理 PageFault 时，如果此段内存具有对应的文件后端，则会将文件对应位置的内容写入新分配的内存。此外，为了实现 msync() 等强制同步内存与对应文件的 syscall，提供了 MapArea::sync_page_with_backend() 函数。

除了处理内存的懒加载，MapArea 还提供了对连续内存段的编辑功能。shrink_left()、shrink_right()、split()、split3() 函数可以修改当前内存段的大小，并及时释放物理内存。
