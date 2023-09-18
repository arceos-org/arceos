#!/bin/bash
sudo apt update
sudo apt install gcc cmake g++ build-essential pkg-config zlib1g-dev libglib2.0-dev meson libpixman-1-dev ninja-build libfdt-dev -y
git clone https://github.com/foxsen/qemu.git
cd qemu
git checkout loongarch
mkdir build
cd build
../configure --target-list=loongarch64-softmmu,loongarch64-linux-user --enable-kvm --enable-debug --disable-werror
make -j$(nproc)
#make install