rm disk.img
dd if=/dev/zero of=disk.img bs=3M count=1024
mkfs.vfat -F 32 disk.img
mkdir -p mnt
sudo mount disk.img mnt
# 此处生成的是初赛的测例
sudo cp -r ./testcases/junior/* ./mnt/
sudo umount mnt
rm -rf mnt
sudo chmod 777 disk.img