use crate::log;
use boot_info::memory_map::{MemoryMapEntry, MemoryType};
use core::ptr;

pub(crate) struct PageFrameAllocator {
    /// The level where the allocation remained.
    last_level: i32,
    /// The index of the last allocated frame.
    last_index: i32,
    /// The virtual address of the bitmap for this level.
    last_bitmap_virt_addr: u64,
    /// The virtual addresses of the bitmaps for the superior levels, starts at L1 and ends at L4.
    virt_addresses_for_superior_bitmaps: [u64; 4],
    /// The indices for this page. Starts at the index in L1 (not used) and ends at the index for L5.
    bitmap_indices: [u8; 5],
}

impl PageFrameAllocator {
    pub(crate) fn new() -> Self {
        Self {
            last_level: 5,
            last_index: 0,
            last_bitmap_virt_addr: 0,
            virt_addresses_for_superior_bitmaps: [0; 4],
            bitmap_indices: [255; 5],
        }
    }

    pub(crate) fn mark_page_free(phys_addr: u64) {
        let indices: [u64; 5] = get_tables_indices(phys_addr);

        let l5_index: u64 = indices[4];
        let l4_virt_addr: u64 = boot_info::PHYS_BITMAP_MANAGER_ADDRESS + 8 + (l5_index * 0x240_000); //jumps of 2.25 MiB

        let l4_index: u64 = indices[3];
        let l3_virt_addr: u64 = l4_virt_addr + 8 + (l4_index * 0x9000); //jumps of 36 KiB

        let l3_index: u64 = indices[2];
        let l2_virt_addr: u64 = l3_virt_addr + 8 + (l3_index * 0x200); //jumps of 512 B

        let l2_index: u64 = indices[1];
        let l1_virt_addr: u64 = l2_virt_addr + 8 + (l2_index * 8);

        let l1_index: u64 = indices[0];

        unsafe {
            //set the corresponding bit, then check if the bitmap is full; if it's not, return early;
            //otherwise, we need to go to the higher bitmap
            *(l1_virt_addr as *mut u64) &= !(1 << l1_index);
            if *(l1_virt_addr as *mut u64) == u64::MAX {
                return;
            }

            *(l2_virt_addr as *mut u64) &= !(1 << l2_index);
            if *(l2_virt_addr as *mut u64) == u64::MAX {
                return;
            }

            *(l3_virt_addr as *mut u64) &= !(1 << l3_index);
            if *(l3_virt_addr as *mut u64) == u64::MAX {
                return;
            }

            *(l4_virt_addr as *mut u64) &= !(1 << l4_index);
        }
    }

    pub(crate) fn get_continuous_frames(&self, num_frames: u64) -> Option<u64> {
        let mut indices: [u8; 5] = [255; 5];
        let mut needed_frames = num_frames;

        let mut phys_addr: u64 = 0;
        let has_found_chunk: bool =
            Self::go_to_level(5, &mut indices, &mut phys_addr, &mut needed_frames);

        if has_found_chunk && needed_frames == 0 && phys_addr != 0 {
            //mark used
            for i in 0..num_frames {
                mark_page_used(phys_addr + i * 0x1000);
            }

            return Some(phys_addr);
        }

        return None;
    }

