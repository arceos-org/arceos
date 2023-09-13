gcc测例运行方法：
1. 下载riscv64-linux-musl-native库，内含gcc可执行文件，源码网址为[(musl.cc)](https://musl.cc/riscv64-linux-musl-native.tgz)，将其解压到testcases目录下。

2. 在本目录下执行如下指令

   ```shell
   $ ./gcc.sh
   ```

   在gcc文件夹生成了gcc测例，另外gcc文件夹原有lib文件，带有动态链接器，请勿删除

3. 在根目录下执行指令

   ```shell
   $ ./build_img.sh gcc
   ```

   即可生成gcc测例镜像。
