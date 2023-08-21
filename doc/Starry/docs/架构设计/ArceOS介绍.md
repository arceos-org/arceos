我们的Starry是基于ArceOS生成的，因此需要简单介绍一下ArceOS实现的内容。

ArceOS采用模块化组件化的设计思维，通过使用内核组件 + 组件化的OS框架 来得到 不同形态的OS kernel。

* 提供了一套组件化的操作系统框架
* 提供各种内核组件的实现，各种内核组件可在没有OS kernel的情况下独立运行
  * 如filesystem, network stack等内核组件可以在裸机或用户态以库的形式运行
  * 各种设备驱动等内核组件可以在裸机上运行
* 理想情况下可以通过选择组件构成unikernel/宏内核/微内核
* 实际上在我们开始实验时它还只支持unikernel
  * 只运行一个用户程序
  * 用户程序与内核链接为同一镜像
  * 不区分地址空间与特权级
  * 安全性由底层 hypervisor 保证

当前ArceOS是面向Unikernel设计的，而我们的Starry便是其宏内核化的一次尝试与成果。

![avatar](../figures/ArceOS介绍.png)