    fn go_to_level(
        level: usize,
        indices: &mut [u8; 5],
        start_phys_addr: &mut u64,
        remaining_frames: &mut u64,
    ) -> bool {
        unsafe {
            let bitmap = match level {
                5 => *(boot_info::PHYS_BITMAP_MANAGER_ADDRESS as *const u64),
                4 => {
                    if indices[4] == 255 {
                        panic!("PMM: Invalid index at level 4.");
                    }

                    let virt_addr = boot_info::PHYS_BITMAP_MANAGER_ADDRESS
                        + 8
                        + (indices[4] as u64 * 0x240_000);
                    *(virt_addr as *const u64)
                }
                3 => {
                    if indices[4] == 255 || indices[3] == 255 {
                        panic!("PMM: Invalid index at level 3.");
                    }

                    let virt_addr = boot_info::PHYS_BITMAP_MANAGER_ADDRESS
                        + 8
                        + (indices[4] as u64 * 0x240_000)
                        + 8
                        + (indices[3] as u64 * 0x9000);
                    *(virt_addr as *const u64)
                }
                2 => {
                    if indices[4] == 255 || indices[3] == 255 || indices[2] == 255 {
                        panic!("PMM: Invalid index at level 2.");
                    }

                    let virt_addr = boot_info::PHYS_BITMAP_MANAGER_ADDRESS
                        + 8
                        + (indices[4] as u64 * 0x240_000)
                        + 8
                        + (indices[3] as u64 * 0x9000)
                        + 8
                        + (indices[2] as u64 * 0x200);
                    *(virt_addr as *const u64)
                }
                1 => {
                    if indices[4] == 255
                        || indices[3] == 255
                        || indices[2] == 255
                        || indices[1] == 255
                    {
                        panic!("PMM: Invalid index at level 1.");
                    }

                    let virt_addr = boot_info::PHYS_BITMAP_MANAGER_ADDRESS
                        + 8
                        + (indices[4] as u64 * 0x240_000)
                        + 8
                        + (indices[3] as u64 * 0x9000)
                        + 8
                        + (indices[2] as u64 * 0x200)
                        + 8
                        + (indices[1] as u64 * 8);
                    *(virt_addr as *const u64)
                }
                _ => u64::MAX,
            };

            return if level > 1 {
                let mut has_found_chunk: bool = false;
                for i in 0..64 {
                    if (bitmap & (1 << i)) != 0 {
                        continue;
                    }

                    indices[level - 1] = i as u8;
                    has_found_chunk =
                        Self::go_to_level(level - 1, indices, start_phys_addr, remaining_frames);

                    //exit recursion
                    if has_found_chunk && *remaining_frames == 0 {
                        return true;
                    }
                }

                //go to the superior level
                has_found_chunk
            } else {
                let mut start_index: usize = if *start_phys_addr != 0 { 0 } else { 255 };

                //if this is a new discovery, we start wherever we find a spot;
                //otherwise we must start from 0, so the chunk is continuous
                if *start_phys_addr == 0 {
                    for i in 0..64 {
                        if (bitmap & (1 << i)) == 0 {
                            start_index = i;
                            break;
                        }
                    }

                    if start_index == 255 {
                        return false;
                    }

                    *start_phys_addr = indices[4] as u64 * 0x10_0000_0000
                        + indices[3] as u64 * 0x00_4000_0000
                        + indices[2] as u64 * 0x00_0100_0000
                        + indices[1] as u64 * 0x4_0000
                        + start_index as u64 * 0x1000;
                }

                for i in start_index..64 {
                    if (bitmap & (1 << i)) != 0 {
                        *start_phys_addr = 0;
                        return false;
                    }

                    //we found the chunk
                    if *remaining_frames == 0 {
                        return true;
                    }

                    *remaining_frames -= 1;
                }

                //we ran out of L1 frames, but there might be more on the next L1 frames, so continue
                true
            }
        }
    }

    fn mark_page_used(&self) {
        let l5_index: u64 = self.bitmap_indices[4] as u64;
        let l4_virt_addr: u64 = boot_info::PHYS_BITMAP_MANAGER_ADDRESS + 8 + (l5_index * 0x240_000); //jumps of 2.25 MiB

        let l4_index: u64 = self.bitmap_indices[3] as u64;
        let l3_virt_addr: u64 = l4_virt_addr + 8 + (l4_index * 0x9000); //jumps of 36 KiB

        let l3_index: u64 = self.bitmap_indices[2] as u64;
        let l2_virt_addr: u64 = l3_virt_addr + 8 + (l3_index * 0x200); //jumps of 512 B

        let l2_index: u64 = self.bitmap_indices[1] as u64;
        let l1_virt_addr: u64 = l2_virt_addr + 8 + (l2_index * 8); //jumps of 8 B

        let l1_index: u64 = self.bitmap_indices[0] as u64;

        //set the corresponding bit, then check if the bitmap is full; if it's not, return early;
        //otherwise, we need to go to the higher bitmap
        unsafe {
            *(l1_virt_addr as *mut u64) |= 1 << l1_index;
            if *(l1_virt_addr as *mut u64) != u64::MAX {
                return;
            }

            *(l2_virt_addr as *mut u64) |= 1 << l2_index;
            if *(l2_virt_addr as *mut u64) != u64::MAX {
                return;
            }

            *(l3_virt_addr as *mut u64) |= 1 << l3_index;
            if *(l3_virt_addr as *mut u64) != u64::MAX {
                return;
            }

            *(l4_virt_addr as *mut u64) |= 1 << l4_index;
            if *(l4_virt_addr as *mut u64) != u64::MAX {
                return;
            }
        }
    }
}

