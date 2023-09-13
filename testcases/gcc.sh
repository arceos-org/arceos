# !bin/bash
# wget https://musl.cc/riscv64-linux-musl-native.tgz
# 如果wget下载过慢，也可以手动将riscv64-linux-musl-native.tgz从浏览器下载到当前目录
cp ./riscv64-linux-musl-native.tgz ./gcc/
cd ./gcc
tar -xvf riscv64-linux-musl-native.tgz
rm -rf riscv64-linux-musl-native.tgz
cd ..
