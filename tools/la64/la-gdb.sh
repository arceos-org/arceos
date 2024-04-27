#!/bin/bash
git clone https://github.com/foxsen/binutils-gdb
git checkout loongarch-v2022-03-10
cd binutils-gdb
mkdir "build"
cd "build"
../configure --target=loongarch64-unknown-linux-gnu --prefix=/opt/gdb
make -j$(nproc)
sudo make install