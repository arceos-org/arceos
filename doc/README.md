# ArceOS Architecture Overview

The key design principle of ArceOS is to divide components based on their **relevance to the OS design concept**, in order to reduce component coupling and improving reusability.

## Crates

Crates are **OS-agnostic** components that can be reused in other OS or system software projects with almost no modification, providing the most reusability. For example, the basic data structures, algorithms, and utilities.

See [arceos-crates](https://github.com/arceos-org/arceos-crates) for crates used by ArceOS.

## Modules

Modules are **OS-related** components that are tightly bound to the design principles of a specific OS and have relatively poor reusability. It may need to be redesigned when it is ported to another OS.

* [axalloc](../modules/axalloc): ArceOS global memory allocator.
* [axconfig](../modules/axconfig): Platform-specific constants and parameters for ArceOS.
* [axdisplay](../modules/axdisplay): ArceOS graphics module.
* [axdriver](../modules/axdriver): ArceOS device drivers.
* [axfs](../modules/axfs): ArceOS filesystem module.
* [axhal](../modules/axhal): ArceOS hardware abstraction layer, provides unified APIs for platform-specific operations.
* [axlog](../modules/axlog): Macros for multi-level formatted logging used by ArceOS.
* [axnet](../modules/axnet): ArceOS network module.
* [axruntime](../modules/axruntime): Runtime library of ArceOS.
* [axsync](../modules/axsync): ArceOS synchronization primitives.
* [axtask](../modules/axtask): ArceOS task management module.

## Applications (Rust)

| App | Extra modules | Enabled features | Description |
|-|-|-|-|
| [helloworld](../apps/helloworld/) | | | A minimal app that just prints a string |
| [exception](../apps/exception/) | | paging | Exception handling test |
| [memtest](../apps/memtest/) | axalloc | alloc, paging | Dynamic memory allocation test |
| [display](../apps/display/) | axalloc, axdisplay | alloc, paging, display | Graphic/GUI test |
| [yield](../apps/task/yield/) | axalloc, axtask | alloc, paging, multitask, sched_fifo | Multi-threaded yielding test |
| [parallel](../apps/task/parallel/) | axalloc, axtask | alloc, paging, multitask, sched_fifo, irq | Parallel computing test (to test synchronization & mutex) |
| [sleep](../apps/task/sleep/) | axalloc, axtask | alloc, paging, multitask, sched_fifo, irq | Thread sleeping test |
| [priority](../apps/task/priority/) | axalloc, axtask | alloc, paging, multitask, sched_cfs | Thread priority test |
| [shell](../apps/fs/shell/) | axalloc, axdriver, axfs | alloc, paging, fs | A simple shell that responds to filesystem operations |
| [httpclient](../apps/net/httpclient/) | axalloc, axdriver, axnet | alloc, paging, net | A simple client that sends an HTTP request and then prints the response |
| [echoserver](../apps/net/echoserver/) | axalloc, axdriver, axnet, axtask | alloc, paging, net, multitask | A multi-threaded TCP server that reverses messages sent by the client  |
| [httpserver](../apps/net/httpserver/) | axalloc, axdriver, axnet, axtask | alloc, paging, net, multitask | A multi-threaded HTTP server that serves a static web page |
| [udpserver](../apps/net/udpserver/) | axalloc, axdriver, axnet | alloc, paging, net | A simple echo server using UDP protocol |

## Applications (C)
| App | Extra modules | Enabled features | Description |
|-|-|-|-|
| [helloworld](../apps/c/helloworld/) | | | A minimal C app that just prints a string |
| [memtest](../apps/c/memtest/) | axalloc | alloc, paging | Dynamic memory allocation test in C |
| [sqlite3](../apps/c/sqlite3/) | axalloc, axdriver, axfs | alloc, paging, fp_simd, fs | Porting of [SQLite3](https://sqlite.org/index.html) |
| [iperf](../apps/c/iperf/) | axalloc, axdriver, axfs, axnet | alloc, paging, fp_simd, fs, net, select | Porting of [iPerf3](https://iperf.fr/) |
| [redis](../apps/c/redis/) | axalloc, axdriver, axtask, axfs, axnet | alloc, paging, fp_simd, irq, multitask, fs, net, pipe, epoll | Porting of [Redis](https://redis.io/) |

## Rustdoc

* https://arceos-org.github.io/arceos/

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
