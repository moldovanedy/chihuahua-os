use core::ptr::NonNull;
use log::error;
use uefi::boot;
use uefi::boot::{AllocateType, MemoryType};

const NUM_PAGES_NEEDED: usize = 16;

pub fn alloc_memory_for_efi_map() -> Option<u64> {
    let addr: uefi::Result<NonNull<u8>> = boot::allocate_pages(
        AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        NUM_PAGES_NEEDED,
    );
    if addr.is_err() {
        error!("Error allocating memory for the EFI memory map.");
        return None;
    }

    let addr = addr.unwrap().as_ptr() as u64;
    if addr == 0 {
        return None;
    }

    return Some(addr);
}
