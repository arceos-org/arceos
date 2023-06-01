# 进展日志

## 第6周
1. 运行 arceos 的 `helloworld`，浏览了多线程相关的代码。
2. 阅读 ext3 的论文 "Journaling the Linux ext2fs Filesystem"，了解了如何高效实现日志和故障恢复，论文地址：https://pdos.csail.mit.edu/6.S081/2020/readings/journal-design.pdf
3. 阅读了 xv6 文件系统部分的代码，它实现了一个简单的带有 log 的文件系统，但是不如 ext3 高效。

### 问题
1. 据我了解，ext4 文件系统有很多的 feature 在 arceos 中无法体现（比如 superblock 的备份，block extent之类的），而且类似于 flexible block group 的 feature 所要求的文件系统容量也比较大，我们在实现文件系统时是否需要严格按照其规范实现？还是说实现其关键的 feature 和功能即可。


### 下一周计划
1. 将 xv6 的文件系统迁移至 arceos，预计实现顺序为：block_manager, 支持 concurrency 的 buffer_manager, 文件（夹）读写、创建删除、软链接等、LOG机制。
2. 进一步调研 ext3 中的LOG机制。

## 第7周
1. 调整了一下目标，因为 ext4 过于复杂，决定按照 ext2 -> （增加 log 机制）ext3 -> （添加其他特性）类ext4
的顺序来进行实现。
2. 阅读了 [The Second Extended File System](https://www.nongnu.org/ext2-doc/ext2.html)，完全了解
了 ext2 的磁盘布局、文件索引等信息。
3. 在 `crates/ext2fs` 中实现了 ext2 文件系统的所有磁盘数据结构，并完成了一部分 buffer_manager 的代码。

### 问题
1. `virtio-driver` 的 block size 是 512 bytes，但是 ext2 要求至少是 1024 bytes，而我想要实现的是 4096 bytes。这个大小是否可以调整呢。
2. 不清楚应该在 crate 中还是在 module 中实现一个支持并发的 buffer_manager，比如说 xv6 的如下代码在 arceos 中感觉找不到比较好的支持：
```c
static struct buf*
bget(uint dev, uint blockno)
{
  struct buf *b;

  acquire(&bcache.lock);

  // Is the block already cached?
  for(b = bcache.head.next; b != &bcache.head; b = b->next){
    if(b->dev == dev && b->blockno == blockno){
      b->refcnt++;
      release(&bcache.lock);
      acquiresleep(&b->lock); //？ sleep lock 需要 OS 的支持
      return b; //？ 如何返回一个上锁的结构的引用？可以返回一个 MutexGuard<T> 这样的对象吗？
    }
  }
  ...
}
//？ 综上，考虑到所需要的对于 OS 的支持，是否应该在 modules 中而非 crates 中实现 buffer_manager
//？ 但是这么做会导致 crates/ext2fs 的割裂比较严重（把 buffer_manager 实现成一个 trait？）
```

### 下一周计划
1. 在上述问题得到解决前，会先在 ext2fs 中实现一个可以从镜像中创建、读写文件系统的库，同时也先不考虑并发。
这部分剩下的工作有：
+ file_disk_manager: 模拟一个基于镜像文件的磁盘管理器
+ buffer_manager (no sync)
+ vfs: 先支持常见的文件操作：创建目录、创建文件，读写文件
2. 与助教沟通，找到一个协调并发问题的方案。


## 第 8 周
1. 在 `crates/ext2fs` 中仿照 `easy-fs` 写了文件系统的接口，目前支持：创建ext2文件系统镜像、从镜像中打开文件系统 `create_file`、`create_dir`、`link` 、`unlink` 等功能。

### 下一周计划
1. 将 ext2 集成到目前的 Arceos 的文件系统框架中；
2. 进一步完善 ext2 文件系统的功能，比如支持软链接、unlink 一个目录（目前只支持文件）、文件状态；
3. 调研带日志的文件系统的实现，准备把 ext2 升级至 ext3。

## 第 9 周
1. 在 ext2 中支持了更多的文件操作，比如：symlink、chown、chmod、truncate
2. 实现了一个 LRU 的 buffer_manager，一些细节如下：
+ 这里我们希望可以用一个容器来管理所有的缓存 (BTreeMap)，同时希望可以维护一个 LRU 队列，所以需要使用到
侵入式链表：
```rust
pub struct BlockCacheManager {
    device: Arc<dyn BlockDevice>,
    max_cache: usize,
    blocks: BTreeMap<usize,Arc<SpinMutex<BlockCache>>>, // 实际上管理 cache 的生命周期
    lru_head: InListNode<BlockCache, ManagerAccessBlockCache> // LRU 链表头
}

pub struct BlockCache {
    lru_head: InListNode<BlockCache, ManagerAccessBlockCache>,
    block_id: usize,
    modified: bool,
    valid: bool,
    cache: Box<[u8]>
}
```
侵入式链表本身和普通的 C 风格的链表类似，内部使用指针来维护连接关系，但是本身不包含数据
```rust
pub struct  ListNode<T> {
    prev: *mut ListNode<T>,
    next: *mut ListNode<T>,
    data: T
}

pub struct InListNode<T, A = ()> {
    node: ListNode<PhantomData<(T, A)>>,
}
```
这里，为了能够通过侵入式链表来访问数据，可以参考 C 语言中的技巧：即可以通过计算 `BlockCache` 中的 `lru_head` 域相对于这个结构体的偏移，然后对于 `&lru_head` 就可以减去这个偏移得到 `BlockCache` 的
起始地址，然后将其转为：`*mut BlockCache` 即可。当然，这么做需要我们自己确保操作的安全性。
```rust
// 该特征中的方法可以从一个侵入式链表的表头得到包含它的结构体的引用
pub trait ListAccess<A, B>: 'static {
    fn offset() -> usize;
    #[inline(always)]
    unsafe fn get(b: &B) -> &A {
        &*(b as *const B).cast::<u8>().sub(Self::offset()).cast()
    }
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    unsafe fn get_mut(b: &mut B) -> &mut A {
        &mut *(b as *mut B).cast::<u8>().sub(Self::offset()).cast()
    }
}

#[macro_export]
macro_rules! inlist_access {
    ($vis: vis $name: ident, $T: ty, $field: ident) => {
        $vis struct $name {}
        impl $crate::list::access::ListAccess<$T, $crate::list::instrusive::InListNode<$T, Self>>
            for $name
        {
            #[inline(always)]
            fn offset() -> usize {
                $crate::offset_of!($T, $field)
            }
        }
    };
}

// 例子
crate::inlist_access!(AccessA, A, node);
struct A {
    _v1: usize,
    node: InListNode<A, AccessA>,
    _v2: usize,
}
```
+ 为了可以更加灵活使用锁，比如有些情况下需要再不持有锁的前提下对其内部进行读写（自信确保安全性），所以
重新实现了支持以上操作的 `SpinMutex`：
```rust
pub struct SpinMutex<T: ?Sized, S: MutexSupport> {
    lock: AtomicBool,
    _marker: PhantomData<S>,
    _not_send_sync: PhantomData<*const ()>,
    data: UnsafeCell<T>, // actual data
}

impl<T: ?Sized, S: MutexSupport> SpinMutex<T, S> {
    ...
    #[inline(always)]
    pub unsafe fn unsafe_get(&self) -> &T {
        &*self.data.get()
    }
    #[allow(clippy::mut_from_ref)]
    #[inline(always)]
    pub unsafe fn unsafe_get_mut(&self) -> &mut T {
        &mut *self.data.get()
    }
    ...
}
```
+ 最后简单介绍一下 LRU buffer_manager 的实现：

    - 在获取缓存块时，先在 blocks 中查找，如果没有则看目前的缓存块数是否达到上限，没有达到则新分配一块，否则就顺着 LRU 队列找到一个没有被其他进程持有的块牺牲掉，这个可以使用 `Arc` 的引用计数来判断。
    - 为了 LRU 策略可以正常执行，需要再不使用块后显示调用 `release_block`，它在没有进程使用该块时会将它重新插入到 LRU 队列的尾部
    ```rust
    pub fn release_block(&mut self, bac: Arc<SpinMutex<BlockCache>>) {
        if Arc::strong_count(&bac) == 2 {
            let ptr = unsafe { bac.unsafe_get_mut() };
            ptr.lru_head.pop_self();
            self.lru_head.push_prev(&mut ptr.lru_head);
            
        }
    }
    ```

3. 阅读了 ![ftl-os]( https://gitlab.eduxiji.net/DarkAngelEX/oskernel2022-ftlos) 中文件系统的实现，上述的侵入式链表就是参考这个实现。另外他们也实现了一个 `inode_manager` 用来管理 `inode` 的缓存，这样在读写文件的时候就不需要每次读磁盘才能知道要读写的块，同时也可以处理多个进程同时读写一个文件的情况。另外也可以更好地支持 Linux 对于文件操作的规范：即只有当文件的引用计数和被所有进程的引用计数都归零后才会回收对应的空间。

### 下一周计划
1. 实现 `inode_manager`，解决并发问题，并且加上缓存机制来提高速度（可选）
2. 为 ext2 提供更好的封装，目前的实现都是直接操作 `Inode`，可以进一步包装成 `Dir` 和 `File`，也可以支持路径搜索等更加复杂的操作
3. 进一步阅读 `ftl-os` 的实现以及 Linux 的相关资料，主要想要了解 vfs 如何设计（或许只是想知道？）




## 第 10、11 周
1. 实现了 `inode_manager` ，主要用于处理并发问题，并且加上了缓存机制来优化读写文件的速度，大致如下：
    + 下面是 `inode` 的缓存结构，它不会写回磁盘，而是根据文件操作而同步更新，保存了文件类型、文件大小，文件中所有块的序号。其中 `valid` 用于表示该 cache 是否有效，当文件被删除时，该位会被置为 0，这样其他持有该文件的进程对于这个文件的所有操作都会失效。
        ```rust
        pub struct InodeCache {
            pub inode_id: usize,
            block_id: usize,
            block_offset: usize,
            fs: Arc<Ext2FileSystem>,

            // cache part
            file_type: u8,
            size: usize,
            blocks: Vec<u32>,
            pub valid: bool,
        }
        ```
        这样在读写文件的时候就不需要调用 `get_block_id` 方法先从磁盘中读取索引了：
        ```rust
        pub fn write_at(
            &mut self,
            offset: usize,
            buf: &[u8],
            manager: &SpinMutex<BlockCacheManager>,
            cache: Option<&Vec<u32>>
        ) -> usize {
            // ...
            let block_id = if let Some(blocks) = cache.as_ref() {
                blocks[start_block]
            } else {
                self.get_block_id(start_block as _, manager)
            };
            // ...
        }
        ```

    + 下面是 `inode_manager`，所有对 `InodeCache` 的访问都要通过它，这样可以保证全局只有一个独立的 `InodeCache`，防止出现多个线程同时读写同一个文件的情况：
        ```rust
        pub struct InodeCacheManager {
            inodes: BTreeMap<usize, Arc<SpinMutex<InodeCache>>>,
            max_inode: usize
        }

        pub struct Inode {
            file_type: u8,
            inner: Arc<SpinMutex<InodeCache>>
        }
        ```

2. 支持了对目录的 unlink，以及可以直接 recursive unlink。


### 下一周计划
1. 和另一位同学交流，把日志功能接入到 ext2 文件系统中。


## 第 12 周
1. 使用 Ext2 文件系统替换 arceos 中的根文件系统，目前支持 fs/shell 中的所有操作：
```bash
arceos:/$ mkdir dira
arceos:/$ cd dira
arceos:/dira/$ echo hello > a.txt
arceos:/dira/$ echo world > b.txt
arceos:/dira/$ ls
-rwxr-xr-x        6 a.txt
-rwxr-xr-x        6 b.txt
arceos:/dira/$ cat a.txt
hello
arceos:/dira/$ rm a.txt
arceos:/dira/$ ls
-rwxr-xr-x        6 b.txt
arceos:/dira/$ pwd
/dira/
arceos:/dira/$ cd ..
arceos:/$ pwd
/
```

### 下一周计划
1. 目前 ext2 实现中的错误处理比较简陋，例如返回一个 `Option<T>`，我希望可以根据 arceos 中 VFS 的设计来
完善错误处理这部分功能。
2. 目前 ext2 的一部分功能，比如说：link、unlink、rm_dir 都没有在 fs/shell 中体现，下一步我会为 arceos 
增加更多的文件相关的系统调用，来用上这些功能。
3. 和另一位同学沟通，将日志功能加入 ext2 文件系统。

## 第 13 周
1. 完善了 ext2 的错误处理，如下所示：
```rust
pub enum Ext2Error {
    /// A directory entry already exists
    AlreadyExists,
    /// A directory is not empty when delete
    DirectoryIsNotEmpty,
    /// An directory entity is not found
    NotFound,
    /// There is no enough storage space for write
    NotEnoughSpace,
    /// The entry has been deleted
    InvalidResource,
    /// The operation is only valid in file
    NotAFile,
    /// The operation is only valid in directory
    NotADir,
    /// Invalid inode number
    InvalidInodeId,
    /// Link to itself
    LinkToSelf,
    /// Link to directory
    LinkToDir,
    /// Path too long when doing symbolic link
    PathTooLong,
    /// Name too long when adding dentry in directory
    NameTooLong,
    /// Not a symbolic link
    NotSymlink,
    /// Invalid file/directory name
    InvalidName,
}

pub type Ext2Result<T = ()> = Result<T, Ext2Error>;
```

2. 支持递归删除文件夹、硬链接，因此修改了 VFS 接口：
```rust
/// Create a hard link to target (maybe file or symlink)
fn link(&self, _name: &str, _handle: &LinkHandle) -> VfsResult {
    ax_err!(Unsupported)
}

/// for hard link support
/// (这是因为硬链接要求是同一个文件系统，LinkHandle 可以用来判断这一点)
fn get_link_handle(&self) -> VfsResult<LinkHandle> {
    ax_err!(Unsupported)
}
/// Remove the node with given `path` in the directory.
fn remove(&self, _path: &str, _recursive: bool) -> VfsResult {
    ax_err!(Unsupported)
}
```

3. 支持软链接，因此路径查询需要在 VFS 层进行，因为软链接可以跨文件系统，这里通过限制软链接跳转的次数
来防止无限循环：
```rust
fn _lookup_symbolic(dir: Option<&VfsNodeRef>, path: &str, count: &mut usize, max_count: usize, final_jump: bool) -> AxResult<VfsNodeRef> {
    debug!("_lookup_symbolic({}, {})", path, count);
    if path.is_empty() {
        return ax_err!(NotFound);
    }
    let parent = parent_node_of(dir, path);
    let is_dir = path.ends_with("/");
    let path = path.trim_matches('/');
    let names = axfs_vfs::path::split_path(path);

    let mut cur = parent.clone();

    for (idx, name) in names.iter().enumerate() {
        let vnode = cur.clone().lookup(name.as_str())?;
        let ty = vnode.get_attr()?.file_type();
        if ty == VfsNodeType::SymLink {
            if idx == names.len() - 1 && !final_jump {
                return Ok(vnode);
            }
            *count += 1;
            if *count > max_count {
                return Err(VfsError::NotFound);
            }
            let mut new_path = vnode.get_path()?;
            let rest_path = names[idx+1..].join("/");
            if !rest_path.is_empty() {
                new_path += "/";
                new_path += &rest_path;
            }
            if is_dir {
                new_path += "/";
            }
            return _lookup_symbolic(None, &new_path, count, max_count, final_jump);
        } else {
            if idx == names.len() - 1 {
                if is_dir && !ty.is_dir() {
                    return Err(AxError::NotADirectory);
                }
                return Ok(vnode);
            } else {
                match ty {
                    VfsNodeType::Dir => {
                        cur = vnode.clone();
                    },
                    VfsNodeType::File => {
                        return Err(AxError::NotADirectory);
                    },
                    _ => panic!("unsupport type")
                };
            }
        }
    }

    panic!("_lookup_symbolic");
}
```

### 下一周计划
1. 整合日志模块。