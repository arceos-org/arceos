#!/bin/sh

# default setting
arch=x86_64
fs=fat32
FILE=

if [ -n "$1" ]; then
	if [ "$1" = "riscv64" ] || [ "$1" = "aarch64" ] || [ "$1" = "x86_64" ]; then # $1 is arch
		arch=$1
		if [ -n "$2" ]; then
			if [ "$2" = "ext4" ] || [ "$2" = "fat32" ]; then # $2 is type of file system 
				fs=$2
				if [ -n "$3" ]; then
					FILE=$3
				fi
			else  # $2 is folder in testcases
				FILE=$2
			fi
		fi
	elif [ "$1" = "ext4" ] || [ "$1" = "fat32" ]; then # $1 is type of file system 
		fs=$1
		if [ -n "$2" ]; then
			FILE=$2
		fi
	else # $1 is folder in testcases
		FILE=$1
	fi
fi

if [ ! -n $FILE ] || [ "$FILE" = "" ]; then # use default testcases
	if [ "$arch" = "riscv64" ]; then
		FILE=sdcard
	elif [ "$arch" = "x86_64" ]; then
		FILE=testsuits-x86_64-linux-musl
	elif [ "$arch" = "aarch64" ]; then
		FILE=aarch64
	else
		exit 1
	fi
fi

if [ "$FILE" = "testsuits-x86_64-linux-musl" ] && [ ! -e testcases/$FILE ]; then # auto download
	wget https://github.com/oscomp/testsuits-for-oskernel/releases/download/final-x86_64/$FILE.tgz
	tar zxvf $FILE.tgz
	mv $FILE testcases/$FILE -f
	rm $FILE.tgz
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
