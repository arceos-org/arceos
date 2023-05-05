rm disk.img
dd if=/dev/zero of=disk.img bs=3M count=1024
mkfs.vfat -F 32 disk.img
mkdir -p mnt
sudo mount disk.img mnt
sudo cp -r ./testcases/* ./mnt/
sudo umount mnt
rm -rf mnt
sudo chmod 777 disk.img