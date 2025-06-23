use core::ptr::NonNull;
use log::error;
use uefi::boot::{self, AllocateType, MemoryType};
use uefi::mem::memory_map::{self, MemoryMap};
use uefi::proto::console::gop;
use x86_64::structures::paging::mapper::{MapToError, MapperFlush};
use x86_64::structures::paging::page_table::PageTable;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, PhysFrame, Size4KiB,
};

///An option of a tuple where the first element is the physical address of the page table and the
/// second element is the physical address of the EFI memory map reserved region.
pub fn setup_paging(
    mem_map: &memory_map::MemoryMapOwned,
    gop_fb: gop::FrameBuffer,
    k_physical_address: u64,
) -> Option<(u64, u64)> {
    let page_table_ptr = boot::allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 4);
    if page_table_ptr.is_err() {
        let err_msg: uefi::Error = page_table_ptr.err().unwrap();
        error!("Error setting up paging: {err_msg}");
        return None;
    }

    let page_table_ptr: NonNull<u8> = page_table_ptr.unwrap();
    let page_table: *mut PageTable = page_table_ptr.as_ptr() as *mut PageTable;

    unsafe {
        (*page_table).zero();
    }

    let mut mapper: OffsetPageTable<'_> =
        unsafe { OffsetPageTable::new(&mut *page_table, x86_64::VirtAddr::new(0)) };
    let mut frame_allocator = UefiFrameAllocator {};

    let success: bool = mmap_kernel(k_physical_address, page_table);
    if !success {
        return None;
    }

    let success: bool = mmap_gop(page_table, gop_fb);
    if !success {
        return None;
    }

    //identity-map 4 GiB
    // for i in 0..0xfffff {
    //     let frame: PhysFrame = PhysFrame::containing_address(x86_64::PhysAddr::new(i * 0x1000));
    //     let flags: PageTableFlags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    //     //let page: Page = Page::containing_address(x86_64::VirtAddr::new(i * 0x1000));
    //
    //     unsafe {
    //         // mapper
    //         //     .map_to(page, frame, flags, &mut frame_allocator)
    //         //     .unwrap()
    //         //     .flush();
    //
    //         let mapper_flush = mapper.identity_map(frame, flags, &mut frame_allocator);
    //         if mapper_flush.is_err() {
    //             error!("Error setting up paging: frame could not be identity-mapped");
    //             return x86_64::PhysAddr::zero();
    //         }
    //
    //         let mapper_flush = mapper_flush.unwrap();
    //         mapper_flush.flush();
    //     }
    // }

    let efi_map_addr = alloc_memory_for_efi_map(page_table);
    if efi_map_addr.is_none() {
        error!("Error finding memory for the EFI memory map.");
        return None;
    }

    //identity-map all the UEFI-used memory so we can continue without any problems
    for map_entry in mem_map.entries() {
        if map_entry.ty == MemoryType::CONVENTIONAL {
            continue;
        }

        for i in 0..map_entry.page_count {
            let frame: PhysFrame = PhysFrame::containing_address(x86_64::PhysAddr::new(
                map_entry.phys_start + i * 0x1000,
            ));
            let flags: PageTableFlags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

            unsafe {
                let mapper_flush: Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>> =
                    mapper.identity_map(frame, flags, &mut frame_allocator);
                if mapper_flush.is_err() {
                    error!("Error mapping a physical page to kernel memory.");
                    return None;
                }

                mapper_flush.unwrap().flush();
            }
        }
    }

    // for i in 0..10 {
    //     let frame: PhysFrame =
    //         PhysFrame::containing_address(x86_64::PhysAddr::new(0x9000_0000 + i * 0x1000));
    //     let flags: PageTableFlags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    //     let page: Page = Page::containing_address(x86_64::VirtAddr::new(
    //         boot_info::EFI_MMAP_VIRTUAL_ADDRESS + i * 0x1000,
    //     ));
    //
    //     unsafe {
    //         let mapper_flush = mapper.map_to(page, frame, flags, &mut frame_allocator);
    //         if mapper_flush.is_err() {
    //             error!("Error mapping the EFI memory map.");
    //             return x86_64::PhysAddr::new(0);
    //         }
    //
    //         mapper_flush.unwrap().flush();
    //     }
    // }

    return Some((page_table_ptr.as_ptr() as u64, efi_map_addr.unwrap()));
}

