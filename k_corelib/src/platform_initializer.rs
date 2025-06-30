use crate::interrupts;
use crate::interrupts::ExceptionType;
use crate::renderer::{text_writer, Color};

pub fn initialize_platform() {
    #[cfg(target_arch = "x86_64")]
    {
        //GDT is setup at startup automatically inside a lazy_static!
        //gdt::setup_gdt();
    }

    interrupts::set_handler(
        ExceptionType::Breakpoint, 
        |args| {
            text_writer::write(
                b"Breakpoint: ",
                Color::from_u32(0xff_00_00),
                Color::from_u32(0x00_00_00),
            );
            
            text_writer::write(
                dog_essentials::format_non_alloc::u64_to_str(args.instruction_pointer())
                    .to_str()
                    .as_bytes(),
                Color::from_u32(0xff_00_00),
                Color::from_u32(0x00_00_00),
            );
            
            text_writer::write(
                b"\n",
                Color::from_u32(0xff_00_00),
                Color::from_u32(0x00_00_00),
            );
        });
    interrupts::setup();
}
