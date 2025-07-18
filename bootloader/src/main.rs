#![no_std]
#![no_main]

use crate::paging::PageTableInfo;
use boot_info::memory_map::MemoryMapEntry;
use core::ptr::NonNull;
use log::{error, info, warn};
use uefi::mem::memory_map::{MemoryMap, MemoryMapMut, MemoryMapOwned};
use uefi::{
    boot::MemoryType,
    prelude::*,
    proto::console::gop::{self, GraphicsOutput},
};
use x86_64;

#[allow(dead_code)]
use crate::sys_config_reader::SystemConfig;

mod graphics_config;
mod kernel_loader;
mod kernel_reader;
mod paging;
mod phys_memory_map;
mod raw_mem_map;
mod sys_config_reader;

fn panic_fn(err: uefi::Error) -> ! {
    error!("Fatal error: {err}. \r\nSystem halted. You can turn off the device now.");

    loop {
        boot::stall(1_000_000);
    }
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

    let mem_map: MemoryMapOwned = get_efi_mmap();

    let config: SystemConfig = sys_config_reader::read_config().unwrap_or(SystemConfig::default());

    let k_load_data: Option<(u64, u64)> = kernel_reader::read_kernel(&mem_map);
    if k_load_data.is_none() {
        panic_fn_str("KERNEL_NOT_LOADED");
    }

    let k_load_data: (u64, u64) = k_load_data.unwrap();
    let k_physical_address: u64 = k_load_data.0;
    let k_entry_point: u64 = k_load_data.1;

    let mut fb_data: Option<boot_info::framebuffer::FramebufferData> =
        graphics_config::set_appropriate_framebuffer(
            config.preferred_width(),
            config.preferred_height(),
        );

    if fb_data.is_none() {
        warn!("Failed to set the best graphics mode.");
        let mode_info: gop::ModeInfo = get_gop().current_mode_info();
        fb_data = graphics_config::fb_data_from_mode_info(mode_info);

        //if still none, panic
        if fb_data.is_none() {
            error!("Fatal: no graphics mode could be retrieved.");
            panic_fn_str("GFX_ERROR");
        }
    }

    let fb_data: boot_info::framebuffer::FramebufferData = fb_data.unwrap();
    let width: u32 = fb_data.width();
    let height: u32 = fb_data.height();
    info!("Switched to graphics mode with resolution {width}x{height}.");

    //let _ = draw_test();

    //here we don't need the updated map, just a sorted one, so we can determine the RAM size
    let mut mem_map = mem_map;
    mem_map.sort();
    let pmm_sections_array: u64 = phys_memory_map::allocate_memory_for_pmm(&mem_map);

    let mem_map: MemoryMapOwned = get_efi_mmap();
    let raw_mem_map_result: Option<(u64, u32)> = raw_mem_map::alloc_memory_for_map(&mem_map);
    if raw_mem_map_result.is_none() {
        error!("Error finding memory for the direct memory map.");
        panic_fn_str("RAW_MEM_MAP_ERROR");
    }

    let raw_mem_map_result: (u64, u32) = raw_mem_map_result.unwrap();
    let raw_mem_map_addr: u64 = raw_mem_map_result.0;
    let raw_mem_map_page_count: u32 = raw_mem_map_result.1;
    let mem_map: MemoryMapOwned = get_efi_mmap();

    let page_table_info: Option<PageTableInfo> = paging::setup_paging(
        &mem_map,
        &mut get_gop().frame_buffer(),
        k_physical_address,
        raw_mem_map_addr,
        pmm_sections_array,
    );
    if page_table_info.is_none() {
        panic_fn_str("MEMORY_PAGING_NOT_MAPPED");
    }

    let page_table_info: PageTableInfo = page_table_info.unwrap();

    info!("Paging enabled");

    let mut mem_map_size: u32 = 0;
    unsafe {
        let mut final_mem_map: MemoryMapOwned =
            boot::exit_boot_services(Some(MemoryType::LOADER_DATA));

        final_mem_map.sort();
        let mut map_writer = raw_mem_map_addr as *mut MemoryMapEntry;

        for entry in final_mem_map.entries() {
            //the EFI memory map now has so many additional entries (even with that +10 "safe zone"),
            //that we can no longer give all the necessary information
            if mem_map_size >= raw_mem_map_page_count * 0x1000 {
                error!(
                    "Error: EFI memory map is suddenly larger than expected for the direct memory map."
                );
                panic_fn_str("EFI_MEM_MAP_UNEXPECTEDLY_LARGE");
            }

            *map_writer = MemoryMapEntry::new(
                boot_info::memory_map::MemoryType::from(entry.ty.0),
                entry.att.bits(),
                entry.phys_start,
                entry.virt_start,
                entry.page_count,
            );

            map_writer = map_writer.add(1);
            mem_map_size += 40;
        }
    }

    let efi_sys_table = uefi::table::system_table_raw();
    if efi_sys_table.is_none() {
        error!("Error: EFI system table was not found.");
        panic_fn_str("EFI_SYS_TABLE_NOT_FOUND");
    }

    let mut efi_rs_addr: u64 = 0;
    let efi_sys_table = efi_sys_table.unwrap();
    unsafe {
        let efi_sys_table = efi_sys_table.read_unaligned();
        efi_rs_addr = efi_sys_table.runtime_services as u64;
    }

    let k_params = boot_info::KParams {
        fb_data,
        memory_map_size: mem_map_size,
        page_table_num_entries: page_table_info.num_entries(),
        uefi_rs_phys_addr: efi_rs_addr,
    };
    kernel_loader::boot_kernel(
        k_entry_point,
        x86_64::PhysAddr::new(page_table_info.phys_addr()),
        k_params,
    );
}

fn get_gop() -> boot::ScopedProtocol<GraphicsOutput> {
    let gop_handle: Result<Handle, uefi::Error> = boot::get_handle_for_protocol::<GraphicsOutput>();
    if gop_handle.is_err() {
        panic_fn(gop_handle.clone().err().unwrap());
    }

    let gop_handle: Handle = gop_handle.unwrap();
    let gop: Result<boot::ScopedProtocol<GraphicsOutput>, uefi::Error> =
        boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle);
    if gop.is_err() {
        panic_fn(gop.unwrap_err());
    }

    return gop.unwrap();
}

fn get_efi_mmap() -> MemoryMapOwned {
    let mem_map = boot::memory_map(MemoryType::LOADER_DATA);
    if mem_map.is_err() {
        let err_msg: uefi::Error = mem_map.err().unwrap();
        error!("Error getting the memory map: {err_msg}");
        panic_fn_str("MMAP_ERROR");
    }

    let mut mem_map: MemoryMapOwned = mem_map.unwrap();
    mem_map.sort();

    return mem_map;
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
