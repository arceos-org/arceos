#!/bin/bash

# A script to build user apps and copy it to FAT FS

set -e
   
ARCH=riscv64
RELEASE=riscv64gc-unknown-none-elf

rm disk.img || true;
make disk_img
make A=apps/microkernel/net_deamon MICRO=y ARCH=$ARCH build_user
make A=apps/microkernel/apps MICRO_TEST=shell MICRO=y ARCH=$ARCH build_user
make A=apps/microkernel/apps MICRO_TEST=http_server MICRO=y ARCH=$ARCH build_user
make A=apps/microkernel/apps MICRO_TEST=http_client MICRO=y ARCH=$ARCH build_user

cd tools/fat32-pack
cargo run -- --disk ../../disk.img --file ../../target/$RELEASE/release/microkernel-net-deamon --output net_deamon
for app in shell http_server http_client; do
    echo Copy $app to disk
    cargo run -- --disk ../../disk.img --file ../../target/$RELEASE/release/$app
done
cd ../..
