#[doc = r" Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - endpoint 0 register"]
    pub usb_ep0r: USB_EP0R,
    #[doc = "0x04 - endpoint 1 register"]
    pub usb_ep1r: USB_EP1R,
    #[doc = "0x08 - endpoint 2 register"]
    pub usb_ep2r: USB_EP2R,
    #[doc = "0x0c - endpoint 3 register"]
    pub usb_ep3r: USB_EP3R,
    #[doc = "0x10 - endpoint 4 register"]
    pub usb_ep4r: USB_EP4R,
    #[doc = "0x14 - endpoint 5 register"]
    pub usb_ep5r: USB_EP5R,
    #[doc = "0x18 - endpoint 6 register"]
    pub usb_ep6r: USB_EP6R,
    #[doc = "0x1c - endpoint 7 register"]
    pub usb_ep7r: USB_EP7R,
    _reserved0: [u8; 32usize],
    #[doc = "0x40 - control register"]
    pub usb_cntr: USB_CNTR,
    #[doc = "0x44 - interrupt status register"]
    pub istr: ISTR,
    #[doc = "0x48 - frame number register"]
    pub fnr: FNR,
    #[doc = "0x4c - device address"]
    pub daddr: DADDR,
    #[doc = "0x50 - Buffer table address"]
    pub btable: BTABLE,
}
#[doc = "endpoint 0 register"]
pub struct USB_EP0R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "endpoint 0 register"]
pub mod usb_ep0r;
#[doc = "endpoint 1 register"]
pub struct USB_EP1R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "endpoint 1 register"]
pub mod usb_ep1r;
#[doc = "endpoint 2 register"]
pub struct USB_EP2R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "endpoint 2 register"]
pub mod usb_ep2r;
#[doc = "endpoint 3 register"]
pub struct USB_EP3R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "endpoint 3 register"]
pub mod usb_ep3r;
#[doc = "endpoint 4 register"]
pub struct USB_EP4R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "endpoint 4 register"]
pub mod usb_ep4r;
#[doc = "endpoint 5 register"]
pub struct USB_EP5R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "endpoint 5 register"]
pub mod usb_ep5r;
#[doc = "endpoint 6 register"]
pub struct USB_EP6R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "endpoint 6 register"]
pub mod usb_ep6r;
#[doc = "endpoint 7 register"]
pub struct USB_EP7R {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "endpoint 7 register"]
pub mod usb_ep7r;
#[doc = "control register"]
pub struct USB_CNTR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "control register"]
pub mod usb_cntr;
#[doc = "interrupt status register"]
pub struct ISTR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "interrupt status register"]
pub mod istr;
#[doc = "frame number register"]
pub struct FNR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "frame number register"]
pub mod fnr;
#[doc = "device address"]
pub struct DADDR {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "device address"]
pub mod daddr;
#[doc = "Buffer table address"]
pub struct BTABLE {
    register: ::vcell::VolatileCell<u32>,
}
#[doc = "Buffer table address"]
pub mod btable;
