# Introduction

This article describes a way to debug jtags via openocd, gdb, jlink debugger. 

## Requirement 

### Resources

1. Authors using [H-JLINK v9 type c Universal ARM Downloader](https://m.tb.cn/h.5FpduM7jUbbFlzo?tk=Z6t5WlsfPtl), 
but theoretically, you can use whatever you want. 

2. [Serial USB to TTL CH340 Module](https://www.amazon.com/HiLetgo-Module-Microcontroller-Download-Serial/dp/B00LZV1G6K/ref=sr_1_3?dib=eyJ2IjoiMSJ9.EVDg6VSjpenXHkAOIddkejC8NrNLBaiI9YKosxxcvxsvWHCkJuYWT97oslmx7iE-il7I7ilkI07pfXYrJnjb0-gM8hu4y8_hMEVA7hiUtPZtjhovoAeF0-L7rM0xTe-hdNscYjbIspct3yjOtYSF9QPNFmr9XmeC5Os2gCQxZihglIJJDxUWWAhJL_MNl06dDKZnk82pkR_p09laqdfg0nFMwJwdxLDObHv3gzDHWNk.pvOBDJ9aVLFwecXlCYuMONK54Z_7sxnzAvdO71qkHWI&dib_tag=se&keywords=CH340&qid=1709218438&sr=8-3)

3. Some Female to Female [Dupont Wire](https://www.amazon.com/Elegoo-EL-CP-004-Multicolored-Breadboard-arduino/dp/B01EV70C78/ref=sr_1_3?dib=eyJ2IjoiMSJ9.OCLDs3D5By4QvSSJfVxcRa7LFdoHpv56YLqS9wbJRIGaY_r5UKFkopHdBRu0aVmyYfSaH77oX0ure59RTu2R0GWeOUm8DEzRUHiLTYnKqPa02peSrC0JWZMUQPaErE40BeYQpDl0ywu9vg7zI1gHJWxdYtOgrehyUhiT9G9657pN73jvrY3Vd5RrBH9-5aAYEDKpN_P1gS48Yqv9n7S3efD7AKdAsgYsLsN1QLBFeyI.VxVp4ZfND0S73SSXYiJlh6KJD8GRdD2Pn2LHUrCHSj4&dib_tag=se&keywords=DuPont+line&qid=1709218493&sr=8-3)

4. A laptop

### Preliminary preparation 

1. Connect CH304 from [bcm2711 pin 8,10,12](https://datasheets.raspberrypi.com/rpi4/raspberry-pi-4-reduced-schematics.pdf) (Rx to Tx, Tx to Rx, GND to GND).
    By the ways, power indicator closest to 1.

2. Connect H-JLINK based on the picture below:

![connect_jtag_debugger](./figures/image_jtag_connected.jpg)
<table>
    <thead> <tr>
            <th>GPIO #</th> <th>Name</th> <th>JTAG #</th> <th>Note</th> <th width="60%">Diagram</th>
    </tr> </thead>
    <tbody>
        <tr>
            <td></td>
            <td>VTREF</td>
            <td>1</td>
            <td>to 3.3V</td>
            <td rowspan="8"><img src="../doc/09_wiring_jtag.png"></td>
        </tr>
        <tr>
            <td></td>
            <td>GND</td>
            <td>4</td>
            <td>to GND</td>
        </tr>
        <tr>
            <td>22</td>
            <td>TRST</td>
            <td>3</td>
            <td></td>
        </tr>
        <tr>
            <td>26</td>
            <td>TDI</td>
            <td>5</td>
            <td></td>
        </tr>
        <tr>
            <td>27</td>
            <td>TMS</td>
            <td>7</td>
            <td></td>
        </tr>
        <tr>
            <td>25</td>
            <td>TCK</td>
            <td>9</td>
            <td></td>
        </tr>
        <tr>
            <td>23</td>
            <td>RTCK</td>
            <td>11</td>
            <td></td>
        </tr>
        <tr>
            <td>24</td>
            <td>TDO</td>
            <td>13</td>
            <td></td>
        </tr>
    </tbody>
</table>
<p align="center"><img src="./figures/draw_jtag_connected.jpg" width="50%"></p>
*thanks for andre-richter provide this sections.*

3. Try to test whether chainboot works properly 
    1. go to tools/raspi4/chainloader
    2. run `make clean && make`, it should generate a kernel8.img under this directory.
    if everything right, the image file should be 8576 via `ls -al`.
    3. move this image into your sd card.
    4. check if your sd card has contians following file 
    [start4.elf](https://raw.githubusercontent.com/raspberrypi/firmware/master/boot/start4.elf), 
    [fixup4.dat](https://github.com/raspberrypi/firmware/raw/master/boot/fixup4.dat), 
    [bcm2711-rpi-4-b.dtb](https://raw.githubusercontent.com/raspberrypi/firmware/master/boot/bcm2711-rpi-4-b.dtb)
    5. just run `make A=apps/helloworld PLATFORM=aarch64-raspi4 chainboot`, then should display this image.

    <table>
    <thead> <tr>
        <th>Minipush 1.0<br><br><br><br><br><br>[MP] ⏳ Waiting for /dev/ttyUSB0</th>
        <th>[MP] ✅ Serial connected<br><br><br>[MP]  Please power the target now</th>
        <th></th>
        <th></th>
        <th></th>
    </tr> </thead>
    <tbody>
      <tr>
        <td>This means that CH340 is not connected or connected in some other way</td>
        <td>connection is successful<br></td>
        <td></td> <td></td> <td></td> </tr>
      <tr>
        <td>maybe /dev/ttyUSB1 or something else</td>
        <td></td> <td></td> <td></td> <td></td> </tr>
    </tbody>
    </table>

    After power up the board:

    ```
           
     __  __ _      _ _                 _
    |  \/  (_)_ _ (_) |   ___  __ _ __| |
    | |\/| | | ' \| | |__/ _ \/ _` / _` |
    |_|  |_|_|_||_|_|____\___/\__,_\__,_|

               Raspberry Pi 4            

    [ML] Requesting binary
    [MP] ⏩ Pushing 36 KiB =========================================🦀 100% 0 KiB/s Time: 00:00:00
    [ML] Loaded! Executing the payload now

    @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
    @ You're using chainboot image    . @
    @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@

           d8888                            .d88888b.   .d8888b.
          d88888                           d88P" "Y88b d88P  Y88b
         d88P888                           888     888 Y88b.
        d88P 888 888d888  .d8888b  .d88b.  888     888  "Y888b.
       d88P  888 888P"   d88P"    d8P  Y8b 888     888     "Y88b.
      d88P   888 888     888      88888888 888     888       "888
     d8888888888 888     Y88b.    Y8b.     Y88b. .d88P Y88b  d88P
    d88P     888 888      "Y8888P  "Y8888   "Y88888P"   "Y8888P"

    arch = aarch64
    platform = aarch64-raspi4
    target = aarch64-unknown-none-softfloat
    smp = 1
    build_mode = release
    log_level = warn

    Hello, world!
    ```

## Run

### Preliminary preparation 

    1. go to tools/raspi4/chainloader
    2. run `make clean && make JTAG=y`, it should generate a kernel8.img under this directory.
    if everything right, the image file should be 8576 via `ls -al`.
    3. move this image into your sd card.
    4. check if your sd card has contians following file 
    [start4.elf](https://raw.githubusercontent.com/raspberrypi/firmware/master/boot/start4.elf), 
    [fixup4.dat](https://github.com/raspberrypi/firmware/raw/master/boot/fixup4.dat), 
    [bcm2711-rpi-4-b.dtb](https://raw.githubusercontent.com/raspberrypi/firmware/master/boot/bcm2711-rpi-4-b.dtb)

### Start Debugging

    1. just run `make A=apps/helloworld PLATFORM=aarch64-raspi4 chainboot` and Power up the board., then should display this image.

    ```
    Minipush 1.0

    [MP] ✅ Serial connected
    [MP] 🔌 Please power the target now

     __  __ _      _ _                 _
    |  \/  (_)_ _ (_) |   ___  __ _ __| |
    | |\/| | | ' \| | |__/ _ \/ _` / _` |
    |_|  |_|_|_||_|_|____\___/\__,_\__,_|

               Raspberry Pi 4            

    [ML] Requesting binary
    [MP] ⏩ Pushing 36 KiB =========================================🦀 100% 0 KiB/s Time: 00:00:00
    [ML] Loaded! Executing the payload now

    @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
    @ You're using a JTAG debug image.  @
    @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
    @ 1. open openocd, gdb              @
    @ 2. target extended-remote :3333;  @
    @ 3. set $pc=0x80000                @
    @ 4. break rust_entry/others        @
    @ 5. break $previous_addr           @
    @ 6. delete 1                       @
    @ 7. load                           @
    @ 8. continue                       @
    @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@

    ```

    *the following guidelines is basically like previous datasheets, In fact, if you're a senior developer, skip the following. XD*
    3. My personal suggestions is using zellij, but you could choice what ever you want.
    4. A: Keeping this miniload running (just don't terminate it) in terminal A.
    5. B: run `make A=apps/helloworld PLATFORM=aarch64-raspi4 openocd`, 
    the windows should display following, but it doesn't matter, we don't need to care about this.

    ```
    $ make A=apps/helloworld PLATFORM=aarch64-raspi4 openocd  

    Launching OpenOCD
    [sudo] password for jacky: 
    Open On-Chip Debugger 0.11.0+dev-g1ad6ed3 (2021-12-02-20:10)
    Licensed under GNU GPL v2
    For bug reports, read
            http://openocd.org/doc/doxygen/bugs.html
    DEPRECATED! use 'adapter speed' not 'adapter_khz'
    Warn : DEPRECATED! use '-baseaddr' not '-ctibase'
    Warn : DEPRECATED! use '-baseaddr' not '-ctibase'
    Warn : DEPRECATED! use '-baseaddr' not '-ctibase'
    Warn : DEPRECATED! use '-baseaddr' not '-ctibase'
    Info : Listening on port 6666 for tcl connections
    Info : Listening on port 4444 for telnet connections
    Info : J-Link V9 compiled May  7 2021 16:26:12
    Info : Hardware version: 9.60
    Info : VTarget = 3.311 V
    Info : clock speed 1000 kHz
    Info : JTAG tap: rpi4.tap tap/device found: 0x4ba00477 (mfg: 0x23b (ARM Ltd), part: 0xba00, ver: 0x4)
    Info : rpi4.core0: hardware has 6 breakpoints, 4 watchpoints
    Info : rpi4.core1: hardware has 6 breakpoints, 4 watchpoints
    Info : rpi4.core2: hardware has 6 breakpoints, 4 watchpoints
    Info : rpi4.core3: hardware has 6 breakpoints, 4 watchpoints
    Info : starting gdb server for rpi4.core0 on 3333
    Info : Listening on port 3333 for gdb connections
    Info : starting gdb server for rpi4.core1 on 3334
    Info : Listening on port 3334 for gdb connections
    Info : starting gdb server for rpi4.core2 on 3335
    Info : Listening on port 3335 for gdb connections
    Info : starting gdb server for rpi4.core3 on 3336
    Info : Listening on port 3336 for gdb connections

    ```

    6. C: run `make A=apps/helloworld PLATFORM=aarch64-raspi4 gdb` in terminal C.
    7. You are now in GDB, but just don't start your debug immediately.
    Because the use of minipush script, we could simple push our image to board before power up the board.
    Like a double-edged sword. It also constrains our behavior, 
    which it's incapable for us to power up the board then use some kinds like halt command to halt the CPU. 
    (I think it can be done through script modification, but now there is no time to study and change ruby for a trivial upgrade.)
    So we add a dead loop at the end of the image we generate lastest.
    
    ```gdb
    (gdb) target extended-remote :3333  // connect to openocd gdb serve
                                        // you should see $pc=0x2080db4 
    (gdb) set $pc=0x80000   // The following behavior is unnecessary and can be debugged normally.
    (gdb) monitor poll      // MMU disable 
    (gdb) break rust_entry  // Start debug module/axhal/ ... the location is VADDR will case error,
                            // you should use b *0x81a90 (manally remove 0xffff ... from output)
    (gdb) b *0x81a90
    (gdb) break rust_main   // as previous
    (gdb) b *0x82888
    (gdb) delete 1 3
    (gdb) continue          // first stop at rust_entry
    (gdb) continue          // second stop at rust_main
    (gdb) monitor poll      // MMU enable (in rust_entry)
    ```

## Reference

1. [rust-embedded/rust-raspberrypi-OS-tutorials](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials)
2. [Raspberry Pi 4をJTAGデバッグしてみる（FTDI C232HM-DDHSL-0使用）](https://hikalium.hatenablog.jp/entry/2021/07/18/214013)
3. [Rust Raspberry Pi OS tutorials 08 HW debug JTAG by hikalium 2021-09-20](https://www.youtube.com/watch?v=6ULvzK1Drgo&t=21s)
4. [my record](https://bitbucket.org/jackyliu16/blog/src/master/content/jtag-load-failure-debug.cn.md)
4. openocd docs, bcm2711 docs, gdb docs ... etc.


