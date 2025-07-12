#![no_std]
#![no_main]

#[allow(dead_code)]
use boot_info;
use k_corelib::log;
use k_corelib::mem_manager::vmm;
use k_corelib::platform_initializer;
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
    platform_initializer::initialize_platform();

    unsafe {
        vmm::init((*k_params).memory_map_size, (*k_params).page_table_size);
        text_writer::write(b"Setup memory.\n", fg_col, bg_col);
    }

    //trigger double fault
    // #[allow(unconditional_panic)]
    // let x = 1 / 0;

    text_writer::write(
        b"No more work to do. System is halting now.\n",
        fg_col,
        bg_col,
    );
    text_writer::write(
        b"You can now safely turn off your computer!",
        renderer::Color::from_u32(0x30_a3_f0),
        bg_col,
    );

    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
