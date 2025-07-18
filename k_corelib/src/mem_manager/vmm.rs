use crate::log::log_debug;
use crate::mem_manager::pmm;
use crate::mem_manager::pmm::PageFrameAllocator;
use boot_info::memory_map::MemoryMapEntry;
use core::ptr;
use dog_essentials::format_non_alloc::u64_to_str_base;
use dog_essentials::static_cell::StaticCell;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::mapper::{MapToError, MapperFlush};
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
};

static K_PAGE_TABLE: StaticCell<u64> = StaticCell::new(0);
static K_PAGE_TABLE_LAST_ENTRY_IDX: StaticCell<u64> = StaticCell::new(0);
/// The number of pages occupied by the page table itself.
static K_PAGE_TABLE_NUM_PAGES: StaticCell<u64> = StaticCell::new(0);

static K_HEAP_REAL_END: StaticCell<u64> = StaticCell::new(boot_info::K_HEAP_START);

pub fn init(mem_map_size: u32, num_entries: u64) {
    unsafe {
        K_PAGE_TABLE.set_value_unsafe(Cr3::read().0.start_address().as_u64());
        K_PAGE_TABLE_LAST_ENTRY_IDX.set_value_unsafe(num_entries - 1);
        K_PAGE_TABLE_NUM_PAGES.set_value_unsafe((num_entries + 511) / 512);
    }

    unsafe {
        let num_entries = mem_map_size as usize / size_of::<MemoryMapEntry>();
        let data_ptr: *const MemoryMapEntry =
            (boot_info::MEM_MAP_VIRTUAL_ADDRESS as *const MemoryMapEntry).add(num_entries - 1);
        let data = ptr::read_unaligned(data_ptr);

        //the size of RAM in bytes
        let memory_size: u64 = data.physical_addr() + (data.num_pages() - 1) * 0x1000;

        //zero-out the PMM bitmaps
        ptr::write_bytes(
            boot_info::PHYS_BITMAP_MANAGER_ADDRESS as *mut u8,
            0,
            (memory_size / (1 << 30) + 1) as usize * 0x9000,
        );
    }

    pmm::init_from_mem_map(mem_map_size);
    //TODO: make this not page fault
    //expand_kernel_heap(16);
}

