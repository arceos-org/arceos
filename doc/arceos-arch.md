# ArceOS Architecture

## ArceOS Modules
* [axruntime](../modules/axruntime/): Bootstraping from the bare-metal environment, and initialization.
* [axhal](../modules/axhal/): Hardware abstraction layer, provides unified APIs for cross-platform.
* [axconfig](../modules/axconfig/): Platform constants and kernel parameters, such as physical memory base, kernel load addresses, stack size, etc.
* [axlog](../modules/axlog/): Multi-level log definition and printing.
* [axalloc](../modules/axalloc/): Dynamic memory allocation.
* [axdriver](../modules/axdriver/): Device driver framework.
* [axdisplay](../modules/axdisplay/): Graphic display framework.
* [axfs](../modules/axfs/): File system framework with low/high level filesystem manipulation operations.
* [axnet](../modules/axnet/): Network stack.
* [axsync](../modules/axsync/): Synchronization primitives.
* [axtask](../modules/axtask/): Task management.

## Crates
* [allocator](../crates/allocator): Memory allocation: bitmap&buddy allocator in page size,  slab allocator in byte size.
* [arm_gic](../crates/arm_gic): ARM GIC(Generic Interrupt Controller) registers & ops.
* [axerrno](../crates/axerrno): Error number in linux.
* [axio](../crates/axio): `std`-like traits, helpers, and type definitions for core I/O functionality.
* [axfs_devfs](../crates/axfs_devfs): Device file system.
* [axfs_vfs](../crates/axfs_vfs): Virtual filesystem interfaces.
* [crate_interface](../crates/crate_interface): crate interface macros for OPs between crates.
* [driver_block](../crates/driver_block): trait(read_block/write_block/flush) of BlockDriver.
* [driver_common](../crates/driver_common): trait(device_name/device_type) of BaseDriver, types of drivers.
* [driver_display](../crates/driver_display):  FrameBuffer, DisplayInfo, trait of DisplayDriver on virtio-gpu.
* [driver_net](../crates/driver_net): trait of NetBuffer & NetDriver.
* [driver_virtio](../crates/driver_virtio): config & probe for VirtioDevice(Block/Net/GPU).
* [handler_table](../crates/handler_table): Exception/Interrupt Handler Table for Hardware abstraction layer -- [axhal](../modules/axhal/).
* [kernel_guard](../crates/kernel_guard): IRQ OPs.
* [lazy_init](../crates/lazy_init): Global static variable instance OPs.
* [linked_list](../crates/linked_list): Based on linux/rust/kernel/linked_list.rs, but use [`unsafe_list::List`](../crates/linked_list/src/unsafe_list.rs) as the inner implementation..
* [memory_addr](../crates/memory_addr): PhyAddr/VirtAddr related OPs.
* [page_table](../crates/page_table): Generic page table. 
* [page_table_entry](../crates/page_table_entry): Generic page table entry.
* [percpu](../crates/percpu): Framework for per-cpu data.
* [percpu_macros](../crates/percpu_macros): Macros for per-cpu data.
* [ratio](../crates/ratio): Convert `numerator / denominator` to `mult / (1 << shift)` to avoid `u128` division.
* [scheduler](../crates/scheduler): FIFO/RR schedulers.
* [slab_allocator](../crates/slab_allocator): Memory: slab allocator.
* [spinlock](../crates/spinlock): Sync: spin lock. 
* [timer_list](../crates/timer_list): Timer event/list & OPs. 
* [tuple_for_each](../crates/tuple_for_each): tuple_for_each to traverse devices. 
## Dependencies

```mermaid
graph TD;
subgraph "User Apps"
A["Rust App"] 
C["C App"]
end
subgraph "ArceOS ulib"
B["rust_libax"]
D["c_libax"]
E("rust_std")
F("c_musl")
end
A --> B;
C --> D;
A --> E;
D --> B;
E --> B;
E --> F;
C --> F;
subgraph "System compatible layer"
G["aeceos"]
H("linux")
end
B --> G;
F --> H;
subgraph "ArceOS modules"
I[axruntime]
J[axlog]
K[axsync]
L[axhal]
M[axconfig]
N[axalloc]
O[axtask]
P[axdriver]
Q[axnet]
Q1[axdisplay]
M1[axfs]
end
G --> I;
H --> I;
I --> IN6;
I --> IN3;
I --> IN9;
I --> IN4;
I --> IN15;
I --> N;
I --> M;
I --> P;
I --> L;
I --> J;
I --> Q;
I --> Q1;
I --> O;
Q1 --> P;
Q1 --> IN4;
Q1 --> K;
Q1 --> T5;
P --> IN11;
P --> T0;
P --> T1;
P --> T2;
P --> T3;
P --> T5;
P --> N;
P --> L;
P --> M;
N --> IN9;
N --> IN5;
N --> R;
N --> IN13;
L --> M;
L --> N;
L --> IN3;
L --> IN9;
L --> IN8;
L --> IN4;
L --> P1;
L --> P11;
L --> IN6;
L --> IN5;
L --> IN2;
L --> IN15;
J --> IN9;
J --> IN15;
Q --> T0;
Q --> T2;
Q --> IN4;
Q --> IN13;
Q --> L;
Q --> K;
Q --> O;
Q --> P;
K --> IN9;
K --> O;
O --> L;
O --> M;
O --> IN6;
O --> IN9;
O --> IN4;
O --> IN5;
O --> S;
O --> IN10;
O --> IN3;
O --> IN15;
M1 --> IN4;
M1 --> F3;
M1 --> T0;
M1 --> T1;
M1 --> IN14;
M1 --> IN13;
M1 --> F1;
M1 --> F2;
M1 --> P;
M1 --> K;
subgraph "ArceOS crates"
R[allocator]
IN12[arm_gic]
IN13[axerrno]
F1[axfs_devfs]
F2[axfs_vfs]
IN14[axio]
F3[capability]
IN15[crate_interface]
T1[driver_blk]
T0[driver_common]
T5[driver_display]
T2[driver_net]
T3[driver_virtio]
IN2[handler_table]
IN3[kernel_guard]
IN4[lazy_init]
Z[linked_list]
IN5[memory_addr]
P1[page_table]
P11[page_table_entry]
IN6[percpu]
IN7[percpu_macros]
IN8[ratio]
S[scheduler-FIFO_RR]
IN1[slab_allocator]
IN9[spinlock]
IN10[timer_list]
IN11[tuple_for_each]
T4(e1000)
V(smoltcp)
W(lwip_rust)
OUT2(bitmap-allocator)
Y(slab_allocator)
S1(FIFO)
S2(RR)
OUT1(buddy_system_allocator)
OUT3(virtio-drivers)
end
R --> OUT1;
R --> OUT2;
R --> Y;
IN4 --> IN13;
S --> Z;
OUT1 --> Z;
OUT2 --> Z;
IN1 --> OUT1;
Y --> Z;
T3 --> T1;
T3 --> T2;
T3 --> T5;
T3 --> OUT3;
T3 --> T0;
T1 --> T0;
T2 --> T0;
T4 --> T0;
T5 --> T0;
IN3 --> IN15;
P1 --> IN5;
P1 --> P11;
P11 --> IN5;
IN6 --> IN3;
IN6 --> IN7;
IN9 --> IN3;
F3 --> IN13;
F2 --> IN13;
F1 --> F2;
```