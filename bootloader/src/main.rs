#![no_std]
#![no_main]

use k_corelib::boot_info::{self, KParams};
use log::{error, info, warn};
use uefi::{
    boot::MemoryType,
    prelude::*,
    proto::console::gop::{self, GraphicsOutput},
};
use x86_64::PhysAddr;

use crate::sys_config_reader::SystemConfig;

mod graphics_config;
mod kernel_loader;
mod kernel_reader;
mod paging;
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

fn panic_fn_str(err: &str) -> ! {
    error!("Fatal error: {err}. \r\nSystem halted. You can turn off the device now.");

    loop {
        boot::stall(1_000_000);
    }
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    info!("Starting boot proces...");

    let config: SystemConfig = sys_config_reader::read_config().unwrap_or(SystemConfig::default());

    let jump_address: u64 = kernel_reader::read_kernel();
    if jump_address == 0 {
        panic_fn_str("KERNEL_NOT_LOADED");
    }

    let mut fb_data: Option<boot_info::FramebufferData> =
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

        let mut gop = gop.unwrap();
        let mode_info: gop::ModeInfo = gop.current_mode_info();
        fb_data = graphics_config::fb_data_from_mode_info(mode_info, gop.frame_buffer());

        //if still none, panic
        if fb_data.is_none() {
            error!("Fatal: no graphics mode could be retrieved.");
            panic!()
        }
    }

    let fb_data: boot_info::FramebufferData = fb_data.unwrap();

    let width: u32 = fb_data.width();
    let height: u32 = fb_data.height();
    info!("Switched to graphics mode with resolution {width}x{height}.");

    //let _ = draw_test();

    let page_table_address: x86_64::PhysAddr = paging::setup_identity_paging();
    if page_table_address == PhysAddr::zero() {
        panic_fn_str("MEMORY_PAGING_NOT_IDENTITY_MAPPED");
    }

    info!("Paging enabled with identity mapping");

    unsafe {
        boot::exit_boot_services(Some(MemoryType::LOADER_DATA));
    }

    let k_params: KParams = KParams { fb_data: fb_data };
    kernel_loader::boot_kernel(jump_address, page_table_address, k_params);
}

//TODO: this will write a logo later

// fn draw_test() -> uefi::Result {
//     let gop_handle: Handle = boot::get_handle_for_protocol::<GraphicsOutput>()?;
//     let gop: boot::ScopedProtocol<GraphicsOutput> =
//         boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;

//     draw_rect(gop, 200, 20, 350, 450, 0xff_00_00);
//     return Ok(());
// }

// fn draw_rect(
//     mut gop: ScopedProtocol<GraphicsOutput>,
//     x: u32,
//     y: u32,
//     width: u32,
//     height: u32,
//     color: u32,
// ) {
//     let fb: *mut u8 = gop.frame_buffer().as_mut_ptr();
//     let pitch: u32 = gop.current_mode_info().stride() as u32;

//     unsafe {
//         for y_pos in y..(y + height) {
//             for x_pos in x..(x + width) {
//                 let base_addr: *mut u8 = fb.add((4 * y_pos * pitch + 4 * x_pos) as usize);
//                 *base_addr.add(0) = color as u8;
//                 *base_addr.add(1) = (color >> 8) as u8;
//                 *base_addr.add(2) = (color >> 16) as u8;
//                 *base_addr.add(3) = (color >> 24) as u8;
//             }
//         }
//     }
// }
