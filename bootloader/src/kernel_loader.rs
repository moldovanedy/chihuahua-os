use log::error;
use uefi::boot::image_handle;
use uefi::{
    prelude::*,
    proto::media::{
        file::{self, File, FileAttribute},
        fs,
    },
};

pub fn load_kernel() -> u64 {
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

    let fs: file::RegularFile = fs.unwrap();

    return 0;
}
