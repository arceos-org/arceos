use tock_registers::interfaces::Readable;
use tock_registers::register_bitfields;

use crate::trap::handle_page_fault;
use page_table_entry::MappingFlags;

use super::TrapFrame;

register_bitfields! { u64,
    pub ESR_EL1_WRAPPER [
        WNR OFFSET(5)  NUMBITS(1) [
            READ_FAULT = 0,
            WRITE_FAULT = 1,
        ],
        DFSC OFFSET(0)  NUMBITS(5) [
            TTBR_ADDRESS_SIZE_FAULT = 0b00_0000,
            LEVEL1_ADDRESS_SIZE_FAULT = 0b00_0001,
            LEVEL2_ADDRESS_SIZE_FAULT = 0b00_0010,
            LEVEL3_ADDRESS_SIZE_FAULT = 0b00_0011,
            LEVEL0_TRANS_FAULT = 0b00_0100,
            LEVEL1_TRANS_FAULT = 0b00_0101,
            LEVEL2_TRANS_FAULT = 0b00_0110,
            LEVEL3_TRANS_FAULT = 0b00_0111,
            LEVEL0_ACCESS_FLAG_FAULT = 0b00_1000,// When FEAT_LPA2 is implemented
            LEVEL1_ACCESS_FLAG_FAULT = 0b00_1001,
            LEVEL2_ACCESS_FLAG_FAULT = 0b00_1010,
            LEVEL3_ACCESS_FLAG_FAULT = 0b00_1011,
            LEVEL0_PERMISSION_FAULT = 0b00_1100,// When FEAT_LPA2 is implemented
            LEVEL1_PERMISSION_FAULT = 0b00_1101,
            LEVEL2_PERMISSION_FAULT = 0b00_1110,
            LEVEL3_PERMISSION_FAULT = 0b00_1111,
        ]
    ]
}

struct EsrReg(u64);

impl Readable for EsrReg {
    type T = u64;
    type R = ESR_EL1_WRAPPER::Register;

    fn get(&self) -> Self::T {
        self.0
    }
}

fn do_page_fault(far: usize, esr: EsrReg) {
    match esr.read_as_enum(ESR_EL1_WRAPPER::WNR) {
        Some(ESR_EL1_WRAPPER::WNR::Value::READ_FAULT) => {
            info!("EL0 data read abort ");
            handle_page_fault(far.into(), MappingFlags::USER | MappingFlags::READ);
        }
        Some(ESR_EL1_WRAPPER::WNR::Value::WRITE_FAULT) => {
            info!("EL0 data write abort");
            handle_page_fault(far.into(), MappingFlags::USER | MappingFlags::WRITE);
        }
        _ => {
            panic!("impossible value");
        }
    }
}

pub fn el0_ia(far: usize, esr: u64, tf: &TrapFrame) {
    let esr_wrapper = EsrReg(esr);
    match esr_wrapper.read_as_enum(ESR_EL1_WRAPPER::DFSC) {
        Some(ESR_EL1_WRAPPER::DFSC::Value::LEVEL0_TRANS_FAULT)
        | Some(ESR_EL1_WRAPPER::DFSC::Value::LEVEL1_TRANS_FAULT)
        | Some(ESR_EL1_WRAPPER::DFSC::Value::LEVEL2_TRANS_FAULT)
        | Some(ESR_EL1_WRAPPER::DFSC::Value::LEVEL3_TRANS_FAULT) => {
            info!("EL0 instruction fault");
            handle_page_fault(far.into(), MappingFlags::USER | MappingFlags::EXECUTE);
        }
        Some(ESR_EL1_WRAPPER::DFSC::Value::LEVEL0_PERMISSION_FAULT)
        | Some(ESR_EL1_WRAPPER::DFSC::Value::LEVEL1_PERMISSION_FAULT)
        | Some(ESR_EL1_WRAPPER::DFSC::Value::LEVEL2_PERMISSION_FAULT)
        | Some(ESR_EL1_WRAPPER::DFSC::Value::LEVEL3_PERMISSION_FAULT) => {
            info!("EL0 permisiion fault");
            handle_page_fault(far.into(), MappingFlags::USER | MappingFlags::EXECUTE);
        }
        _ => {
            panic!(
                "Unknown EL0 ia {:#x?} esr: {:#x?}  tf {:#x?}",
                far,
                esr_wrapper.get(),
                tf
            );
        }
    }
}

pub fn el0_da(far: usize, esr: u64, tf: &TrapFrame) {
    let esr_wrapper = EsrReg(esr);
    match esr_wrapper.read_as_enum(ESR_EL1_WRAPPER::DFSC) {
        Some(ESR_EL1_WRAPPER::DFSC::Value::LEVEL3_TRANS_FAULT) => {
            info!("EL0 data abort  l3 fault");
            do_page_fault(far, esr_wrapper);
        }
        Some(ESR_EL1_WRAPPER::DFSC::Value::LEVEL0_PERMISSION_FAULT)
        | Some(ESR_EL1_WRAPPER::DFSC::Value::LEVEL1_PERMISSION_FAULT)
        | Some(ESR_EL1_WRAPPER::DFSC::Value::LEVEL2_PERMISSION_FAULT)
        | Some(ESR_EL1_WRAPPER::DFSC::Value::LEVEL3_PERMISSION_FAULT) => {
            info!("EL0 permisiion fault");
            do_page_fault(far, esr_wrapper);
        }
        _ => {
            panic!(
                "Unknown EL0 da {:#x?} esr: {:#x?} tf {:#x?}",
                far,
                esr_wrapper.get(),
                tf
            );
        }
    }
}
