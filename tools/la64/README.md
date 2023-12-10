# Run Arceos on loongarch64

1. install `qemu`

   ```
   copy la-qemu.sh to your dir and run it
   ```

2. install `gcc `

   ```
   wget https://github.com/foxsen/qemu-loongarch-runenv/releases/download/toolchain/loongarch64-clfs-2021-12-18-cross-tools-gcc-full.tar.xz
   tar zxf loongarch64-clfs-2021-12-18-cross-tools-gcc-full.tar.xz
   # exec below command in bash OR add below info in ~/.bashrc
   ```

3. install `gdb`

   ```
   copy la-gdb.sh to your dir and run it
   ```

4. install `rust`

   ```
   rust version >= nightly 1.74
   ```

In order to run qemu, some firmware is also required. You can clone the repository [qemu-loongarch-runenv](https://github.com/foxsen/qemu-loongarch-runenv) and copy the `efi-virtio.rom` and `loongarch_bios_0310_{debug}.bin`  files to the project root directory.

