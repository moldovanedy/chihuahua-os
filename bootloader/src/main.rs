#![no_std]
#![no_main]

use log::{error, info, warn};
use uefi::{
    boot::ScopedProtocol,
    prelude::*,
    proto::console::gop::{self, GraphicsOutput},
};

use crate::{graphics_config::FramebufferData, sys_config_reader::SystemConfig};

mod graphics_config;
mod kernel_loader;
mod sys_config_reader;

fn panic_fn(err: uefi::Error) -> ! {
    error!(
        "Fatal error: {err}. \r\nSystem halted. You can turn off the device now (auto power-off in 5 seconds)."
    );

    for _i in 0..4 {
        boot::stall(1_000_000);
    }

    panic!();
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    info!("Starting boot proces...");

    let config: SystemConfig = sys_config_reader::read_config().unwrap_or(SystemConfig::default());

    let jump_address: u64 = kernel_loader::load_kernel();
    if jump_address == 0 {
        panic!();
    }

    let mut fb_data: Option<graphics_config::FramebufferData> =
        graphics_config::set_appropriate_framebuffer(
            config.preferred_width(),
            config.preferred_height(),
        );

    if fb_data.is_none() {
        warn!("Failed to set the best graphics mode.");

        let gop_handle: Result<Handle, uefi::Error> =
            boot::get_handle_for_protocol::<GraphicsOutput>();
        if gop_handle.is_err() {
            panic_fn(gop_handle.clone().err().unwrap());
        }

        let gop_handle: Handle = gop_handle.unwrap();
        let gop: Result<boot::ScopedProtocol<GraphicsOutput>, uefi::Error> =
            boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle);
        if gop.is_err() {
            panic_fn(gop.unwrap_err());
        }

        let mode_info: gop::ModeInfo = gop.unwrap().current_mode_info();
        fb_data = FramebufferData::from_mode_info(mode_info);

        //if still none, panic
        if fb_data.is_none() {
            error!("Fatal: no graphics mode could be retrieved.");
            panic!()
        }
    }

    let fb_data: graphics_config::FramebufferData = fb_data.unwrap();

    let width: u32 = fb_data.width();
    let height: u32 = fb_data.height();
    info!("Switched to graphics mode with resolution {width}x{height}.");

    let _ = draw_test();

    //TODO: setup identity paging, exit BS, and boot the kernel

    loop {
        boot::stall(1_000_000);
    }
}

fn draw_test() -> uefi::Result {
    let gop_handle: Handle = boot::get_handle_for_protocol::<GraphicsOutput>()?;
    let gop: boot::ScopedProtocol<GraphicsOutput> =
        boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;

    draw_rect(gop, 200, 20, 350, 450, 0xff_00_00);
    return Ok(());
}

fn draw_rect(
    mut gop: ScopedProtocol<GraphicsOutput>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: u32,
) {
    let fb: *mut u8 = gop.frame_buffer().as_mut_ptr();
    let pitch: u32 = gop.current_mode_info().stride() as u32;

    unsafe {
        for y_pos in y..(y + height) {
            for x_pos in x..(x + width) {
                let base_addr: *mut u8 = fb.add((4 * y_pos * pitch + 4 * x_pos) as usize);
                *base_addr.add(0) = color as u8;
                *base_addr.add(1) = (color >> 8) as u8;
                *base_addr.add(2) = (color >> 16) as u8;
                *base_addr.add(3) = (color >> 24) as u8;
            }
        }
    }
}
