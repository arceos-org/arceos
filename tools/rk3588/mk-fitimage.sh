#!/bin/bash

set -e

SOURCE_DIR="$(pwd)"
CUR_DIR="${SOURCE_DIR}/tools/rk3588"

fdt=0
kernel=0
ramdisk=0
resource=0
OUTPUT_TARGET_IMAGE="$1"
src_its_file="$2"
ramdisk_file_path="$3"
kernel_image="$4"
kernel_dtb_file="$5"
target_its_file="${CUR_DIR}/.tmp_its"

if [ -f $target_its_file ]; then
	rm ${target_its_file}
fi

if [ ! -f $src_its_file ]; then
	echo "Not Fount $src_its_file ..."
	exit -1
fi

while read line
do
	############################# generate fdt path
	if [ $fdt -eq 1 ];then
		echo "data = /incbin/(\"$kernel_dtb_file\");" >> $target_its_file
		fdt=0
		continue
	fi
	if echo $line | grep -w "^fdt" |grep -v ";"; then
		fdt=1
		echo "$line" >> $target_its_file
		continue
	fi

	############################# generate kernel image path
	if [ $kernel -eq 1 ];then
		echo "data = /incbin/(\"$kernel_image\");" >> $target_its_file
		kernel=0
		continue
	fi
	if echo $line | grep -w "^kernel" |grep -v ";"; then
		kernel=1
		echo "$line" >> $target_its_file
		continue
	fi

	############################# generate ramdisk path
	if [ -f $ramdisk_file_path ]; then
		if [ $ramdisk -eq 1 ];then
			echo "data = /incbin/(\"$ramdisk_file_path\");" >> $target_its_file
			ramdisk=0
			continue
		fi
		if echo $line | grep -w "^ramdisk" |grep -v ";"; then
			ramdisk=1
			echo "$line" >> $target_its_file
			continue
		fi
	fi

	############################# generate resource path
	if [ $resource -eq 1 ];then
		echo "data = /incbin/(\"$SOURCE_DIR/resource.img\");" >> $target_its_file
		resource=0
		continue
	fi
	if echo $line | grep -w "^resource" |grep -v ";"; then
		resource=1
		echo "$line" >> $target_its_file
		continue
	fi

	echo "$line" >> $target_its_file
done < $src_its_file

${CUR_DIR}/mkimage -f $target_its_file  -E -p 0x800 $OUTPUT_TARGET_IMAGE
