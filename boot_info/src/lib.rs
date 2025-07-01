#![no_std]

pub mod framebuffer;
pub mod memory_map;

pub const KERNEL_VIRTUAL_ADDRESS: u64 = 0xffff_ffff_8000_0000;
pub const GOP_VIRTUAL_ADDRESS: u64 = 0xffff_fffe_f000_0000;
pub const MMAP_VIRTUAL_ADDRESS: u64 = 0xffff_fffe_e000_0000;

#[repr(C)]
pub struct KParams {
    pub fb_data: framebuffer::FramebufferData,
    pub memory_map_size: u32,
}
