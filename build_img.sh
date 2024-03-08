#!/bin/sh

arch=$1
fs=$2

if [ "$arch" != "riscv64" ] && [ "$arch" != "aarch64" ]; then
	arch=x86_64
	FILE=testsuits-x86_64-linux-musl
	if [ ! -e testcases/$FILE ]; then
		wget https://github.com/oscomp/testsuits-for-oskernel/releases/download/final-x86_64/$FILE.tgz
		tar zxvf $FILE.tgz
		mv $FILE testcases/$FILE -f
		rm $FILE.tgz
	fi
else
	if [ -n "$3" ]; then
		FILE=$3
	else
		if [ "$arch" = "riscv64" ]; then
			FILE=sdcard
		else
			FILE=aarch64
		fi
	fi
fi

rm disk.img
dd if=/dev/zero of=disk.img bs=4M count=30

if [ "$fs" = "ext4" ]; then
	mkfs.ext4 -t ext4 disk.img
else
	fs=fat32
	mkfs.vfat -F 32 disk.img
fi

mkdir -p mnt
sudo mount disk.img mnt

# 根据命令行参数生成对应的测例
echo "Copying $arch $fs $FILE/* to disk"
sudo cp -r ./testcases/$FILE/* ./mnt/
sudo umount mnt
sudo rm -rf mnt
sudo chmod 777 disk.img
