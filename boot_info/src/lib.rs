#![no_std]

pub mod framebuffer;
pub mod memory_map;

pub const KERNEL_VIRTUAL_ADDRESS: u64 = 0xffff_eeee_8000_0000;
pub const GOP_VIRTUAL_ADDRESS: u64 = 0xffff_eeed_f000_0000;
pub const MEM_MAP_VIRTUAL_ADDRESS: u64 = 0xffff_eeed_e000_0000;
/// Can theoretically use up to 0xffff_fffe_e000_0000, but should cap at 0xffff_fffe_8000_0000
pub const PHYS_BITMAP_MANAGER_ADDRESS: u64 = 0xffff_eeed_0000_0000;
/// The virtual address for the page tables themselves.
pub const PAGE_TABLES_ADDRESS: u64 = 0xffff_eeec_f000_0000;

/// This is the **theoretical** heap limit of the kernel (the max virtual address). In reality,
/// the kernel uses way less memory for its heap.
pub const K_HEAP_END: u64 = 0xffff_ee00_0000_0000;
/// This is the start of the kernel heap. It grows upwards towards [`K_HEAP_END`].
pub const K_HEAP_START: u64 = 0xffff_e000_0000_0000;

#[repr(C)]
pub struct KParams {
    pub fb_data: framebuffer::FramebufferData,
    pub memory_map_size: u32,
    /// The number of entries in the page table.
    pub page_table_num_entries: u64,
    /// The physical address of the EFI Runtime Services table. Interpret this to interact with the
    /// system using UEFI.
    pub uefi_rs_phys_addr: u64,
}
