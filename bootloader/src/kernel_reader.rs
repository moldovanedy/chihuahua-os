use elf;
use elf::endian::LittleEndian;
use log::error;
use uefi::boot::{image_handle, AllocateType, MemoryType};
use uefi::mem::memory_map::{MemoryMap, MemoryMapOwned};
use uefi::{
    prelude::*,
    proto::media::{
        file::{self, File, FileAttribute},
        fs,
    },
};

const IDEAL_PHYSICAL_ADDRESS: u64 = 0x8000_0000;

/// Loads the kernel ELF and returns an option for a tuple where the first item is the physical
/// address of the kernel and the second item is the virtual address of the entry point.
pub fn read_kernel(mem_map: &MemoryMapOwned) -> Option<(u64, u64)> {
    let start_physical_address: u64 = find_physical_region(mem_map)?;

    if start_physical_address == 0 {
        error!("Error finding a suitable memory region to load the kernel");
        return None;
    }

    let img: Result<boot::ScopedProtocol<fs::SimpleFileSystem>, uefi::Error> =
        boot::get_image_file_system(image_handle());

    if img.is_err() {
        let err_msg: uefi::Error = img.err().unwrap();
        error!("Error reading kernel file: {err_msg}");
        return None;
    }

    let mut img: boot::ScopedProtocol<fs::SimpleFileSystem> = img.unwrap();
    let root_dir: Result<file::Directory, uefi::Error> = img.open_volume();
    if root_dir.is_err() {
        let err_msg: uefi::Error = root_dir.err().unwrap();
        error!("Error reading kernel file: {err_msg}");
        return None;
    }

    let mut root_dir: file::Directory = root_dir.unwrap();
    let fs: Result<file::FileHandle, uefi::Error> = root_dir.open(
        cstr16!("kernel.elf"),
        file::FileMode::Read,
        FileAttribute::empty(),
    );
    if fs.is_err() {
        let err_msg: uefi::Error = fs.err().unwrap();
        error!("Error reading kernel file: {err_msg}");
        return None;
    }

    let fs: file::FileHandle = fs.unwrap();
    let fs: Option<file::RegularFile> = fs.into_regular_file();
    if fs.is_none() {
        error!("Error reading kernel file: not a file");
        return None;
    }

    let mut buffer: [u8; 64 * 1024] = [0; 64 * 1024];
    let mut fs: file::RegularFile = fs.unwrap();

    let read: Result<usize, uefi::Error> = fs.read(&mut buffer);
    if read.is_err() {
        let err_msg: uefi::Error = read.err().unwrap();
        error!("Error reading kernel file: {err_msg}");
        return None;
    }

    let elf_data: Result<elf::ElfBytes<LittleEndian>, elf::ParseError> =
        elf::ElfBytes::<LittleEndian>::minimal_parse(&buffer[..read.unwrap()]);
    if elf_data.is_err() {
        let err_msg: elf::ParseError = elf_data.err().unwrap();
        error!("Error parsing kernel file: {err_msg}");
        return None;
    }

    let elf_data: elf::ElfBytes<'_, LittleEndian> = elf_data.unwrap();
    let prog_headers = elf_data.segments();
    if prog_headers.is_none() {
        error!("Error parsing kernel file: no segments found");
        return None;
    }

    let prog_headers = prog_headers.unwrap();

    for ph in prog_headers {
        if ph.p_type == elf::abi::PT_LOAD {
            // let dest: usize = (ph.p_paddr & ((1 << 32) - 1)) as usize;
            let dest: u64 = start_physical_address;
            let size: usize = ph.p_memsz as usize;
            let src_offset: usize = ph.p_offset as usize;
            let src_size: usize = ph.p_filesz as usize;

            let ptr = boot::allocate_pages(
                AllocateType::Address(dest),
                MemoryType::LOADER_DATA,
                // (size + 0xFFF) / 0x1000,
                (size / 0x1000) + 1,
            );

            if ptr.is_err() {
                let err_msg: uefi::Error = ptr.err().unwrap();
                error!("Error allocating kernel memory: {err_msg}");
                return None;
            }

            //copy the segment code at the physical address previously allocated
            unsafe {
                core::ptr::copy_nonoverlapping(
                    buffer[src_offset..].as_ptr(),
                    dest as *mut u8,
                    src_size,
                );

                //zero-out BSS after the code segment
                if size > src_size {
                    core::ptr::write_bytes(
                        (dest as usize + src_size) as *mut u8,
                        0,
                        size - src_size,
                    );
                }
            }
        }
    }

    return Some((start_physical_address, elf_data.ehdr.e_entry));
}

fn find_physical_region(mem_map: &MemoryMapOwned) -> Option<u64> {
    //check for the region between 0x8000_0000 and 0xffff_ffff, because that's the region where
    //the kernel should generally be loaded
    for entry in mem_map.entries() {
        if entry.ty != MemoryType::CONVENTIONAL
            || entry.ty != MemoryType::LOADER_DATA
            || entry.ty != MemoryType::LOADER_CODE
        {
            continue;
        }

        //if there is a region that starts at 0x8000_0000 (IDEAL_PHYSICAL_ADDRESS) and is at least
        //1 MiB in size, use 0x8000_0000
        if entry.phys_start <= IDEAL_PHYSICAL_ADDRESS
            && entry.phys_start + (entry.page_count * 0x1000) > 0x8100_0000
        {
            return Some(IDEAL_PHYSICAL_ADDRESS);
        }
    }

    //otherwise find any suitable region
    for entry in mem_map.entries() {
        if entry.ty != MemoryType::CONVENTIONAL {
            continue;
        }

        //needs to be at least 1 MiB
        if entry.page_count >= 0x10_00_00 / 0x1000 {
            return Some(entry.phys_start as u64);
        }
    }

    return None;
}
