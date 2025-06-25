#![no_std]
#![no_main]

use boot_info;
use k_corelib::log;
use k_corelib::renderer;

mod init_text;

#[unsafe(no_mangle)]
pub extern "C" fn kmain(k_params: *const boot_info::KParams) -> ! {
    log::log_debug("Kernel booted!");

    let fb_info: &boot_info::framebuffer::FramebufferData = unsafe { &(*k_params).fb_data };
    renderer::setup_fb(fb_info);

    let fg_col: renderer::Color = renderer::Color::from_u32(0xff_ff_ff);
    let bg_col: renderer::Color = renderer::Color::from_u32(0x00_00_00);
    init_text::init();
    init_text::write(b"Kernel booted!\n", fg_col, bg_col);

    let mem_map_size: u32 = unsafe { (*k_params).memory_map_size };
    // for i in 0..mem_map_size {
    //     unsafe {
    //         let data: u8 = *((boot_info::EFI_MMAP_VIRTUAL_ADDRESS as *const u8).add(i as usize));
    //     }
    // }

    for i in 0..500 {
        init_text::write(dog_essentials::format_non_alloc::i32_to_str(i).to_str().as_bytes(), fg_col, bg_col);
        init_text::write(b"\n", fg_col, bg_col);
    }
    
    init_text::write(b"Halting now.", fg_col, bg_col);
    loop {
        unsafe {
            core::arch::asm!("cli");
            core::arch::asm!("hlt");
        }
    }
}