pub fn mmap_efi_map(phys_address: u64, size: u32, page_table: *mut PageTable) -> bool {
    let mut mapper: OffsetPageTable<'_> =
        unsafe { OffsetPageTable::new(&mut *page_table, x86_64::VirtAddr::new(0)) };
    let mut frame_allocator = UefiFrameAllocator {};

    let mut needed_memory: u32 = size;
    let mut i: u64 = 0;

    while needed_memory > 0 {
        let frame: PhysFrame =
            PhysFrame::containing_address(x86_64::PhysAddr::new(phys_address + i * 0x1000));
        let flags: PageTableFlags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        // let page: Page =
        //     Page::containing_address(x86_64::VirtAddr::new(crate::EFI_MMAP_VIRTUAL_ADDRESS + i * 0x1000));

        unsafe {
            let mapper_flush = mapper.identity_map(frame, flags, &mut frame_allocator);
            if mapper_flush.is_err() {
                return false;
            }

            mapper_flush.unwrap().flush();
        }

        needed_memory -= 0x1000;
        i += 1;
    }

    return true;
}

fn mmap_kernel(k_physical_address: u64, page_table: *mut PageTable) -> bool {
    let mut mapper: OffsetPageTable<'_> =
        unsafe { OffsetPageTable::new(&mut *page_table, x86_64::VirtAddr::new(0)) };
    let mut frame_allocator = UefiFrameAllocator {};

    //1 MiB
    let mut needed_memory: u64 = 0x100000;
    let mut i: u64 = 0;

    while needed_memory > 0 {
        let frame: PhysFrame =
            PhysFrame::containing_address(x86_64::PhysAddr::new(k_physical_address + i * 0x1000));
        let flags: PageTableFlags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        let page: Page = Page::containing_address(x86_64::VirtAddr::new(
            boot_info::KERNEL_VIRTUAL_ADDRESS + i * 0x1000,
        ));

        unsafe {
            let mapper_flush = mapper.map_to(page, frame, flags, &mut frame_allocator);
            if mapper_flush.is_err() {
                error!("Error mapping a physical page to kernel memory.");
                return false;
            }

            mapper_flush.unwrap().flush();
        }

        needed_memory -= 0x1000;
        i += 1;
    }

    return true;
}

fn mmap_gop(page_table: *mut PageTable, mut gop_fb: gop::FrameBuffer) -> bool {
    let mut mapper: OffsetPageTable<'_> =
        unsafe { OffsetPageTable::new(&mut *page_table, x86_64::VirtAddr::new(0)) };
    let mut frame_allocator = UefiFrameAllocator {};

    let mut needed_memory: u64 = gop_fb.size() as u64;
    let base_physical_address: u64 = gop_fb.as_mut_ptr() as u64;

    let mut i: u64 = 0;
    //can't have more than 65536 pages of 4 KiB for a framebuffer
    while needed_memory > 0 && i < 0x1_00_00 {
        let frame: PhysFrame = PhysFrame::containing_address(x86_64::PhysAddr::new(
            base_physical_address + i * 0x1000,
        ));
        let flags: PageTableFlags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        let page: Page = Page::containing_address(x86_64::VirtAddr::new(
            boot_info::GOP_VIRTUAL_ADDRESS + i * 0x1000,
        ));

        unsafe {
            let mapper_flush = mapper.map_to(page, frame, flags, &mut frame_allocator);
            if mapper_flush.is_err() {
                error!("Error mapping a physical page to GOP memory.");
                return false;
            }

            mapper_flush.unwrap().flush();
        }

        needed_memory -= 0x1000;
        i += 1;
    }

    //just in case...
    if i >= 0x1_00_00 {
        error!("Framebuffer is too big.");
        return false;
    }

    return true;
}

fn alloc_memory_for_efi_map(page_table: *mut PageTable) -> Option<u64> {
    let mut mapper: OffsetPageTable<'_> =
        unsafe { OffsetPageTable::new(&mut *page_table, x86_64::VirtAddr::new(0)) };
    let mut frame_allocator = UefiFrameAllocator {};

    let addr: uefi::Result<NonNull<u8>> =
        boot::allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 16);
    if addr.is_err() {
        error!("Error allocating memory for the EFI memory map.");
        return None;
    }

    let addr = addr.unwrap().as_ptr() as u64;
    if addr == 0 {
        return None;
    }

    for i in 0..16 {
        let frame: PhysFrame =
            PhysFrame::containing_address(x86_64::PhysAddr::new(addr + i * 0x1000));
        let flags: PageTableFlags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        let page: Page = Page::containing_address(x86_64::VirtAddr::new(
            boot_info::EFI_MMAP_VIRTUAL_ADDRESS + i * 0x1000,
        ));

        unsafe {
            let mapper_flush = mapper.map_to(page, frame, flags, &mut frame_allocator);
            if mapper_flush.is_err() {
                error!("Error mapping the EFI memory map.");
                return None;
            }

            mapper_flush.unwrap().flush();
        }
    }

    return Some(addr);
}

struct UefiFrameAllocator {}

unsafe impl FrameAllocator<Size4KiB> for UefiFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let page = boot::allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1);
        if page.is_err() {
            return None;
        }

        let page: u64 = page.unwrap().as_ptr() as u64;
        return Some(PhysFrame::containing_address(x86_64::PhysAddr::new(page)));
    }
}
