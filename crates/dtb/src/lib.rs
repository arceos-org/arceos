//! Some useful interfaces for device tree.

#![no_std]
use fdt_rs::{
    base::{DevTree, DevTreeNode, DevTreeProp},
    prelude::{FallibleIterator, PropReader},
};
use lazy_init::LazyInit;

/// The device Tree.
static TREE: LazyInit<DevTree> = LazyInit::new();

/// Init device tree from given address.
/// # Safety
///
/// Callers of this method the must guarantee the following:
///
/// - The passed address is 32-bit aligned.
pub unsafe fn init(dtb: *const u8) {
    TREE.init_by(DevTree::from_raw_pointer(dtb).unwrap());
}

/// A node on the device tree.
pub struct DeviceNode<'a>(DevTreeNode<'a, 'static>);

/// A prop of a node.
pub struct DeviceProp<'a>(DevTreeProp<'a, 'static>);

impl<'a> DeviceNode<'a> {
    /// Find a node's prop with given name(may not exist).
    pub fn find_prop(&'a self, name: &str) -> Option<DeviceProp<'a>> {
        self.0
            .props()
            .filter(|p| p.name().map(|s| s == name))
            .next()
            .unwrap()
            .map(DeviceProp)
    }

    /// Find a node's prop with given name(must exist).
    pub fn prop(&'a self, name: &str) -> DeviceProp<'a> {
        self.find_prop(name).unwrap()
    }
}

impl<'a> DeviceProp<'a> {
    /// Assume the prop is a u32 array. Get an element.
    pub fn u32(&self, index: usize) -> u32 {
        self.0.u32(index).unwrap()
    }

    /// Assume the prop is a u64 array. Get an element.
    pub fn u64(&self, index: usize) -> u64 {
        self.0.u64(index).unwrap()
    }

    /// Assume the prop is a str. Get the whole str.
    pub fn str(&self) -> &'static str {
        self.0.str().unwrap()
    }
}

/// Find the first node with given compatible(may not exist).
pub fn compatible_node(compatible: &str) -> Option<DeviceNode> {
    TREE.compatible_nodes(compatible)
        .next()
        .unwrap()
        .map(DeviceNode)
}

/// Find the first node with given name(may not exist).
pub fn get_node(name: &str) -> Option<DeviceNode> {
    TREE.nodes()
        .filter(|n| n.name().map(|s| s == name))
        .next()
        .unwrap()
        .map(DeviceNode)
}

/// Do something for all devices with given type.
pub fn devices<F>(device_type: &str, f: F)
where
    F: Fn(DeviceNode),
{
    TREE.nodes()
        .filter_map(|n| {
            let n = DeviceNode(n);
            Ok(
                if n.find_prop("device_type").map(|p| p.str()) == Some(device_type) {
                    Some(n)
                } else {
                    None
                },
            )
        })
        .for_each(|n| {
            f(n);
            Ok(())
        })
        .unwrap();
}

/// Do something for all nodes with given compatible.
pub fn compatible_nodes<F>(compatible: &str, mut f: F)
where
    F: FnMut(DeviceNode),
{
    TREE.compatible_nodes(compatible)
        .for_each(|n| {
            f(DeviceNode(n));
            Ok(())
        })
        .unwrap();
}
