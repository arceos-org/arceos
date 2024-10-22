# How to run ArceOS on rk3588

1. Use Command `make ARCH=aarch64 PLATFORM=aarch64-rk3588j A=$(pwd)/examples/helloworld kernel` to build the kernel image `boot.img`.
2. Download the [RKDevTool](https://download.t-firefly.com/product/Board/RK3588/Tool/Window/RKDevTool_Release_v3.31.zip). 
    >This tool has only been tested on [Pji's](https://www.pji.net.cn/) Electronic Control Unit of RK3588. Other RK3588 development boards require independent testing.
3. Set the path of `boot.img` in **boot** and connect the RK3588 board.
4. Press the `Run` button to flash the image to the RK3588 board.

![RKDevTool](./figures/RKDevTool3.3.png)