rm sdcard.img
dd if=/dev/zero of=sdcard.img bs=3M count=2048
mkfs.vfat -F 32 sdcard.img
mkdir -p mnt
sudo mount sdcard.img mnt
<<<<<<< HEAD
# 此处生成的是libc的测例
sudo cp -r ./testcases/libc-static/* ./mnt/
=======
# 根据命令行参数生成对应的测例
sudo cp -r ./testcases/$1/* ./mnt/
>>>>>>> dev
sudo umount mnt
rm -rf mnt
sudo chmod 777 sdcard.img