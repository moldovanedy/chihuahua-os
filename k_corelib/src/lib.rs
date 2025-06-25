#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    log::log_fatal("Kernel panic:");

    let msg = _info.message().as_str();
    if msg.is_some() {
        log::log_fatal(msg.unwrap());
    }

    loop {
        unsafe {
            core::arch::asm!("cli");
            core::arch::asm!("hlt");
        }
    }
}

pub mod log;
pub mod ports;
pub mod renderer;
pub mod essentials_clone;

mod arch;
mod k_drivers;
