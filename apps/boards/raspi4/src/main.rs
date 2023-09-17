#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;
use arm_pl011::pl011::Pl011Uart;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    println!("Hello, world!");

    fn delay(seconds: u64) {
        for i in 1..seconds+1 {
            println!("{} ", i);

            fn fibonacci_recursive(n: u64) -> u64 {
                if n == 0 {
                    return 0;
                }
                if n == 1 {
                    return 1;
                }
                return fibonacci_recursive(n - 1) + fibonacci_recursive(n - 2);
            }

            fibonacci_recursive(34 + (i % 2));
        }
    }

    let uart_base = 0xffff_0000_fe20_1000 as *mut u8;
    let mut uart = Pl011Uart::new(uart_base);

    println!("start");
    {
        // 鸣笛：0xFF_FC_05_02_60_00_67
        uart.putchar(0xff);
        uart.putchar(0xfc);
        uart.putchar(0x05);
        uart.putchar(0x02);
        uart.putchar(0x60);
        uart.putchar(0x00);
        uart.putchar(0x67);
    }
    delay(1);

    loop {
        println!("forward");
        {
            // 前进：0xff_fc_07_11_01_01_64_00_7e
            uart.putchar(0xff);
            uart.putchar(0xfc);
            uart.putchar(0x07);
            uart.putchar(0x11);
            uart.putchar(0x01);
            uart.putchar(0x01);
            uart.putchar(0x64);
            uart.putchar(0x00);
            uart.putchar(0x7e);
        }
        delay(6);

        println!("stop");
        {
            // 停止：0xff_fc_07_11_01_00_00_00_19
            uart.putchar(0xff);
            uart.putchar(0xfc);
            uart.putchar(0x07);
            uart.putchar(0x11);
            uart.putchar(0x01);
            uart.putchar(0x00);
            uart.putchar(0x00);
            uart.putchar(0x00);
            uart.putchar(0x19);
        }
        delay(1);

        println!("turn right");
        {
            // // 左转：0xff_fc_07_11_01_05_64_00_82
            // uart.putchar(0xff);
            // uart.putchar(0xfc);
            // uart.putchar(0x07);
            // uart.putchar(0x11);
            // uart.putchar(0x01);
            // uart.putchar(0x05);
            // uart.putchar(0x64);
            // uart.putchar(0x00);
            // uart.putchar(0x82);

            // 右转：0xff_fc_07_11_01_06_64_00_83
            uart.putchar(0xff);
            uart.putchar(0xfc);
            uart.putchar(0x07);
            uart.putchar(0x11);
            uart.putchar(0x01);
            uart.putchar(0x06);
            uart.putchar(0x64);
            uart.putchar(0x00);
            uart.putchar(0x83);
        }
        delay(1);
        // println!("forward");
        // {
        //     // 前进：0xff_fc_07_11_01_01_64_00_7e
        //     uart.putchar(0xff);
        //     uart.putchar(0xfc);
        //     uart.putchar(0x07);
        //     uart.putchar(0x11);
        //     uart.putchar(0x01);
        //     uart.putchar(0x01);
        //     uart.putchar(0x64);
        //     uart.putchar(0x00);
        //     uart.putchar(0x7e);
        // }
        // delay(4);

        println!("stop");
        {
            // 停止：0xff_fc_07_11_01_00_00_00_19
            uart.putchar(0xff);
            uart.putchar(0xfc);
            uart.putchar(0x07);
            uart.putchar(0x11);
            uart.putchar(0x01);
            uart.putchar(0x00);
            uart.putchar(0x00);
            uart.putchar(0x00);
            uart.putchar(0x19);
        }
        delay(1);

        println!("turn right");
        {
            // // 左转：0xff_fc_07_11_01_05_64_00_82
            // uart.putchar(0xff);
            // uart.putchar(0xfc);
            // uart.putchar(0x07);
            // uart.putchar(0x11);
            // uart.putchar(0x01);
            // uart.putchar(0x05);
            // uart.putchar(0x64);
            // uart.putchar(0x00);
            // uart.putchar(0x82);

            // 右转：0xff_fc_07_11_01_06_64_00_83
            uart.putchar(0xff);
            uart.putchar(0xfc);
            uart.putchar(0x07);
            uart.putchar(0x11);
            uart.putchar(0x01);
            uart.putchar(0x06);
            uart.putchar(0x64);
            uart.putchar(0x00);
            uart.putchar(0x83);
        }
        delay(1);
    }

}
