use super::super::tlb::TLBREra;
use super::csr::Register;
use super::csr::CSR_ESTAT;
use super::ecfg::Ecfg;
use bit_field::BitField;
use core::arch::asm;
// 该寄存器记录例外的状态信息，包括所触发例外的一二级编码，以及各中断的状态
#[derive(Debug, Clone, Copy)]
pub struct Estat {
    pub bits: usize,
}

impl Register for Estat {
    fn read() -> Self {
        //读取sstat的内容出来
        let mut bits;
        unsafe {
            // asm!("csrrd {},{}", out(reg) bits,const CSR_ESTAT);
            asm!("csrrd {}, 0x5", out(reg) bits);
        }
        Estat { bits }
    }
    fn write(&mut self) {
        unsafe {
            // asm!("csrwr {},{}", in(reg) self.bits,const CSR_ESTAT);
            asm!("csrwr {}, 0x5", in(reg) self.bits);
        }
    }
}

impl Estat {
    pub fn get_val(&self) -> usize {
        self.bits
    }
    pub fn set_val(&mut self, val: usize) -> &mut Self {
        self.bits = val;
        self
    }
    pub fn get_is_with_index(&self, index: usize) -> bool {
        // 0-12位为中断
        assert!(index < 13);
        self.bits.get_bit(index)
    }
    // 只有写0和1位有效，这两位控制软件中断
    pub fn set_is_with_index(&mut self, index: usize, value: bool) -> &mut Self {
        assert!(index < 13);
        self.bits.set_bit(index, value);
        self
    }
    // 例外类型一级编码。触发例外时：
    // 如果是 TLB 重填例外或机器错误例外，该域保持不变；
    // 否则，硬件会根据例外类型将表 7- 8 中 Ecode 栏定义的数值写入该域。
    //例外类型一级编号 21-16位
    pub fn get_ecode(&self) -> usize {
        self.bits.get_bits(16..=21)
    }
    //例外类型二级编号 22-30位
    pub fn get_esubcode(&self) -> usize {
        self.bits.get_bits(22..=30)
    }

    pub fn cause(&self) -> Trap {
        // 优先判断是否是重填异常
        let is_tlb_reload = TLBREra::read().get_is_tlbr();
        if is_tlb_reload {
            return Trap::Exception(Exception::TLBRFill);
        }
        let ecode = self.get_ecode();
        if ecode == 0 {
            // 仅当 CSR.ECFG.VS=0 时，表示是中断
            let ecfg = Ecfg::read();
            let ecfg_vs = ecfg.get_vs();
            if ecfg_vs == 0 {
                // 读取中断位
                for index in (0..13).rev() {
                    if self.get_is_with_index(index) && ecfg.get_lie_with_index(index) {
                        //log::debug!("index={:#x?}", index);
                        return Trap::Interrupt(Interrupt::from_usize(index));
                    }
                }
            }
            return Trap::Unknown;
        }
        let sub_ecode = self.get_esubcode();
        match ecode {
            0x1 => Trap::Exception(Exception::LoadPageInvalid), // load
            0x2 => Trap::Exception(Exception::StorePageInvalid), // store
            0x3 => Trap::Exception(Exception::FetchPageInvalid), //取指操作页面不存在
            0x4 => Trap::Exception(Exception::PageModification), //页修改例外
            0x5 => Trap::Exception(Exception::PageNonReadable), //页不可读
            0x6 => Trap::Exception(Exception::PageNonExecutable), //页不可执行
            0x7 => Trap::Exception(Exception::PagePrivilegeIllegal), //页特权级不合规
            0x8 => {
                match sub_ecode {
                    0x1 => Trap::Exception(Exception::FetchInstructionAddressError), //取指地址错误
                    0x2 => Trap::Exception(Exception::MemoryAccessInstructionAddressError), //访存地址访问错误
                    _ => Trap::Unknown,
                }
            }
            0x9 => Trap::Exception(Exception::AddressAlignmentFault), //地址不对齐
            0xa => Trap::Exception(Exception::BoundCheck),            //越界例外
            0xb => Trap::Exception(Exception::Syscall),               //系统调用
            0xc => Trap::Exception(Exception::Breakpoint),            //调试中断
            0xd => Trap::Exception(Exception::InstructionNonDefined), //指令不合规
            0xe => Trap::Exception(Exception::InstructionPrivilegeError), //指令特权级不合规
            0xf => Trap::Exception(Exception::FloatingPointInstructionDisable), //浮点处理器不可用
            _ => Trap::Unknown,
        }
    }
}

// 异常类型
#[derive(Debug, Clone, Copy)]
pub enum Exception {
    LoadPageInvalid,
    StorePageInvalid,
    FetchPageInvalid,
    PageModification,
    PageNonReadable,
    PageNonExecutable,
    PagePrivilegeIllegal,
    FetchInstructionAddressError,
    MemoryAccessInstructionAddressError,   //内存访问错误
    AddressAlignmentFault = 0x9,           //地址不对齐
    BoundCheck = 0xA,                      //越界检查错误
    Syscall = 0xB,                         //系统调用
    Breakpoint = 0xC,                      //调试中断
    InstructionNonDefined = 0xD,           //指令不合规
    InstructionPrivilegeError = 0xE,       //特权指令不合规
    FloatingPointInstructionDisable = 0xF, //浮点不可用
    TLBRFill,                              //TLB重填
}

// 中断类型
#[derive(Debug, Clone, Copy)]
pub enum Interrupt {
    SWI0,  //软件中断0
    SWI1,  //软件中断1
    HWI0,  //硬件中断0
    HWI1,  //硬件中断1
    HWI2,  //硬件中断2
    HWI3,  //硬件中断3
    HWI4,  //硬件中断4
    HWI5,  //硬件中断5
    HWI6,  //硬件中断6
    HWI7,  //硬件中断7
    PMI,   //性能监测计数溢出中断
    Timer, //定时器中断
    IPI,   //多处理器间的中断
}

impl Interrupt {
    pub fn from_usize(value: usize) -> Self {
        match value {
            0 => Interrupt::SWI0,
            1 => Interrupt::SWI1,
            2 => Interrupt::HWI0,
            3 => Interrupt::HWI1,
            4 => Interrupt::HWI2,
            5 => Interrupt::HWI3,
            6 => Interrupt::HWI4,
            7 => Interrupt::HWI5,
            8 => Interrupt::HWI6,
            9 => Interrupt::HWI7,
            10 => Interrupt::PMI,
            11 => Interrupt::Timer,
            12 => Interrupt::IPI,
            _ => panic!("invalid interrupt index"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Trap {
    Exception(Exception),
    Interrupt(Interrupt),
    Unknown,
}
