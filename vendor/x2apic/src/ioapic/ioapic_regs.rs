use core::ptr::{self, Unique};

#[derive(Debug)]
pub struct IoApicRegisters {
    ioregsel: Unique<u32>,
    ioregwin: Unique<u32>,
}

impl IoApicRegisters {
    pub unsafe fn new(base_addr: u64) -> Self {
        let base = base_addr as *mut u32;

        IoApicRegisters {
            ioregsel: Unique::new_unchecked(base.offset(0)),
            ioregwin: Unique::new_unchecked(base.offset(4)),
        }
    }

    pub unsafe fn read(&mut self, selector: u32) -> u32 {
        ptr::write_volatile(self.ioregsel.as_ptr(), selector);
        ptr::read_volatile(self.ioregwin.as_ptr())
    }

    pub unsafe fn write(&mut self, selector: u32, value: u32) {
        ptr::write_volatile(self.ioregsel.as_ptr(), selector);
        ptr::write_volatile(self.ioregwin.as_ptr(), value);
    }

    pub unsafe fn set(&mut self, selector: u32, mask: u32) {
        ptr::write_volatile(self.ioregsel.as_ptr(), selector);

        let val = ptr::read_volatile(self.ioregwin.as_ptr());
        ptr::write_volatile(self.ioregwin.as_ptr(), val | mask);
    }

    pub unsafe fn clear(&mut self, selector: u32, mask: u32) {
        ptr::write_volatile(self.ioregsel.as_ptr(), selector);

        let val = ptr::read_volatile(self.ioregwin.as_ptr());
        ptr::write_volatile(self.ioregwin.as_ptr(), val & !mask);
    }
}

// Register selectors
pub const ID: u32 = 0x00;
pub const VERSION: u32 = 0x01;
pub const ARBITRATION: u32 = 0x02;
pub const TABLE_BASE: u32 = 0x10;
