use crate::interrupts::cpu_exceptions::ExceptionType;
use crate::interrupts::{cpu_exceptions, x86_64_pic_interrupts};
use crate::renderer::{text_writer, Color};

pub fn initialize_platform() {
    //GDT should already be initialized, as it is inside a lazy_static

    cpu_exceptions::set_handler(ExceptionType::Breakpoint, |args| {
        text_writer::write(
            b"Breakpoint: ",
            Color::from_u32(0xff_00_00),
            Color::from_u32(0x00_00_00),
        );

        text_writer::write(
            dog_essentials::format_non_alloc::u64_to_str_base(args.instruction_pointer(), 16)
                .to_str()
                .as_bytes(),
            Color::from_u32(0xff_00_00),
            Color::from_u32(0x00_00_00),
        );

        text_writer::write(b"\n", Color::from_u32(0xff_00_00), Color::from_u32(0));
    });
    cpu_exceptions::setup();

    #[cfg(target_arch = "x86_64")]
    {
        x86_64_pic_interrupts::init();
        x86_64::instructions::interrupts::enable();
    }

    text_writer::write(
        b"Set up interrupts.\n",
        Color::from_u32(0xff_ff_ff),
        Color::from_u32(0),
    );
}
