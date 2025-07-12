use core::cmp::min;
use core::ptr::NonNull;
use log::error;
use uefi::boot;
use uefi::boot::{AllocateType, MemoryType};
use uefi::mem::memory_map::{MemoryMap, MemoryMapOwned};

/// Allocates memory for the physical memory manager (bitmap tree). It will store the physical addresses
/// for each bitmap tree section (each one is 36 KiB and can map 1 GiB) and return the address of that
/// array. That is a 32 KiB (8 pages) array, so free it after mapping it to the corresponding virtual
/// address in [`paging::setup_paging`]. The array has this size so the OS can reach the theoretical
/// limit of 4 TiB of RAM. Returns 0 if it failed.
/// # Params:
/// - mem_map: the EFI memory mapped that MUST be sorted.
pub(crate) fn allocate_memory_for_pmm(mem_map: &MemoryMapOwned) -> u64 {
    let last_entry = mem_map.entries().last();
    if last_entry.is_none() {
        error!("Error allocating memory for Physical Memory Manager: EFI memory map issue.");
        return 0;
    }

    let last_entry = last_entry.unwrap();
    //this is the TOTAL size, including reserved areas and possibly other regions that are not actually
    //in the RAM; we limit it to 4 TiB of RAM :D, if there's more, we still use only those 4 TiB
    let ram_size = min(
        last_entry.phys_start + 0x1000 * last_entry.page_count,
        0x400_0000_0000,
    );

    let section_array: uefi::Result<NonNull<u8>> =
        boot::allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 8);

    if section_array.is_err() {
        let err_msg: uefi::Error = section_array.err().unwrap();
        error!("Error allocating memory for Physical Memory Manager: {err_msg}");
        return 0;
    }

    let section_array = section_array.unwrap();
    let needed_sections = ram_size / 0x4000_0000; //RAM in GiB

    for i in 0..needed_sections {
        //allocate memory for a single PMM section (maps 1 GiB and needs 36 KiB)
        //we allocate as runtime services so that the kernel will set this data as occupied
        let section: uefi::Result<NonNull<u8>> =
            boot::allocate_pages(AllocateType::AnyPages, MemoryType::RUNTIME_SERVICES_DATA, 9);
        if section.is_err() {
            let err_msg: uefi::Error = section.err().unwrap();
            error!("Error allocating memory for Physical Memory Manager section: {err_msg}");
            return 0;
        }

        let section: NonNull<u8> = section.unwrap();
        unsafe {
            *(section_array.byte_add((i * 8) as usize).as_ptr() as *mut u64) =
                section.addr().get() as u64;
        }
    }

    //the unused sections will get zeroed, so the first section holding 0 indicates the end of the
    //array
    for i in needed_sections..4096 {
        unsafe {
            *(section_array.byte_add((i * 8) as usize).as_ptr() as *mut u64) = 0;
        }
    }

    return section_array.addr().get() as u64;
}
