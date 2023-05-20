# Week13

## 决赛环境配置

* 环境为WSL+Ubuntu22.04-LTS

* 配置必要依赖

  ```shell
  sudo apt install build-essential
  sudo apt install musl-tools
  sudo apt-get install libncurses5-dev libncursesw5-dev
  ```

* 配置riscv64-linux-musl-cross

  * 官网下载https://musl.cc/riscv64-linux-musl-cross.tgz

  * 解压压缩包

    ```shell
    tar zxvf musl-1.2.1.tar.gz
    ```

  * 运行配置文件

    ```shell
    ./configure
    make
    sudo make install
    ```

  * 若无法正常运行musl-gcc，需要配置环境变量。

  * 此后便可以正常使用`musl-gcc`代替`gcc`进行编译。

* 动态链接库加载：

  * 根据`README.md`，它需要一个动态链接库，原本是`/lib/ld-musl-riscv64-sf.so.1`，但它实质上是一个链接文件，指向了`libc.so`，所以将`libc.so`直接手动加入到文件镜像中。

    > `libc.so`通过由maturin的文件镜像获取

  * 内核需要手动建立从`/lib/ld-musl-riscv64-sf.so.1`到`libc.so`的链接，即用`libc.so`代替`/lib/ld-musl-riscv64-sf.so.1`。

    > 原因：fat32不支持符号链接，elf默认请求的是`/lib/ld-musl-riscv64-sf.so.1`，此时通过内核手动建立链接让其转发给`libc.so`。

* 编译可执行文件

  * 在`libc-test`文件夹下的`makefile`修改`MUSL_LIB`和`PREFIX`为当前`musl`的交叉编译版本。本机编译版本为`riscv64-linux-musl-gcc`。

  * 问题：本机版本为`11.2.1`，使用make编译会进行报错：

    ![image-20230520211208912](C:\Users\zyj57\AppData\Roaming\Typora\typora-user-images\image-20230520211208912.png)

    出错原因：`dso.obj`是一个动态链接库，加入了`-static`编译参数。会导致错误。

    

    闭浩扬学长版本为`8.2.0`，此时make编译不会报错。



