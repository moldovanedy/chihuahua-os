#![no_std]
#![no_main]

use core::panic::PanicInfo;
use k_corelib::boot_info;
use k_corelib::log;

mod renderer;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain(k_params: *const boot_info::FramebufferData) -> ! {
    log::log_debug("Kernel booted!");

    let fb_info: &boot_info::FramebufferData = unsafe { &*k_params };

    renderer::setup_fb(fb_info);
    renderer::draw_test();

    loop {
        unsafe {
            core::arch::asm!("cli");
            core::arch::asm!("hlt");
        }
    }
}
