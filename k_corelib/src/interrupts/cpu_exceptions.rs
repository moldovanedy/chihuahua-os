use crate::arch::x86_64::idt;
use crate::interrupts::InterruptHandler;

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExceptionType {
    Division = 0,
    Breakpoint = 1,
    InvalidOpcode = 2,
    ProtectionFault = 3,
    PageFault = 4,
    FloatingPointError = 5,
    NonMaskableInterrupt = 6,
    DoubleFault = 0xff_ff,
}

impl From<u32> for ExceptionType {
    fn from(value: u32) -> Self {
        match value {
            0 => ExceptionType::Division,
            1 => ExceptionType::Breakpoint,
            2 => ExceptionType::InvalidOpcode,
            3 => ExceptionType::ProtectionFault,
            4 => ExceptionType::PageFault,
            5 => ExceptionType::FloatingPointError,
            6 => ExceptionType::NonMaskableInterrupt,
            0xff_ff => ExceptionType::DoubleFault,
            _ => ExceptionType::DoubleFault,
        }
    }
}

pub fn setup() {
    #[cfg(target_arch = "x86_64")]
    idt::setup_idt();
}

pub fn set_handler(exception_type: ExceptionType, handler: InterruptHandler) {
    #[cfg(target_arch = "x86_64")]
    {
        match exception_type {
            ExceptionType::Division => idt::set_division_handler(handler),
            ExceptionType::Breakpoint => idt::set_breakpoint_handler(handler),
            ExceptionType::InvalidOpcode => idt::set_invalid_opcode_handler(handler),
            ExceptionType::ProtectionFault => idt::set_gpf_handler(handler),
            ExceptionType::PageFault => idt::set_page_fault_handler(handler),
            ExceptionType::FloatingPointError => idt::set_simd_fpe_handler(handler),
            ExceptionType::NonMaskableInterrupt => idt::set_non_maskable_interrupt_handler(handler),
            ExceptionType::DoubleFault => idt::set_double_fault_handler(handler),
        }
    }
}
