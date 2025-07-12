#![no_std]

pub mod framebuffer;
pub mod memory_map;

pub const KERNEL_VIRTUAL_ADDRESS: u64 = 0xffff_ffff_8000_0000;
pub const GOP_VIRTUAL_ADDRESS: u64 = 0xffff_fffe_f000_0000;
pub const MEM_MAP_VIRTUAL_ADDRESS: u64 = 0xffff_fffe_e000_0000;
/// Can theoretically use up to 0xffff_fffe_e000_0000, but should cap at 0xffff_fffe_8000_0000
pub const PHYS_BITMAP_MANAGER_ADDRESS: u64 = 0xffff_fffe_0000_0000;

#[repr(C)]
pub struct KParams {
    pub fb_data: framebuffer::FramebufferData,
    pub memory_map_size: u32,
    /// The number of pages occupied by the page table itself.
    pub page_table_size: u32,
}
