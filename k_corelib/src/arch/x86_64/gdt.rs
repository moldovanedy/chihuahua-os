use x86_64::instructions::segmentation;
use x86_64::registers::segmentation::{Segment};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
use dog_essentials::lazy_static::lazy_static;

lazy_static! {
    static ref GDT: GlobalDescriptorTable = {
        let mut gdt: GlobalDescriptorTable = GlobalDescriptorTable::new();
        let cs = gdt.append(Descriptor::kernel_code_segment());
        let ds = gdt.append(Descriptor::kernel_data_segment());
        
        GDT.load();
        
        unsafe {
            segmentation::CS::set_reg(cs);
    
            // Reload data segment registers (in long mode, usually set to 0 or valid selector)
            segmentation::DS::set_reg(ds);
            segmentation::ES::set_reg(ds);
            segmentation::SS::set_reg(ds);
        }
        
        return gdt;
    };
}
