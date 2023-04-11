mod bindings;

#[no_mangle]
extern "C" fn ax_print() {
    println!("ax_print");
}

fn main() {
    println!("Hello, world!");
    unsafe {
        bindings::lwip_init();
    }
}
