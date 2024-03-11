# StarryOS

## 简介

这里是StarryOS，一个基于ArceOS实现的宏内核。

> Starry意指布满星星的，寓意本OS的开发学习借鉴了许多前辈的思路，并将其汇总归一为这个内核。

在线文档详见：[Starry (azure-stars.github.io)](https://azure-stars.github.io/Starry/)

## 成员

陈嘉钰、郑友捷、王昱栋

## Usage

通过修改 `apps/monolithic_userboot/src/batch.rs` 中的 `SDCARD_TESTCASES` 常量，并且在启动时加入编译参数`APP_FEATURES=batch`，可以选择让内核启动后以批处理形式运行给定程序。

如果未添加 `APP_FEATURES=batch` 参数，内核将以交互式的形式运行，默认开机之后直接进入`busybox sh`终端。

### x86_64

```shell
# 构建镜像
# 默认构建x86_64架构的fat32磁盘镜像
./build_img.sh

# 或构建ext4格式的磁盘文件
# ./build_img.sh x86_64 ext4

# 运行宏内核
make run

# 或运行ext4文件系统的宏内核
# make run FEATURES=ext4fs

# 显式指定参数并运行（实际上这些参数已在根目录 Makefile 中给出）
# make A=apps//build_img.sh sdcard AARCH=x86_64 FEATURES=fp_simd run

```

如果 `./build_x86.sh` 卡住，可以[手动下载](https://github.com/oscomp/testsuits-for-oskernel/releases/download/final-x86_64/testsuits-x86_64-linux-musl.tgz)测例文件，然后把其中的 `wget` 一行注释掉并再次执行。


### RISC-V

```shell
# 构建镜像
./build_img.sh riscv

# 运行 Unikernel 架构内核
make run

# 以宏内核形式启动(当前仅支持 riscv 架构)
make A=apps/monolithic_userboot ARCH=riscv64 run

# 使用 ramdisk 加载文件镜像并且运行内核，可以显著提高文件 IO 速度
make A=apps/monolithic_userboot ARCH=riscv64 FEATURES=img run

# 使用批处理模式启动宏内核并且运行给定测例
make A=apps/monolithic_userboot ARCH=riscv64 APP_FEATURES=batch run

```

## CI 说明
本项目的 CI 结构继承了 arceos 的 CI 结构，同时在其上加入了宏内核的测试用例，以求保证该项目可以在宏内核和 Unikernel 架构以及不同的指令集架构下正常运行。

当前的 Unikernel 基本适配了 arceos 测例，而宏内核仅支持在 riscv 架构上运行。各个 CI 含义如下：

* Clippy CI：代码风格检查

* Test CI / unit-test：单元测试，当前由于宏内核代码紧耦合 riscv 而导致无法通过单元测试

* build CI / build：默认架构 ( Unikernel + x86_64 ) 构建测试

* build CI / build-apps-for-unikernel + ARCH：Unikernel 架构下不同指令集的测例构建测试

* build CI / build-apps-for-monolithic + ARCH：宏内核架构下不同指令集的测例构建测试

* Test CI / app-test-for-unikernel + ARCH：Unikernel 架构下不同指令集的测例运行测试

* Test CI / app-test-for-monolithic + ARCH：宏内核架构下不同指令集的测例运行测试

## 项目结构

### 整体结构图

![image-20230603005345201](https://raw.githubusercontent.com/Azure-stars/Figure-Bed/main/image-20230603005345201.png)



### 模块依赖图

```mermaid
graph TD;
axsync-->axdisplay
axdriver-->axdisplay

axhal-->axdriver
axalloc-->axdriver
axconfig-->axdriver

axdriver-->axfs
axsync-->axfs
axtask-.dev.->axfs

axconfig-->axhal
axalloc-->axhal
axlog-->axhal

axhal-->axnet
axsync-->axnet
axtask-->axnet
axdriver-->axnet

axalloc-->axruntime
axconfig-->axruntime
axdriver-->axruntime
axhal-->axruntime
axlog-->axruntime
axnet-->axruntime
axdisplay-->axruntime
axtask-->axruntime
axprocess-->axruntime
axtask-->axsync
axtask-->axprocess
axfs-->axprocess
axhal-->axprocess

axalloc-->axtask
axhal-->axtask
axconfig-->axtask
axlog-->axtask

axfs-->axmem
axalloc-->axmem
axhal-->axmem
axmem-->axprocess
```

* crates：与OS设计无关的公共组件
* modules：与OS设计更加耦合的组件
* doc：每周汇报文档，当前位于doc分支上
* apps：unikernel架构下的用户程序，继承原有ArceOS
* scripts：makefile脚本，继承原有ArceOS
* ulib：用户库，继承原有ArceOS



## 测例切换和执行

执行如下指令可以生成sdcard文件镜像

```shell
$ ./build_img.sh sdcard
```

如果想要切换到其他测例，如切换到gcc，请在保证testcases/gcc文件夹下对应文件夹内容满足需求之后，执行如下指令

```shell
$ ./build_img.sh gcc
```

当使用 gcc 测例时，由于 gcc 测例内容过大，不直接拷贝到 ramdisk 上，因此不能启动 `FEATURES=img`。

通过修改指令可以切换生成的文件镜像中包含的测例。相应测例存放在`testcases/`文件夹下，如执行`./build_img.sh libc-static`可以生成libc静态测例。



## 文档

内核文档存放在`doc/Starry决赛设计文档.pdf`。

另外，可以通过静态部署网页[Starry (azure-stars.github.io)](https://azure-stars.github.io/Starry/)查看更好排版的文档。

[关于ZLMediaKit 的支持文档](./doc/ZLMediaKit/README.md)
