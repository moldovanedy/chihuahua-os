use core::num::NonZero;
use core::ptr::NonNull;
use log::{error, info};
use uefi::boot::{self, AllocateType, MemoryType};
use uefi::mem::memory_map::{self, MemoryMap};
use uefi::proto::console::gop;
use x86_64::structures::paging::mapper::{MapToError, MapperFlush};
use x86_64::structures::paging::page_table::PageTable;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, PhysFrame, Size4KiB,
};

const KERNEL_SIZE: u64 = 0x100000;
const MEM_MAP_NEEDED_PAGES: u64 = 16;

pub struct PageTableInfo {
    /// The physical address of the page table (on x86_64 this will be written in CR3).
    phys_addr: u64,
    /// The number of entries in this page table.
    num_entries: u64,
}

impl PageTableInfo {
    pub fn phys_addr(&self) -> u64 {
        self.phys_addr
    }

    pub fn num_entries(&self) -> u64 {
        self.num_entries
    }
}

///Returns an option for the page table info.
pub fn setup_paging(
    mem_map: &memory_map::MemoryMapOwned,
    gop_fb: &mut gop::FrameBuffer,
    k_physical_address: u64,
    raw_mem_map_physical_address: u64,
    pmm_sections_array: u64,
) -> Option<PageTableInfo> {
    let num_entries: usize = calculate_page_table_entries(gop_fb, pmm_sections_array);
    let needed_pages: usize = (num_entries + 511) / 512;

    let page_table_ptr: uefi::Result<NonNull<u8>> = boot::allocate_pages(
        AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        needed_pages,
    );
    if page_table_ptr.is_err() {
        let err_msg: uefi::Error = page_table_ptr.err().unwrap();
        error!("Error setting up paging: {err_msg}");
        return None;
    }

    let page_table_ptr: NonNull<u8> = page_table_ptr.unwrap();
    let page_table: *mut PageTable = page_table_ptr.as_ptr() as *mut PageTable;

    unsafe {
        (*page_table).zero();
        (*(page_table.add(1))).zero();
        (*(page_table.add(2))).zero();
        (*(page_table.add(3))).zero();
        (*(page_table.add(4))).zero();
    }

    let mut mapper: OffsetPageTable<'_> =
        unsafe { OffsetPageTable::new(&mut *page_table, x86_64::VirtAddr::new(0)) };
    let mut frame_allocator = UefiFrameAllocator {};

    let success: bool = mmap_kernel(k_physical_address, page_table);
    if !success {
        return None;
    }

    let success: bool = mmap_gop(gop_fb, page_table);
    if !success {
        return None;
    }

    let success: bool = mmap_raw_mem_map(raw_mem_map_physical_address, page_table);
    if !success {
        return None;
    }

    let success: bool = mmap_pmm_sections(pmm_sections_array, page_table);
    if !success {
        return None;
    }

    //map the page tables themselves, so we can access them from the kernel
    for i in 0..needed_pages as u64 {
        let frame: PhysFrame = PhysFrame::containing_address(x86_64::PhysAddr::new(
            page_table_ptr.as_ptr() as u64 + i * 0x1000,
        ));
        let flags: PageTableFlags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        let page: Page = Page::containing_address(x86_64::VirtAddr::new(
            boot_info::PAGE_TABLES_ADDRESS + i * 0x1000,
        ));

        unsafe {
            let mapper_flush: Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>> =
                mapper.map_to(page, frame, flags, &mut frame_allocator);
            if mapper_flush.is_err() {
                error!("Error mapping a the page tables themselves.");
                return None;
            }

            mapper_flush.unwrap().flush();
        }
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

    return Some(PageTableInfo {
        phys_addr: page_table_ptr.as_ptr() as u64,
        num_entries: num_entries as u64,
    });
}

fn mmap_kernel(k_physical_address: u64, page_table: *mut PageTable) -> bool {
    let mut mapper: OffsetPageTable<'_> =
        unsafe { OffsetPageTable::new(&mut *page_table, x86_64::VirtAddr::new(0)) };
    let mut frame_allocator = UefiFrameAllocator {};

    //1 MiB
    let mut needed_memory: u64 = KERNEL_SIZE;
    let mut i: u64 = 0;

    while needed_memory > 0 {
        let frame: PhysFrame =
            PhysFrame::containing_address(x86_64::PhysAddr::new(k_physical_address + i * 0x1000));
        let flags: PageTableFlags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        let page: Page = Page::containing_address(x86_64::VirtAddr::new(
            boot_info::KERNEL_VIRTUAL_ADDRESS + i * 0x1000,
        ));

        unsafe {
            let mapper_flush: Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>> =
                mapper.map_to(page, frame, flags, &mut frame_allocator);
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

fn mmap_gop(mut gop_fb: &mut gop::FrameBuffer, page_table: *mut PageTable) -> bool {
    let mut mapper: OffsetPageTable<'_> =
        unsafe { OffsetPageTable::new(&mut *page_table, x86_64::VirtAddr::new(0)) };
    let mut frame_allocator = UefiFrameAllocator {};

    let mut needed_memory: u64 = gop_fb.size() as u64;
    let base_physical_address: u64 = gop_fb.as_mut_ptr() as u64;

    let mut i: u64 = 0;
    //can't have more than 65_536 pages of 4 KiB for a framebuffer
    while needed_memory > 0 && i < 0x1_00_00 {
        let frame: PhysFrame = PhysFrame::containing_address(x86_64::PhysAddr::new(
            base_physical_address + i * 0x1000,
        ));
        let flags: PageTableFlags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        let page: Page = Page::containing_address(x86_64::VirtAddr::new(
            boot_info::GOP_VIRTUAL_ADDRESS + i * 0x1000,
        ));

        unsafe {
            let mapper_flush: Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>> =
                mapper.map_to(page, frame, flags, &mut frame_allocator);
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

fn mmap_raw_mem_map(efi_mmap_phys_addr: u64, page_table: *mut PageTable) -> bool {
    let mut mapper: OffsetPageTable<'_> =
        unsafe { OffsetPageTable::new(&mut *page_table, x86_64::VirtAddr::new(0)) };
    let mut frame_allocator = UefiFrameAllocator {};

    for i in 0..MEM_MAP_NEEDED_PAGES {
        let frame: PhysFrame =
            PhysFrame::containing_address(x86_64::PhysAddr::new(efi_mmap_phys_addr + i * 0x1000));
        let flags: PageTableFlags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        let page: Page = Page::containing_address(x86_64::VirtAddr::new(
            boot_info::MEM_MAP_VIRTUAL_ADDRESS + i * 0x1000,
        ));

        unsafe {
            let mapper_flush: Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>> =
                mapper.map_to(page, frame, flags, &mut frame_allocator);
            if mapper_flush.is_err() {
                error!("Error mapping the direct memory map.");
                return false;
            }

            mapper_flush.unwrap().flush();
        }
    }

    return true;
}

fn mmap_pmm_sections(pmm_sections_array: u64, page_table: *mut PageTable) -> bool {
    let mut mapper: OffsetPageTable<'_> =
        unsafe { OffsetPageTable::new(&mut *page_table, x86_64::VirtAddr::new(0)) };
    let mut frame_allocator = UefiFrameAllocator {};

    let mut i: u32 = 0;
    let mut array_ptr: *mut u64 = pmm_sections_array as *mut u64;

    unsafe {
        while *array_ptr != 0 && i < 4096 {
            //9 pages in a section (36 KiB)
            for pg_in_section in 0..9 {
                //base + the current section (that's why we have 36 KiB jumps) + the current
                //page of the section
                let virt_addr: u64 = boot_info::PHYS_BITMAP_MANAGER_ADDRESS
                    + (i * 9 * 0x1000) as u64
                    + pg_in_section * 0x1000;

                let frame: PhysFrame = PhysFrame::containing_address(x86_64::PhysAddr::new(
                    *array_ptr + pg_in_section * 0x1000,
                ));
                let flags: PageTableFlags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
                let page: Page = Page::containing_address(x86_64::VirtAddr::new(virt_addr));

                let mapper_flush: Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>> =
                    mapper.map_to(page, frame, flags, &mut frame_allocator);
                if mapper_flush.is_err() {
                    error!("Error mapping the PMM sections.");
                    return false;
                }

                mapper_flush.unwrap().flush();
            }

            i += 1;
            array_ptr = array_ptr.add(1);
        }
    }

    //we're done with the large array, free it (will make the EFI memory map outdated, but it shouldn't
    //be a problem)
    unsafe {
        let addr = NonZero::new(pmm_sections_array as usize);
        if addr.is_some() {
            let _ = boot::free_pages(NonNull::without_provenance(addr.unwrap()), 8);
        }
    }

    return true;
}

/// Returns the number of necessary entries for the page table itself.
fn calculate_page_table_entries(gop_fb: &mut gop::FrameBuffer, pmm_sections_array: u64) -> usize {
    let mut array_ptr: *mut u64 = pmm_sections_array as *mut u64;
    let mut pmm_needed_pages: u64 = 0;

    let mut i = 0;
    unsafe {
        while *array_ptr != 0 && i < 4096 {
            pmm_needed_pages += 9;
            i += 1;
            array_ptr = array_ptr.add(1);
        }
    }

    let page_count = KERNEL_SIZE / 0x1000
        + gop_fb.size() as u64 / 0x1000
        + MEM_MAP_NEEDED_PAGES
        + pmm_needed_pages;

    //each page table section holds information about 512 frames (4096 / 8)
    return page_count as usize;
}

pub(crate) struct UefiFrameAllocator {}

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
