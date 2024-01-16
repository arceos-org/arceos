use fdt::standard_nodes::MemoryRegion;

pub struct Memory {
    pub(crate) node: fdt::node::FdtNode<'static, 'static>,
}

impl Memory {
    /// Returns an iterator over all of the available memory regions
    pub fn regions(self) -> impl Iterator<Item = MemoryRegion> +'static {
        self.node.reg().unwrap()
    }
}
