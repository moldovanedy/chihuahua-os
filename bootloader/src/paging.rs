use core::num::NonZero;
use core::ptr::NonNull;
use log::{error, info};
use uefi::boot::{self, AllocateType, MemoryType};
use uefi::mem::memory_map::{self, MemoryMap};
use uefi::proto::console::gop;
use x86_64::structures::paging::page_table::{PageTable, PageTableEntry};
use x86_64::structures::paging::{FrameAllocator, Page, PageTableFlags, PhysFrame, Size4KiB};

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

/// Returns an option for the page table info.
pub fn setup_paging(
    mem_map: &memory_map::MemoryMapOwned,
    gop_fb: &mut gop::FrameBuffer,
    k_physical_address: u64,
    raw_mem_map_physical_address: u64,
    pmm_sections_array: u64,
) -> Option<PageTableInfo> {
    // let num_entries: usize = calculate_page_table_entries(gop_fb, pmm_sections_array);
    // let needed_pages: usize = (num_entries + 511) / 512;

    let page_table_ptr: NonNull<u8> = match allocate_pages(1) {
        Ok(ptr) => ptr,
        Err(err_msg) => {
            error!("Error setting up paging: {err_msg}");
            return None;
        }
    };
    let page_table: *mut PageTable = page_table_ptr.as_ptr() as *mut PageTable;

    unsafe {
        (*page_table).zero();

        let p4_frame =
            PhysFrame::containing_address(x86_64::PhysAddr::new(page_table as *const _ as u64));
        (&mut *page_table)[510]
            .set_frame(p4_frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
    }

    let mut mapper = ManualMapper::new(page_table);

    let success: bool = mmap_kernel(k_physical_address, &mut mapper);
    if !success {
        return None;
    }

    let success: bool = mmap_gop(gop_fb, &mut mapper);
    if !success {
        return None;
    }

    let success: bool = mmap_raw_mem_map(raw_mem_map_physical_address, &mut mapper);
    if !success {
        return None;
    }

    let success: bool = mmap_pmm_sections(pmm_sections_array, &mut mapper);
    if !success {
        return None;
    }

    //map the page tables themselves, so we can access them from the kernel
    // for i in 0..num_entries as u64 {
    for i in 0..1u64 {
        let frame: PhysFrame = PhysFrame::containing_address(x86_64::PhysAddr::new(
            page_table_ptr.as_ptr() as u64 + i * 0x1000,
        ));
        let flags: PageTableFlags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        let page: Page = Page::containing_address(x86_64::VirtAddr::new(
            boot_info::PAGE_TABLES_ADDRESS + i * 0x1000,
        ));

        let success = mapper.map_to(page, frame, flags);
        if !success {
            error!("Error mapping a the page tables themselves.");
            return None;
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
            let page: Page =
                Page::containing_address(x86_64::VirtAddr::new(map_entry.phys_start + i * 0x1000));

            let success = mapper.map_to(page, frame, flags);
            if !success {
                error!("Error mapping the identity map.");
                return None;
            }
        }
    }

    return Some(PageTableInfo {
        phys_addr: page_table_ptr.as_ptr() as u64,
        // num_entries: num_entries as u64,
        num_entries: 1u64,
    });
}

fn mmap_kernel(k_physical_address: u64, mapper: &mut ManualMapper) -> bool {
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

        let success = mapper.map_to(page, frame, flags);
        if !success {
            error!("Error mapping a physical page to kernel memory.");
            return false;
        }

        needed_memory -= 0x1000;
        i += 1;
    }

    return true;
}

fn mmap_gop(gop_fb: &mut gop::FrameBuffer, mapper: &mut ManualMapper) -> bool {
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

        let success = mapper.map_to(page, frame, flags);
        if !success {
            error!("Error mapping a physical page to GOP memory.");
            return false;
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

fn mmap_raw_mem_map(efi_mmap_phys_addr: u64, mapper: &mut ManualMapper) -> bool {
    for i in 0..MEM_MAP_NEEDED_PAGES {
        let frame: PhysFrame =
            PhysFrame::containing_address(x86_64::PhysAddr::new(efi_mmap_phys_addr + i * 0x1000));
        let flags: PageTableFlags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        let page: Page = Page::containing_address(x86_64::VirtAddr::new(
            boot_info::MEM_MAP_VIRTUAL_ADDRESS + i * 0x1000,
        ));

        let success = mapper.map_to(page, frame, flags);
        if !success {
            error!("Error mapping the direct memory map.");
            return false;
        }
    }

    return true;
}

fn mmap_pmm_sections(pmm_sections_array: u64, mapper: &mut ManualMapper) -> bool {
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

                let success = mapper.map_to(page, frame, flags);
                if !success {
                    error!("Error mapping the PMM sections.");
                    return false;
                }
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

fn allocate_pages(needed_pages: usize) -> Result<NonNull<u8>, uefi::Error> {
    boot::allocate_pages(
        AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        needed_pages,
    )
}

struct ManualMapper {
    p4: *mut PageTable,
    frame_allocator: UefiFrameAllocator,
}

impl ManualMapper {
    fn new(p4: *mut PageTable) -> Self {
        Self {
            p4,
            frame_allocator: UefiFrameAllocator {},
        }
    }

    fn map_to(&mut self, page: Page, frame: PhysFrame, flags: PageTableFlags) -> bool {
        let p4 = unsafe { &mut *self.p4 };

        let p3_idx = page.start_address().p4_index();
        let p3 = match Self::get_or_create_table(&mut self.frame_allocator, &mut p4[p3_idx]) {
            Some(table) => table,
            None => return false,
        };

        let p2_idx = page.start_address().p3_index();
        let p2 = match Self::get_or_create_table(&mut self.frame_allocator, &mut p3[p2_idx]) {
            Some(table) => table,
            None => return false,
        };

        let p1_idx = page.start_address().p2_index();
        let p1 = match Self::get_or_create_table(&mut self.frame_allocator, &mut p2[p1_idx]) {
            Some(table) => table,
            None => return false,
        };

        let p1_idx = page.start_address().p1_index();
        p1[p1_idx].set_frame(frame, flags);

        true
    }

    fn get_or_create_table(
        allocator: &mut UefiFrameAllocator,
        entry: &mut PageTableEntry,
    ) -> Option<&'static mut PageTable> {
        if entry.is_unused() {
            let frame = allocator.allocate_frame()?;
            let table_ptr = frame.start_address().as_u64() as *mut PageTable;
            unsafe {
                (*table_ptr).zero();
            }
            entry.set_frame(frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
        }

        let table_ptr = entry.frame().ok()?.start_address().as_u64() as *mut PageTable;
        Some(unsafe { &mut *table_ptr })
    }
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