impl Iterator for PageFrameAllocator {
    type Item = u64;

    /// Returns the physical address of the next free page frame and marks it as used. Will return
    /// None if there is no more free memory.
    fn next(&mut self) -> Option<Self::Item> {
        while self.last_level >= 1 {
            match self.last_level {
                5 => {
                    for i in self.last_index..64 {
                        let l4_virt_addr: u64 = get_next_free_table(&FrameDataResumed {
                            current_level: 5,
                            idx_in_bitmap: i as u8,
                            bitmap_virt_addr: self.last_bitmap_virt_addr,
                        });

                        if l4_virt_addr == 0 {
                            //no more memory
                            return None;
                        } else {
                            self.last_level -= 1;
                            self.last_index = 0;
                            self.bitmap_indices[4] = i as u8;
                            self.last_bitmap_virt_addr = l4_virt_addr;
                        }
                    }
                }
                4 => {
                    self.virt_addresses_for_superior_bitmaps[3] = self.last_bitmap_virt_addr;

                    for i in self.last_index..64 {
                        let l3_virt_addr: u64 = get_next_free_table(&FrameDataResumed {
                            current_level: 4,
                            idx_in_bitmap: i as u8,
                            bitmap_virt_addr: self.last_bitmap_virt_addr,
                        });

                        if l3_virt_addr == 0 {
                            //this L4 exhausted, go to L5 for the next free L4
                            self.last_level += 1;
                            self.last_index = 0;
                            self.last_bitmap_virt_addr = 0;
                        } else {
                            self.last_level -= 1;
                            self.last_index = 0;
                            self.bitmap_indices[3] = i as u8;
                            self.last_bitmap_virt_addr = l3_virt_addr;
                        }
                    }
                }
                3 => {
                    self.virt_addresses_for_superior_bitmaps[2] = self.last_bitmap_virt_addr;

                    for i in self.last_index..64 {
                        let l2_virt_addr: u64 = get_next_free_table(&FrameDataResumed {
                            current_level: 3,
                            idx_in_bitmap: i as u8,
                            bitmap_virt_addr: self.last_bitmap_virt_addr,
                        });

                        if l2_virt_addr == 0 {
                            //this L3 exhausted, go to L4 for the next free L3
                            self.last_level += 1;
                            self.last_index = 0;
                            self.last_bitmap_virt_addr =
                                self.virt_addresses_for_superior_bitmaps[3];
                        } else {
                            self.last_level -= 1;
                            self.last_index = 0;
                            self.bitmap_indices[2] = i as u8;
                            self.last_bitmap_virt_addr = l2_virt_addr;
                        }
                    }
                }
                2 => {
                    self.virt_addresses_for_superior_bitmaps[1] = self.last_bitmap_virt_addr;

                    for i in self.last_index..64 {
                        let l1_virt_addr: u64 = get_next_free_table(&FrameDataResumed {
                            current_level: 2,
                            idx_in_bitmap: i as u8,
                            bitmap_virt_addr: self.last_bitmap_virt_addr,
                        });

                        if l1_virt_addr == 0 {
                            //this L2 exhausted, go to L3 for the next free L2
                            self.last_level += 1;
                            self.last_index = 0;
                            self.last_bitmap_virt_addr =
                                self.virt_addresses_for_superior_bitmaps[2];
                        } else {
                            self.last_level -= 1;
                            self.last_index = 0;
                            self.bitmap_indices[1] = i as u8;
                            self.last_bitmap_virt_addr = l1_virt_addr;
                        }
                    }
                }
                1 => {
                    self.virt_addresses_for_superior_bitmaps[0] = self.last_bitmap_virt_addr;

                    unsafe {
                        let bitmap: u64 = *(self.last_bitmap_virt_addr as *const u64);
                        let mut page_frame_phys_addr: u64 = 0;

                        for i in self.last_index..64 {
                            if bitmap & (1 << i) == 0 {
                                //this will be the physical address of the frame
                                //@formatter:off
                                page_frame_phys_addr = self.bitmap_indices[4] as u64
                                    * 0x10_0000_0000
                                    + self.bitmap_indices[3] as u64 * 0x00_4000_0000
                                    + self.bitmap_indices[2] as u64 * 0x00_0100_0000
                                    + self.bitmap_indices[1] as u64 * 0x4_0000
                                    + i as u64 * 0x1000;
                                //@formatter:on

                                self.bitmap_indices[0] = i as u8;
                                self.mark_page_used();
                                return Some(page_frame_phys_addr);
                            }
                        }

                        if page_frame_phys_addr == 0 {
                            //this L1 exhausted, go to L2 for the next free L1
                            self.last_level += 1;
                            self.last_index = 0;
                            self.last_bitmap_virt_addr =
                                self.virt_addresses_for_superior_bitmaps[1];
                        }
                    }
                }
                _ => {
                    return None;
                }
            }
        }

        return None;
    }
}

