#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod multiboot2_initializer;
use multiboot2_initializer::*;

mod k_drivers;
mod renderer;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain(multiboot2_magic: u64, multiboot2_info: u64) -> ! {
    //k_drivers::com_debug::init_serial();
    //k_drivers::com_debug::write_char(b'1');

    if multiboot2_magic != 0x36_d7_62_89 {
        panic!()
    }

    init_mb2_modules(multiboot2_info);
    renderer::draw_test();

    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
