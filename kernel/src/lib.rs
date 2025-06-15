#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod k_drivers;
mod renderer;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    //k_drivers::com_debug::init_serial();
    //k_drivers::com_debug::write_char(b'1');

    //renderer::draw_test();

    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
