use crate::arch::x86_64;

/// Reads a 32-bit value from the given port. Equivalent to x86_64 inl.
pub unsafe fn read_u32(port: u32) -> u32 {
    unsafe {
        #[cfg(target_arch = "x86_64")]
        x86_64::ports::read_u32(port)
    }
}

/// Writes a 32-bit value to the given port. Equivalent to x86_64 outl.
pub unsafe fn write_u32(port: u32, value: u32) {
    unsafe {
        #[cfg(target_arch = "x86_64")]
        x86_64::ports::write_u32(port, value)
    }
}

/// Reads a 16-bit value from the given port. Equivalent to x86_64 inw.
pub unsafe fn read_u16(port: u32) -> u16 {
    unsafe {
        #[cfg(target_arch = "x86_64")]
        x86_64::ports::read_u16(port)
    }
}

/// Writes a 16-bit value to the given port. Equivalent to x86_64 outw.
pub unsafe fn write_u16(port: u32, value: u16) {
    unsafe {
        #[cfg(target_arch = "x86_64")]
        x86_64::ports::write_u16(port, value)
    }
}

/// Reads a 8-bit value from the given port. Equivalent to x86_64 inb.
pub unsafe fn read_u8(port: u32) -> u8 {
    unsafe {
        #[cfg(target_arch = "x86_64")]
        x86_64::ports::read_u8(port)
    }
}

/// Writes a 8-bit value to the given port. Equivalent to x86_64 outb.
pub unsafe fn write_u8(port: u32, value: u8) {
    unsafe {
        #[cfg(target_arch = "x86_64")]
        x86_64::ports::write_u8(port, value)
    }
}
