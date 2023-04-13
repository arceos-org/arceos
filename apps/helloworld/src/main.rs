#![no_std]
#![no_main]

#[no_mangle]
fn main() {
    // libax::println!("!!!");
    let output = "hello world!\n".as_bytes();
    // libax::sys_write(0, output);
    libax::sys_write(1, output);
    libax::sys_exit(0);
}
