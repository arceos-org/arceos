use crate::parsing::BigEndianU32;
use fdt::standard_nodes::MemoryRegion;

#[derive(Clone, Copy)]
pub struct Memory {
    pub(crate) node: fdt::node::FdtNode<'static, 'static>,
}

impl Memory {
    /// Returns an iterator over all of the available memory regions
    pub fn regions(self) -> impl Iterator<Item = MemoryRegion> + 'static {
        self.node.reg().unwrap()
    }
}

#[derive(Clone, Copy)]
pub struct Pcsi {
    pub(crate) node: fdt::node::FdtNode<'static, 'static>,
}

impl Pcsi {
    /// `compatible` property
    pub fn compatible(self) -> &'static str {
        self.node.compatible().unwrap().first()
    }

    /// `method` property
    pub fn method(self) -> &'static str {
        self.node
            .properties()
            .find(|p| p.name == "method")
            .and_then(|p| {
                core::str::from_utf8(p.value)
                    .map(|s| s.trim_end_matches('\0'))
                    .ok()
            })
            .unwrap()
    }
    /// Optional`cpu_suspend` property
    pub fn cpu_suspend(self) -> Option<u32> {
        self.node
            .properties()
            .find(|p| p.name == "cpu_suspend")
            .map(|p| BigEndianU32::from_bytes(p.value).unwrap().get())
    }

    /// Optional`cpu_on` property
    pub fn cpu_on(self) -> Option<u32> {
        self.node
            .properties()
            .find(|p| p.name == "cpu_on")
            .map(|p| BigEndianU32::from_bytes(p.value).unwrap().get())
    }

    /// Optional`cpu_off` property
    pub fn cpu_off(self) -> Option<u32> {
        self.node
            .properties()
            .find(|p| p.name == "cpu_off")
            .map(|p| BigEndianU32::from_bytes(p.value).unwrap().get())
    }

    /// Optional`migrate` property
    pub fn migrate(self) -> Option<u32> {
        self.node
            .properties()
            .find(|p| p.name == "migrate")
            .map(|p| BigEndianU32::from_bytes(p.value).unwrap().get())
    }
}