struct FrameDataResumed {
    current_level: u8,
    /// The index in the bitmap where the previous page was allocated.
    idx_in_bitmap: u8,
    /// The virtual address of the bitmap. Jump in the memory accordingly to get to an inferior level.
    bitmap_virt_addr: u64,
}

pub(super) fn init_from_mem_map(mem_map_size: u32) {
    if mem_map_size as usize % size_of::<MemoryMapEntry>() != 0 {
        log::log_error("PMM: mem map is not aligned");
    }

    let num_entries = mem_map_size as usize / size_of::<MemoryMapEntry>();

    for i in 0..num_entries {
        unsafe {
            let data_ptr: *const MemoryMapEntry =
                (boot_info::MEM_MAP_VIRTUAL_ADDRESS as *const MemoryMapEntry).add(i);
            let data = ptr::read_unaligned(data_ptr);

            if data.mem_type() != MemoryType::Conventional
                && data.mem_type() != MemoryType::BootServicesCode
                && data.mem_type() != MemoryType::BootServicesData
            {
                for j in 0..data.num_pages() {
                    mark_page_used(data.physical_addr() + j * 0x1000);
                }
            }
        }
    }

    //get the last entry again to mark the bitmap as "used" for unmapped pages (larger than RAM)
    unsafe {
        let data_ptr: *const MemoryMapEntry = (boot_info::MEM_MAP_VIRTUAL_ADDRESS
            as *const MemoryMapEntry)
            .add(num_entries - 1);
        let data = ptr::read_unaligned(data_ptr);

        let phys_addr: u64 = data.physical_addr() + (data.num_pages() - 1) * 0x1000;
        let indices: [u64; 5] = get_tables_indices(phys_addr);

        let l5_index: u64 = indices[4];
        let l4_virt_addr: u64 = boot_info::PHYS_BITMAP_MANAGER_ADDRESS + 8 + (l5_index * 0x240_000); //jumps of 2.25 MiB;

        let l4_index: u64 = indices[3];
        let l3_virt_addr: u64 = l4_virt_addr + 8 + (l4_index * 0x9000); //jumps of 36 KiB

        let l3_index: u64 = indices[2];
        let l2_virt_addr: u64 = l3_virt_addr + 8 + (l3_index * 0x200); //jumps of 512 B

        let l2_index: u64 = indices[1];
        let l1_virt_addr: u64 = l2_virt_addr + 8 + (l2_index * 8);

        let l1_index: u64 = indices[0];

        *(l1_virt_addr as *mut u64) |= u64::MAX << l1_index + 1;
        *(l2_virt_addr as *mut u64) |= u64::MAX << l2_index + 1;
        *(l3_virt_addr as *mut u64) |= u64::MAX << l3_index + 1;
        *(l4_virt_addr as *mut u64) |= u64::MAX << l4_index + 1;
    }
}

