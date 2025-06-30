use crate::arch::x86_64::idt;

pub struct InterruptArguments {
    instruction_pointer: u64,
    cpu_flags: u64,
    stack_pointer: u64,
}

impl InterruptArguments {
    pub fn new(instruction_pointer: u64, cpu_flags: u64, stack_pointer: u64) -> Self {
        Self {
            instruction_pointer,
            cpu_flags,
            stack_pointer,
        }
    }

    pub fn instruction_pointer(&self) -> u64 {
        self.instruction_pointer
    }

    pub fn cpu_flags(&self) -> u64 {
        self.cpu_flags
    }

    pub fn stack_pointer(&self) -> u64 {
        self.stack_pointer
    }
}

pub type InterruptHandler = fn(InterruptArguments);

#[repr(u32)]
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
