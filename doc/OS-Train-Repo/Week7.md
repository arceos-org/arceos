# Week 7

本周实际工作内容大致是将 axstarry 依据 syscall 分为了不同的模块，方便后续根据 syscall 进行自动化选择模块启动。



将当前的模块划分为如下部分：

* syscall_task：主管任务模块，包括进程管理、调度、互斥资源管理等
* syscall_fs：文件系统对外封装，包括打开、关闭、读写文件等Linux相关语义支持
* syscall_mem：内存管理模块封装，包括动态分配堆内存、修改页面权限等操作
* syscall_net：网络控制模块，提供 socket 等网络相关结构的支持
* syscall_entry：系统调用入口，通过不同的 feature 启动不同的模块，并且加入对应的底层模块



