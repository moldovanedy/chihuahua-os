use crate::k_drivers::pic::PicPair;
use dog_essentials::sync::mutex::Mutex;
use crate::arch::x86_64::idt;
use crate::interrupts::InterruptHandler;

pub static PIC: Mutex<PicPair> = Mutex::new(PicPair::new());

pub fn init() {
    unsafe {
        PIC.lock().init();
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    PitTick = 32,
    Kbd = 33
}

impl From<u32> for InterruptIndex {
    fn from(value: u32) -> Self {
        match value {
            32 => Self::PitTick,
            33 => Self::Kbd,
            _ => Self::PitTick
        }
    }
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub fn set_handler(exception_type: InterruptIndex, handler: InterruptHandler) {
    #[cfg(target_arch = "x86_64")]
    {
        match exception_type {
            InterruptIndex::PitTick => idt::set_pic_tick_handler(handler),
            InterruptIndex::Kbd => idt::set_kbd_input_handler(handler)
        }
    }
}
