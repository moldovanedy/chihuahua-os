use boot_info::KParams;
use x86_64::{
    registers::control::{Cr0, Cr0Flags, Cr3, Cr4, Cr4Flags, Efer, EferFlags},
    structures::paging::PhysFrame,
};

pub fn boot_kernel(
    entry_point: u64,
    page_table_address: x86_64::PhysAddr,
    kernel_params: KParams,
) -> ! {
    unsafe {
        // Set CR3 to new page table
        Cr3::write(
            PhysFrame::containing_address(page_table_address),
            Cr3::read().1,
        );

        // Enable PAE and long mode
        Cr4::update(|cr4: &mut Cr4Flags| cr4.insert(Cr4Flags::PHYSICAL_ADDRESS_EXTENSION));
        Efer::update(|efer: &mut EferFlags| efer.insert(EferFlags::LONG_MODE_ENABLE));

        // Enable paging
        Cr0::update(|cr0: &mut Cr0Flags| {
            cr0.insert(Cr0Flags::PAGING);
            cr0.insert(Cr0Flags::PROTECTED_MODE_ENABLE);
        });

        let k_params_ptr = &kernel_params as *const KParams;

        // Far jump to kernel entry
        let entry: extern "C" fn() -> ! = {
            //dangerously set the kernel boot parameters in the rdi register (first param in SysV calling convention)
            core::arch::asm!("mov rdi, {}", in(reg) k_params_ptr);
            core::mem::transmute(entry_point)
        };
        entry();
    };
}

// fn setup_gdt() {
//     let mut gdt = GlobalDescriptorTable::new();
//     let k_cs = gdt.append(Descriptor::kernel_code_segment());
//     gdt.load();
// 
//     //x86_64::instructions::segmentation::CS::set_reg()
// }
