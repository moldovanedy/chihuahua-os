use crate::ports;

/// Command sent to begin PIC initialization.
const CMD_INIT: u8 = 0x11;

/// Command sent to acknowledge an interrupt.
const CMD_END_OF_INTERRUPT: u8 = 0x20;

// The mode in which we want to run our PICs.
const MODE_8086: u8 = 0x01;

struct Pic {
    offset: u8,
    command_port: u8,
    data_port: u8,
}

pub struct PicPair {
    pair: [Pic; 2],
}

impl PicPair {
    pub const fn new() -> Self {
        PicPair {
            pair: [
                Pic {
                    offset: 32,
                    command_port: 0x20,
                    data_port: 0x21,
                },
                Pic {
                    offset: 40,
                    command_port: 0xA0,
                    data_port: 0xA1,
                },
            ],
        }
    }

    pub unsafe fn init(&mut self) {
        unsafe {
            //this will create an artificial delay
            let io_wait: fn() = || ports::write_u8(0x80, 0);

            //notify PIC that we will start a 3-byte init sequence
            ports::write_u8(self.pair[0].command_port as u32, CMD_INIT);
            io_wait();
            ports::write_u8(self.pair[1].command_port as u32, CMD_INIT);
            io_wait();

            //Byte 1: set offsets
            ports::write_u8(self.pair[0].data_port as u32, self.pair[0].offset);
            io_wait();
            ports::write_u8(self.pair[1].data_port as u32, self.pair[1].offset);
            io_wait();

            //Byte 2: setup chaining
            ports::write_u8(self.pair[0].data_port as u32, 4);
            io_wait();
            ports::write_u8(self.pair[1].data_port as u32, 2);
            io_wait();

            //Byte 3: set operation mode
            ports::write_u8(self.pair[0].data_port as u32, MODE_8086);
            io_wait();
            ports::write_u8(self.pair[1].data_port as u32, MODE_8086);
            io_wait();

            //unmask
            ports::write_u8(self.pair[0].data_port as u32, 0);
            ports::write_u8(self.pair[1].data_port as u32, 0);

            //temp: setup PIT here, later we will write a separate driver
            let divisor: u32 = 1193182 / 100;
            ports::write_u8(0x43, 0x36);
            ports::write_u8(0x40, (divisor & 0xff) as u8);
            ports::write_u8(0x40, ((divisor >> 8) & 0xff) as u8);
        }
    }

    pub unsafe fn read_pic1(&self) -> u8 {
        unsafe { ports::read_u8(self.pair[0].data_port as u32) }
    }

    pub unsafe fn read_pic2(&self) -> u8 {
        unsafe { ports::read_u8(self.pair[1].data_port as u32) }
    }

    pub unsafe fn write_pic1(&mut self, data: u8) {
        unsafe { ports::write_u8(self.pair[0].data_port as u32, data) }
    }

    pub unsafe fn write_pic2(&mut self, data: u8) {
        unsafe { ports::write_u8(self.pair[1].data_port as u32, data) }
    }

    pub unsafe fn disable_pic1(&mut self) {
        unsafe {
            ports::write_u8(self.pair[0].data_port as u32, 0xFF);
        }
    }

    pub unsafe fn disable_pic2(&mut self) {
        unsafe {
            ports::write_u8(self.pair[1].data_port as u32, 0xFF);
        }
    }

    pub fn does_handle_interrupt(&self, irq: u8) -> bool {
        if self.pair[0].offset <= irq && irq < self.pair[0].offset + 8 {
            return true;
        }
        if self.pair[1].offset <= irq && irq < self.pair[1].offset + 8 {
            return true;
        }

        return false;
    }

    pub fn send_end_of_interrupt(&mut self, irq: u8) {
        unsafe {
            if self.does_handle_interrupt(irq) {
                //does_handle_interrupt for PIC2 only
                if self.pair[1].offset <= irq && irq < self.pair[1].offset + 8 {
                    ports::write_u8(self.pair[1].command_port as u32, CMD_END_OF_INTERRUPT);
                }

                ports::write_u8(self.pair[0].command_port as u32, CMD_END_OF_INTERRUPT);
            }
        }
    }
}
