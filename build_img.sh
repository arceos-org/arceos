rm disk.img
dd if=/dev/zero of=disk.img bs=4M count=30
mkfs.vfat -F 32 disk.img
mkdir -p mnt
sudo mount disk.img mnt
# 根据命令行参数生成对应的测例
sudo cp -r ./testcases/$1/* ./mnt/
sudo umount mnt
rm -rf mnt
sudo chmod 777 disk.img