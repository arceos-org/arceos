#!/bin/sh
################################################################
# 	MacOS  : brew install dosfstools
# 	Ubuntu : apt-get install dosfstools
#	Usage:
# 		build_img.sh -m [arch] -fs [ext4|fat32] -file [testcast]
################################################################
# default setting
arch=x86_64
fs=fat32
FILE=

display_help()
{
	echo ""
	echo "./build_img.sh -m [arch] -fs [filesystem] -file [testcast]"
	echo ""
	exit 1
}

while [ -n "$1" ]; do
	case $1 in
		-m)
			shift
			arch="$1"
			;;
		-fs)
			shift
			fs="$1"
			;;
		-file)
			shift
			FILE="$1"
			;;
		riscv64)
			arch=riscv64
			;;
		x86_64)
			arch=x86_64
			;;
		aarch64)
			arch=aarch64
			;;
		fat32)
			fs=fat32
			;;
		ext4)
			fs=ext4
			;;
		sdcard)
			FILE=sdcard
			;;
		gcc)
			FILE=gcc
			;;
		redis)
			FILE=redis
			;;
		testsuits-x86_64-linux-musl)
			FILE=testsuits-x86_64-linux-musl
			;;
		ZLM)
			FILE=ZLM
			;;
		libc-dynamic)
			FILE=libc-dynamic
			;;
		libc-static)
			FILE=libc-static
			;;
		*)
			display_help
			;;
	esac
	shift
done


if [ -z "$FILE" ]; then # use default testcases
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

rm -f disk.img
dd if=/dev/zero of=disk.img bs=4M count=30

if [ "$fs" = "ext4" ]; then
	mkfs.ext4 -t ext4 disk.img
else
	fs=fat32
	mkfs.vfat -F 32 disk.img
fi

mkdir -p mnt

#### 添加 MacOS 支持
os=`uname -s`
if [ "x$os" = "xDarwin" ];then
	hdiutil detach mnt > /dev/null 2>&1
	hdiutil attach disk.img -mountpoint mnt
	echo "Copying $arch $fs $FILE/* to disk"
	cp -r ./testcases/$FILE/* ./mnt/
	hdiutil detach mnt
	chmod 777 disk.img
else
	sudo mount disk.img mnt
	# 根据命令行参数生成对应的测例
	echo "Copying $arch $fs $FILE/* to disk"
	sudo cp -r ./testcases/$FILE/* ./mnt/
	sudo umount mnt
	sudo rm -rf mnt
	sudo chmod 777 disk.img
fi