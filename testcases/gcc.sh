# !bin/bash
cp ./riscv64-linux-musl-native.tgz ./gcc/
cd ./gcc
tar -xvf riscv64-linux-musl-native.tgz
rm -rf riscv64-linux-musl-native.tgz
cd ..