/// Returns the virtual address of the bitmap for the next free table. Only for superior levels 5,
/// 4, 3, and 2. For the others it will return 0.
fn get_next_free_table(superior_level_resume_data: &FrameDataResumed) -> u64 {
    unsafe {
        let bitmap: u64 = *(superior_level_resume_data.bitmap_virt_addr as *const u64);
        for i in superior_level_resume_data.idx_in_bitmap..64 {
            if (bitmap & (1 << i)) == 0 {
                return match superior_level_resume_data.current_level {
                    //jumps of 2.25 MiB
                    5 => boot_info::PHYS_BITMAP_MANAGER_ADDRESS + 8 + (i as u64 * 0x240_000),
                    //jumps of 36 KiB
                    4 => superior_level_resume_data.bitmap_virt_addr + 8 + (i as u64 * 0x9000),
                    //jumps of 512 B
                    3 => superior_level_resume_data.bitmap_virt_addr + 8 + (i as u64 * 0x200),
                    //jumps of 8 B
                    2 => superior_level_resume_data.bitmap_virt_addr + 8 + (i as u64 * 8),
                    _ => 0,
                };
            }
        }

        return 0;
    }
}

fn mark_page_used(phys_addr: u64) {
    let indices: [u64; 5] = get_tables_indices(phys_addr);

    let l5_index: u64 = indices[4];
    let l4_virt_addr: u64 = boot_info::PHYS_BITMAP_MANAGER_ADDRESS + 8 + (l5_index * 0x240_000); //jumps of 2.25 MiB;

    let l4_index: u64 = indices[3];
    let l3_virt_addr: u64 = l4_virt_addr + 8 + (l4_index * 0x9000); //jumps of 36 KiB

    let l3_index: u64 = indices[2];
    let l2_virt_addr: u64 = l3_virt_addr + 8 + (l3_index * 0x200); //jumps of 512 B

    let l2_index: u64 = indices[1];
    let l1_virt_addr: u64 = l2_virt_addr + 8 + (l2_index * 8);

    let l1_index: u64 = indices[0];

    //set the corresponding bit, then check if the bitmap is full; if it's not, return early;
    //otherwise, we need to go to the higher bitmap
    unsafe {
        *(l1_virt_addr as *mut u64) |= 1 << l1_index;
        if *(l1_virt_addr as *mut u64) != u64::MAX {
            return;
        }

        *(l2_virt_addr as *mut u64) |= 1 << l2_index;
        if *(l2_virt_addr as *mut u64) != u64::MAX {
            return;
        }

        *(l3_virt_addr as *mut u64) |= 1 << l3_index;
        if *(l3_virt_addr as *mut u64) != u64::MAX {
            return;
        }

        *(l4_virt_addr as *mut u64) |= 1 << l4_index;
        if *(l4_virt_addr as *mut u64) != u64::MAX {
            return;
        }
    }
}

fn get_tables_indices(mut addr: u64) -> [u64; 5] {
    let l5_index: u64 = addr / 0x10_0000_0000; //each 64 GiB
    if l5_index >= 64 {
        return [0, 0, 0, 0, 0];
    }
    addr %= 0x10_0000_0000;

    let l4_index: u64 = addr / 0x00_4000_0000; //each 1 GiB
    if l4_index >= 64 {
        panic!("PMM: l4_index >= 64");
    }
    addr %= 0x00_4000_0000;

    let l3_index: u64 = addr / 0x00_0100_0000; //each 16 MiB
    if l3_index >= 64 {
        panic!("PMM: l3_index >= 64");
    }
    addr %= 0x00_0100_0000;

    let l2_index: u64 = addr / 0x00_0004_0000; //each 256 KiB
    if l2_index >= 64 {
        panic!("PMM: l2_index >= 64");
    }
    addr %= 0x00_0004_0000;

    let l1_index: u64 = addr / 0x1000;
    if l1_index >= 64 {
        panic!("PMM: l1_index >= 64");
    }
    //addr %= 0x1000;

    return [l1_index, l2_index, l3_index, l4_index, l5_index];
}
