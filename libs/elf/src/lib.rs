#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe {
            core::arch::asm!("cli");
            core::arch::asm!("hlt");
        }
    }
}

// this will be our own implementation for reading ELF files
pub mod elf_file;
