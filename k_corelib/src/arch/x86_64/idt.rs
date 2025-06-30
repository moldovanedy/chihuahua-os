use crate::interrupts::{InterruptArguments, InterruptHandler};
use dog_essentials::lazy_static::lazy_static;
use dog_essentials::static_cell::StaticCell;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.divide_error.set_handler_fn(on_division_error);
        idt.breakpoint.set_handler_fn(on_breakpoint);
        idt.invalid_opcode.set_handler_fn(on_invalid_opcode);
        idt.general_protection_fault.set_handler_fn(on_gpf);
        idt.page_fault.set_handler_fn(on_page_fault);
        idt.simd_floating_point.set_handler_fn(on_simd_fpe);
        idt.double_fault.set_handler_fn(on_double_fault);
        idt.non_maskable_interrupt.set_handler_fn(on_non_maskable_interrupt_opcode);
        return idt;
    };
}

pub(crate) fn setup_idt() {
    IDT.load();
}

static DIVISION_HANDLER: StaticCell<InterruptHandler> = StaticCell::new(|_| {});

pub(crate) fn set_division_handler(handler: InterruptHandler) {
    DIVISION_HANDLER.set_value_unsafe(handler);
}

static NON_MASKABLE_INTERRUPT_HANDLER: StaticCell<InterruptHandler> = StaticCell::new(|_| {});

pub(crate) fn set_non_maskable_interrupt_handler(handler: InterruptHandler) {
    NON_MASKABLE_INTERRUPT_HANDLER.set_value_unsafe(handler);
}

static BREAKPOINT_HANDLER: StaticCell<InterruptHandler> = StaticCell::new(|_| {});

pub(crate) fn set_breakpoint_handler(handler: InterruptHandler) {
    BREAKPOINT_HANDLER.set_value_unsafe(handler);
}

static INVALID_OPCODE_HANDLER: StaticCell<InterruptHandler> = StaticCell::new(|_| {});

pub(crate) fn set_invalid_opcode_handler(handler: InterruptHandler) {
    INVALID_OPCODE_HANDLER.set_value_unsafe(handler);
}

static GPF_HANDLER: StaticCell<InterruptHandler> = StaticCell::new(|_| {});

pub(crate) fn set_gpf_handler(handler: InterruptHandler) {
    GPF_HANDLER.set_value_unsafe(handler);
}

static PAGE_FAULT_HANDLER: StaticCell<InterruptHandler> = StaticCell::new(|_| {});

pub(crate) fn set_page_fault_handler(handler: InterruptHandler) {
    PAGE_FAULT_HANDLER.set_value_unsafe(handler);
}

static SIMD_FPE_HANDLER: StaticCell<InterruptHandler> = StaticCell::new(|_| {});

pub(crate) fn set_simd_fpe_handler(handler: InterruptHandler) {
    SIMD_FPE_HANDLER.set_value_unsafe(handler);
}

static DOUBLE_FAULT_HANDLER: StaticCell<InterruptHandler> = StaticCell::new(|_| {});

pub(crate) fn set_double_fault_handler(handler: InterruptHandler) {
    DOUBLE_FAULT_HANDLER.set_value_unsafe(handler);
}


extern "x86-interrupt" fn on_division_error(stack_frame: InterruptStackFrame) {
    DIVISION_HANDLER.get_value_unsafe()(get_args(&stack_frame));
}

extern "x86-interrupt" fn on_non_maskable_interrupt_opcode(stack_frame: InterruptStackFrame) {
    NON_MASKABLE_INTERRUPT_HANDLER.get_value_unsafe()(get_args(&stack_frame));
}

extern "x86-interrupt" fn on_breakpoint(stack_frame: InterruptStackFrame) {
    BREAKPOINT_HANDLER.get_value_unsafe()(get_args(&stack_frame));
}

extern "x86-interrupt" fn on_invalid_opcode(stack_frame: InterruptStackFrame) {
    INVALID_OPCODE_HANDLER.get_value_unsafe()(get_args(&stack_frame));
}

extern "x86-interrupt" fn on_gpf(stack_frame: InterruptStackFrame, _error_code: u64) {
    GPF_HANDLER.get_value_unsafe()(get_args(&stack_frame));
}

extern "x86-interrupt" fn on_page_fault(
    stack_frame: InterruptStackFrame,
    _error_code: PageFaultErrorCode,
) {
    PAGE_FAULT_HANDLER.get_value_unsafe()(get_args(&stack_frame));
}

extern "x86-interrupt" fn on_simd_fpe(stack_frame: InterruptStackFrame) {
    SIMD_FPE_HANDLER.get_value_unsafe()(get_args(&stack_frame));
}

extern "x86-interrupt" fn on_double_fault(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    DOUBLE_FAULT_HANDLER.get_value_unsafe()(get_args(&stack_frame));
    
    panic!(
        "Double fault! Error code: {:#?}\n Stack frame:\n{:#?}",
        error_code, stack_frame
    );
}

fn get_args(stack_frame: &InterruptStackFrame) -> InterruptArguments {
    let args: InterruptArguments = InterruptArguments::new(
        stack_frame.instruction_pointer.as_u64(),
        stack_frame.cpu_flags.bits(),
        stack_frame.stack_pointer.as_u64());
    
    return args;
}
