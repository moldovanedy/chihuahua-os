use log::error;
use uefi::boot::{self, MemoryType};
use x86_64::structures::paging::page_table::PageTable;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, PageTableFlags, PhysFrame, Size4KiB,
};

pub fn setup_identity_paging() -> x86_64::PhysAddr {
    let page_table_ptr =
        boot::allocate_pages(boot::AllocateType::AnyPages, MemoryType::LOADER_DATA, 4);
    if page_table_ptr.is_err() {
        let err_msg: uefi::Error = page_table_ptr.err().unwrap();
        error!("Error setting up paging: {err_msg}");
        return x86_64::PhysAddr::zero();
    }

    let page_table_ptr: core::ptr::NonNull<u8> = page_table_ptr.unwrap();
    let page_table: *mut PageTable = page_table_ptr.as_ptr() as *mut PageTable;

    unsafe {
        (*page_table).zero();
    }

    let phys_mem_offset = x86_64::VirtAddr::new(0);
    let mut mapper: OffsetPageTable<'_> =
        unsafe { OffsetPageTable::new(&mut *page_table, phys_mem_offset) };
    let mut frame_allocator = UefiFrameAllocator {};

    //identity-map 4 GiB
    for i in 0..0xfffff {
        let frame: PhysFrame = PhysFrame::containing_address(x86_64::PhysAddr::new(i * 0x1000));
        let flags: PageTableFlags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        //let page: Page = Page::containing_address(x86_64::VirtAddr::new(i * 0x1000));

        unsafe {
            // mapper
            //     .map_to(page, frame, flags, &mut frame_allocator)
            //     .unwrap()
            //     .flush();

            let mapper_flush = mapper.identity_map(frame, flags, &mut frame_allocator);
            if mapper_flush.is_err() {
                error!("Error setting up paging: frame could not be identity-mapped");
                return x86_64::PhysAddr::zero();
            }

            let mapper_flush = mapper_flush.unwrap();
            mapper_flush.flush();
        }
    }

    return x86_64::PhysAddr::new(page_table_ptr.as_ptr() as u64);
}

struct UefiFrameAllocator {}

unsafe impl FrameAllocator<Size4KiB> for UefiFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let page = boot::allocate_pages(boot::AllocateType::AnyPages, MemoryType::LOADER_DATA, 1);
        if page.is_err() {
            return None;
        }

        let page: u64 = page.unwrap().as_ptr() as u64;
        return Some(PhysFrame::containing_address(x86_64::PhysAddr::new(page)));
    }
}
