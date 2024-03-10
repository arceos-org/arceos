static BST_DTB_DATA: &[u8] = include_bytes!("./bsta1000b-fada-bus.dtb");

fn setup() {
    unsafe {
        of::init_fdt_ptr(BST_DTB_DATA.as_ptr());
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

#[test]
fn test_pcsi() {
    setup();
    let of_pcsi = of::pcsi();
    assert!(of_pcsi.is_some());
    let of_pcsi = of_pcsi.unwrap();
    assert_eq!(of_pcsi.method(), "smc");
    assert_eq!(of_pcsi.cpu_on().unwrap(), 0xC4000003);
    assert_eq!(of_pcsi.cpu_off().unwrap(), 0x84000002);
    assert_eq!(of_pcsi.cpu_suspend().unwrap(), 0xC4000001);
}
