
static MY_DTB: &[u8] = include_bytes!("./bsta1000b-fada-bus.dtb");

fn setup() {
    unsafe {
        of::init_fdt_ptr(MY_DTB.as_ptr());
    }
}

#[test]
fn test_module() {
    setup();
    let model = of::machin_name();
    assert_eq!(model, "BST A1000B FAD-A");
}

#[test]
fn test_find_compatible() {
    const CONSOLE_COMPATIABLE: &'static [&'static str] = &["snps,dw-apb-uart"];
    const CONSOLE_COUNT: usize = 4;
    setup();
    let console_node = of::find_compatible_node(CONSOLE_COMPATIABLE);
    assert_eq!(console_node.count(), CONSOLE_COUNT);
}
