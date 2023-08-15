#!/bin/bash

gzip -9 -cvf os.bin > arceos-fu740.bin.gz
mkimage -f tools/fu740/fu740_uboot_fit.its arceos-fu740.itb
echo 'Built the FIT-uImage arceos-fu740.itb'

###
### 编译ArceOS for fu740
#
# make A=apps/fs/shell ARCH=riscv64 PLATFORM=riscv64-qemu-virt LOG=info FS=y APP_FEATURES=axstd/driver-ramdisk fu740

### 基于U-Boot启动系统镜像

# 首先搭建一台tftp服务器, 例如，在Linux服务器中安装`tftpd-hpa`, 一般tftp服务的目录会在`/srv/tftp/`;

# 然后把编译出的ArceOS for fu740系统镜像`arceos-fu740.itb`拷贝到tftp服务目录；

# 开发板fu740开机，并进入U-Boot命令行：

# ```
# # 配置开发板IP地址和服务器IP地址
# setenv ipaddr <IP>
# setenv serverip <Server IP>

# # 通过tftp协议加载系统镜像
# tftp 0xa0000000 arceos-fu740.itb

# # 运行
# bootm 0xa0000000
# ```

###
# go
# u-boot go 命令传给内核的命令参数会减掉"go"程序本身字符, a0是u-boot命令参数的个数，a0是保存命令参数的字符指针数组;
