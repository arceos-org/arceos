//! A pure-Rust #![no_std] crate for parsing Flattened Devicetrees, 
//! with the goal of having a very ergonomic and idiomatic API.

#![no_std]

pub const FDT_SIZE:usize = 0x200000;  // 2MB

pub struct MachineFdt<'a>(fdt::Fdt<'a>);

static mut MY_FDT_PTR: Option<*const u8> = None;

lazy_static::lazy_static! {
    static ref MY_MACHINE_FDT: MachineFdt<'static> = 
        unsafe {init_from_ptr(MY_FDT_PTR.unwrap())};
}

pub unsafe fn init_fdt_ptr(virt_addr: *const u8) {
    MY_FDT_PTR = Some(virt_addr);
}

/// Init the DTB root, call after dtb finish mapping
unsafe fn init_from_ptr(virt_addr: *const u8) -> MachineFdt<'static> {
    MachineFdt(fdt::Fdt::from_ptr(virt_addr).unwrap())
}

/// Root Node found model or first compatible
pub fn machin_name() -> &'static str {
    let root_node = MY_MACHINE_FDT.0.root();
    let model = root_node
        .properties()
        .find(|p| p.name == "model")
        .and_then(|p| core::str::from_utf8(p.value).map(|s| s.trim_end_matches('\0')).ok());
    if model != None {
        return model.unwrap();
    }

    return root_node.compatible().first();
}

/// Searches for a node which contains a `compatible` property and contains
/// one of the strings inside of `with`
pub fn find_compatible_node(with: &'static[&'static str]) -> impl Iterator<Item = fdt::node::FdtNode<'static, 'static>> {
    MY_MACHINE_FDT.0.all_nodes().filter(|n| {
       n.compatible().and_then(|compats| 
           compats.all().find(|c| with.contains(c))).is_some()})
}

pub fn bootargs() -> Option<&'static str> {
    MY_MACHINE_FDT.0.chosen().bootargs()
}
