#!/bin/bash

CUR_DIR=$(dirname $0)

write_fs() {
    for i in $(seq 1 1000); do
        echo "Rust is cool!" >>long.txt
    done
    echo "Rust is cool!" >short.txt
    mkdir -p a/long/path
    echo "Rust is cool!" >a/long/path/test.txt
    mkdir -p very-long-dir-name
    echo "Rust is cool!" >>very-long-dir-name/very-long-file-name.txt
}
init_fs() {
    local name=$1
    local options=$2
    mkdir -p mnt
    sudo mount -o loop "$name" mnt -o "$options"

    sudo chmod 777 mnt

    cd mnt
    write_fs
    cd ..

    sudo umount mnt
    rm -r mnt
}
create_fat_img() {
    local name=$1
    local kb=$2
    local fatSize=$3
    dd if=/dev/zero of="$name" bs=1K count=$kb
    mkfs.vfat -s 1 -F $fatSize "$name"

    init_fs "$name" rw,uid=$USER,gid=$USER
}
create_ext4_img() {
    local name=$1
    local kb=$2
    dd if=/dev/zero of="$name" bs=1K count=$kb
    mkfs.ext4 -O ^metadata_csum "$name"

    init_fs "$name" rw
}

create_fat_img "$CUR_DIR/fat16.img" 2500 16
create_fat_img "$CUR_DIR/fat32.img" 34000 32

create_ext4_img "$CUR_DIR/ext4.img" 30000
