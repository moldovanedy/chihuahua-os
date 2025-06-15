use k_corelib::ports;

const PORT: u32 = 0x3f8; //COM1 interface

pub fn init_serial() -> bool {
    unsafe {
        ports::write_u8(PORT + 1, 0x00); // Disable all interrupts
        ports::write_u8(PORT + 3, 0x80); // Enable DLAB (set baud rate divisor)
        ports::write_u8(PORT + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
        ports::write_u8(PORT + 1, 0x00); //                  (hi byte)
        ports::write_u8(PORT + 3, 0x03); // 8 bits, no parity, one stop bit
        ports::write_u8(PORT + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
        ports::write_u8(PORT + 4, 0x0B); // IRQs enabled, RTS/DSR set
        ports::write_u8(PORT + 4, 0x1E); // Set in loopback mode, test the serial chip

        // Test serial chip (send byte 0xAE and check if serial returns same byte)
        ports::write_u8(PORT + 0, 0xAE);
        if ports::read_u8(PORT + 0) != 0xAE {
            return false;
        }

        ports::write_u8(PORT + 4, 0x0F); // Normal operation mode
        return true;
    }
}

pub fn write_char(chr: u8) {
    while is_transmit_empty() {}

    unsafe { ports::write_u8(PORT, chr as u8) };
}

fn is_transmit_empty() -> bool {
    unsafe {
        return (ports::read_u8(PORT + 5) & 0x20) == 0;
    }
}
