use core::fmt;

use x86_64::addr::VirtAddr;
use x86_64::structures::idt::{Entry, HandlerFunc, InterruptDescriptorTable};
use x86_64::structures::DescriptorTablePointer;

const NUM_INT: usize = 256;

/// A wrapper of the Interrupt Descriptor Table (IDT).
#[repr(transparent)]
pub struct IdtStruct {
    table: InterruptDescriptorTable,
}

impl IdtStruct {
    /// Constructs a new IDT struct that filled with entries from
    /// `trap_handler_table`.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        extern "C" {
            #[link_name = "trap_handler_table"]
            static ENTRIES: [extern "C" fn(); NUM_INT];
        }
        let mut idt = Self {
            table: InterruptDescriptorTable::new(),
        };

        let entries = unsafe {
            core::slice::from_raw_parts_mut(
                &mut idt.table as *mut _ as *mut Entry<HandlerFunc>,
                NUM_INT,
            )
        };
        for i in 0..NUM_INT {
            entries[i].set_handler_fn(unsafe { core::mem::transmute(ENTRIES[i]) });
        }
        idt
    }

    /// Returns the IDT pointer (base and limit) that can be used in the `lidt`
    /// instruction.
    pub fn pointer(&self) -> DescriptorTablePointer {
        DescriptorTablePointer {
            base: VirtAddr::new(&self.table as *const _ as u64),
            limit: (core::mem::size_of::<InterruptDescriptorTable>() - 1) as u16,
        }
    }

    /// Loads the IDT into the CPU (executes the `lidt` instruction).
    ///
    /// # Safety
    ///
    /// This function is unsafe because it manipulates the CPU's privileged
    /// states.
    pub unsafe fn load(&'static self) {
        self.table.load();
    }
}

impl fmt::Debug for IdtStruct {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("IdtStruct")
            .field("pointer", &self.pointer())
            .field("table", &self.table)
            .finish()
    }
}
