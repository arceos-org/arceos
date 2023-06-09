rm sdcard.img
dd if=/dev/zero of=sdcard.img bs=3M count=1024
mkfs.vfat -F 32 sdcard.img
mkdir -p mnt
sudo mount sdcard.img mnt
# 此处生成的是libc的测例
sudo cp -r ./testcases/junior/* ./mnt/
sudo umount mnt
rm -rf mnt
sudo chmod 777 sdcard.img