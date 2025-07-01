#![no_std]

pub mod fb_writer;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe {
            fb_writer::cover_screen();
            
            core::arch::asm!("cli");
            core::arch::asm!("hlt");
        }
    }
}


