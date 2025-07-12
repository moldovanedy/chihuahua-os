use core::ptr::NonNull;
use log::error;
use uefi::boot;
use uefi::boot::{AllocateType, MemoryType};
use uefi::mem::memory_map::{MemoryMap, MemoryMapOwned};

/// Returns the physical address of the raw map, as well as the number of pages taken by it.
pub fn alloc_memory_for_map(mem_map: &MemoryMapOwned) -> Option<(u64, u32)> {
    //each entry will occupy 40 bytes, but we will add 10 as a "safe zone" (in case there will
    //be any allocations after this); then the total size is divided by 4096 to get the page count
    let pages_needed: usize = (((mem_map.entries().len() + 10) * 40) / 0x1000) + 1;
    
    let addr: uefi::Result<NonNull<u8>> = boot::allocate_pages(
        AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        pages_needed,
    );
    if addr.is_err() {
        error!("Error allocating memory for the direct memory map.");
        return None;
    }

    let addr = addr.unwrap().as_ptr() as u64;
    if addr == 0 {
        return None;
    }

    return Some((addr, pages_needed as u32));
}