/// Expands the kernel heap space by the given number of pages. Returns true if it succeeded, false
/// otherwise. It is similar to sbrk on Linux.
pub fn expand_kernel_heap(num_pages: u32) -> bool {
    let current_last_entry_idx: u64 = *K_PAGE_TABLE_LAST_ENTRY_IDX.get_value_unsafe();
    let desired_last_entry_idx: u64 = current_last_entry_idx + num_pages as u64;
    let mut needed_enlargement: bool = false;

    //if we also need to enlarge the page table
    if (desired_last_entry_idx + 1 + 511) / 512 > *K_PAGE_TABLE_NUM_PAGES.get_value_unsafe() {
        let new_table: Option<NewPageTableInfo> = find_new_chunk_for_page_table(
            *K_PAGE_TABLE.get_value_unsafe(),
            current_last_entry_idx,
            desired_last_entry_idx,
        );

        if new_table.is_none() {
            return false;
        }

        let new_table: NewPageTableInfo = new_table.unwrap();
        K_PAGE_TABLE.set_value_unsafe(new_table.phys_addr);
        K_PAGE_TABLE_NUM_PAGES.set_value_unsafe(new_table.num_frames_occupied);
        needed_enlargement = true;
    }

    let mut allocator = PageFrameAllocator::new();
    let prev_heap_end: u64 = *K_HEAP_REAL_END.get_value_unsafe();

    let table_phys_addr: u64 = *K_PAGE_TABLE.get_value_unsafe();
    let page_table_ptr: *mut PageTable = x86_64::VirtAddr::new(table_phys_addr).as_mut_ptr();

    let mut mapper: OffsetPageTable<'_> = unsafe {
        OffsetPageTable::new(
            &mut *page_table_ptr,
            x86_64::VirtAddr::new(boot_info::PAGE_TABLES_ADDRESS),
        )
    };
    let mut frame_allocator: PhysFrameAllocator = PhysFrameAllocator::new();

    for i in 0..num_pages as u64 {
        log_debug("BP1");
        let frame: Option<u64> = allocator.next();

        if frame.is_none() {
            return false;
        }

        let frame: PhysFrame = PhysFrame::containing_address(x86_64::PhysAddr::new(frame.unwrap()));
        let flags: PageTableFlags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        let virt_addr = prev_heap_end + i * 0x1000;
        log_debug(u64_to_str_base(frame.start_address().as_u64(), 16).to_str());
        let page: Page = Page::containing_address(x86_64::VirtAddr::new(virt_addr));

        unsafe {
            log_debug("Before map_to");
            //EXCEPTION: Page fault... We need to rethink the entire paging implementation
            let mapper_flush: Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>> =
                mapper.map_to(page, frame, flags, &mut frame_allocator);
            log_debug("After map_to");

            if mapper_flush.is_err() {
                log_debug("FAIL");
                log_debug(u64_to_str_base(virt_addr, 16).to_str());
                return false;
            }

            log_debug("Before flush");
            mapper_flush.unwrap().flush();
            log_debug("After flush");
        }

        // let frame: PhysFrame = PhysFrame::containing_address(x86_64::PhysAddr::new(frame.unwrap()));
        // let flags: PageTableFlags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        // let page: Page =
        //     Page::containing_address(x86_64::VirtAddr::new(prev_heap_end + i * 0x1000));
        //
        // unsafe {
        //     let mapper_flush: Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>> =
        //         mapper.map_to(page, frame, flags, &mut frame_allocator);
        //     if mapper_flush.is_err() {
        //         log_debug("BP14");
        //         return false;
        //     }
        //
        //     log_debug("BP15");
        //     mapper_flush.unwrap().flush();
        // }
    }

    log_debug("BP4");
    K_HEAP_REAL_END.set_value_unsafe(prev_heap_end + num_pages as u64 * 0x1000);

    if needed_enlargement {
        log_debug("BP5");
        unsafe {
            #[cfg(target_arch = "x86_64")]
            {
                Cr3::write(
                    PhysFrame::containing_address(x86_64::PhysAddr::new(
                        *K_PAGE_TABLE.get_value_unsafe(),
                    )),
                    Cr3::read().1,
                );
            }
        }
    }

    return true;
}

pub struct NewPageTableInfo {
    phys_addr: u64,
    num_frames_occupied: u64,
}

/// Returns an option for a tuple with the new physical address of the page table and the new
/// size of the page table (in frames).
pub fn find_new_chunk_for_page_table(
    current_phys_addr: u64,
    current_last_entry_idx: u64,
    desired_last_entry_idx: u64,
) -> Option<NewPageTableInfo> {
    let size_needed: u64 = (desired_last_entry_idx + 1 + 511) / 512;
    let phys_addr = pmm::PageFrameAllocator::get_continuous_frames(size_needed);
    if phys_addr.is_none() {
        return None;
    }

    let phys_addr: u64 = phys_addr.unwrap();
    unsafe {
        core::ptr::copy(
            current_phys_addr as *const u8,
            phys_addr as *mut u8,
            ((current_last_entry_idx + 1) * 8) as usize,
        );
    }

    //will free the previous chunk
    let prev_size: u64 = (current_last_entry_idx + 1 + 511) / 512;
    for i in 0..prev_size {
        pmm::PageFrameAllocator::mark_page_free(current_phys_addr + i * 0x1000);
    }

    return Some(NewPageTableInfo {
        phys_addr,
        num_frames_occupied: size_needed,
    });
}

pub(crate) struct PhysFrameAllocator {
    allocator: PageFrameAllocator,
}

impl PhysFrameAllocator {
    const fn new() -> Self {
        PhysFrameAllocator {
            allocator: PageFrameAllocator::new(),
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for PhysFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        log_debug("AA1");
        let phys_addr = self.allocator.next();
        if phys_addr.is_none() {
            log_debug("AA2");
            return None;
        }

        log_debug("AA3");
        log_debug(u64_to_str_base(phys_addr.unwrap(), 16).to_str());
        return Some(PhysFrame::containing_address(x86_64::PhysAddr::new(
            phys_addr.unwrap(),
        )));
    }
}
