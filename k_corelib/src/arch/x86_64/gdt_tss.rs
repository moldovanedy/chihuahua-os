use dog_essentials::lazy_static::lazy_static;
use x86_64::instructions::{segmentation, tables};
use x86_64::registers::segmentation::Segment;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref GDT: GlobalDescriptorTable = {
        let mut gdt: GlobalDescriptorTable = GlobalDescriptorTable::new();
        let cs = gdt.append(Descriptor::kernel_code_segment());
        let ds = gdt.append(Descriptor::kernel_data_segment());
        let tss_segment = gdt.append(Descriptor::tss_segment(&TSS));

        GDT.load();

        unsafe {
            segmentation::CS::set_reg(cs);

            // Reload data segment registers (in long mode, usually set to 0 or valid selector)
            segmentation::DS::set_reg(ds);
            segmentation::ES::set_reg(ds);
            segmentation::SS::set_reg(ds);
            tables::load_tss(tss_segment);
        }

        return gdt;
    };

    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
                const STACK_SIZE: usize = 4096 * 5;
                static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

                #[allow(unused_unsafe)]
                let stack_start = unsafe { VirtAddr::from_ptr(&raw const STACK) };
                let stack_end = stack_start + STACK_SIZE as u64;
                stack_end
        };
        return tss;
    };
}
