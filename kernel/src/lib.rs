#![no_std]
#![no_main]

#[allow(dead_code)]
use boot_info;
use k_corelib::log;
use k_corelib::platform_initializer::initialize_platform;
use k_corelib::renderer;
use k_corelib::renderer::text_writer;

#[unsafe(no_mangle)]
pub extern "C" fn kmain(k_params: *const boot_info::KParams) -> ! {
    log::log_debug("Entered in kernel.");

    let fb_info: &boot_info::framebuffer::FramebufferData = unsafe { &(*k_params).fb_data };
    renderer::setup_fb(fb_info);

    let fg_col: renderer::Color = renderer::Color::from_u32(0xff_ff_ff);
    let bg_col: renderer::Color = renderer::Color::from_u32(0x00_00_00);
    text_writer::init();
    text_writer::write(b"Kernel booted!\n", fg_col, bg_col);
    initialize_platform();

    let mem_map_size: u32 = unsafe { (*k_params).memory_map_size };
    // for i in 0..mem_map_size {
    //     unsafe {
    //         let data: u8 = *((boot_info::MMAP_VIRTUAL_ADDRESS as *const u8).add(i as usize));
    //     }
    // }

    //trigger double fault
    // #[allow(unconditional_panic)]
    // let x = 1 / 0;

    text_writer::write(b"Halting now.", fg_col, bg_col);
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
