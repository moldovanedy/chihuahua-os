pub mod cpu_exceptions;
pub mod x86_64_pic_interrupts;

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
