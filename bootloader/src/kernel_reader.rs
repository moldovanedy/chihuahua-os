use elf::endian::LittleEndian;
use log::error;
use uefi::boot::{AllocateType, image_handle};
use uefi::{
    prelude::*,
    proto::media::{
        file::{self, File, FileAttribute},
        fs,
    },
};

/// Loads the kernel ELF and returns the physical address of the entry point or 0 if something went wrong.
pub fn read_kernel() -> u64 {
    let img: Result<boot::ScopedProtocol<fs::SimpleFileSystem>, uefi::Error> =
        boot::get_image_file_system(image_handle());

    if img.is_err() {
        let err_msg: uefi::Error = img.err().unwrap();
        error!("Error reading kernel file: {err_msg}");
        return 0;
    }

    let mut img: boot::ScopedProtocol<fs::SimpleFileSystem> = img.unwrap();
    let root_dir: Result<file::Directory, uefi::Error> = img.open_volume();
    if root_dir.is_err() {
        let err_msg: uefi::Error = root_dir.err().unwrap();
        error!("Error reading kernel file: {err_msg}");
        return 0;
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
        return 0;
    }

    let fs: file::FileHandle = fs.unwrap();
    let fs: Option<file::RegularFile> = fs.into_regular_file();
    if fs.is_none() {
        error!("Error reading kernel file: not a file");
        return 0;
    }

    let mut buffer: [u8; 64 * 1024] = [0; 64 * 1024];
    let mut fs: file::RegularFile = fs.unwrap();

    let read: Result<usize, uefi::Error> = fs.read(&mut buffer);
    if read.is_err() {
        let err_msg: uefi::Error = read.err().unwrap();
        error!("Error reading kernel file: {err_msg}");
        return 0;
    }

    let elf_data = elf::ElfBytes::<LittleEndian>::minimal_parse(&buffer[..read.unwrap()]);
    if elf_data.is_err() {
        let err_msg: elf::ParseError = elf_data.err().unwrap();
        error!("Error parsing kernel file: {err_msg}");
        return 0;
    }

    let elf_data: elf::ElfBytes<'_, LittleEndian> = elf_data.unwrap();
    let prog_headers = elf_data.segments();
    if prog_headers.is_none() {
        error!("Error parsing kernel file: no segments found");
        return 0;
    }

    let prog_headers = prog_headers.unwrap();

    for ph in prog_headers {
        if ph.p_type == elf::abi::PT_LOAD {
            let dest: usize = (ph.p_paddr & ((1 << 32) - 1)) as usize;
            let size: usize = ph.p_memsz as usize;
            let src_offset: usize = ph.p_offset as usize;
            let src_size: usize = ph.p_filesz as usize;

            let ptr = boot::allocate_pages(
                AllocateType::Address(dest as u64),
                uefi::boot::MemoryType::LOADER_DATA,
                // (size + 0xFFF) / 0x1000,
                (size / 0x1000) + 1,
            );

            if ptr.is_err() {
                let err_msg: uefi::Error = ptr.err().unwrap();
                error!("Error allocating kernel memory: {err_msg}");
                return 0;
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
                    core::ptr::write_bytes((dest + src_size) as *mut u8, 0, size - src_size);
                }
            }
        }
    }

    return elf_data.ehdr.e_entry;
}
