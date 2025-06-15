use log::warn;
use uefi::{
    prelude::*,
    proto::media::{
        file::{self, File, FileAttribute},
        fs,
    },
};

/// Will read the dog.cfg config file to establish boot preferences. Returns an Option<SystemConfig>, it also
/// automatically writes a warning to the console if something went wrong.
pub fn read_config() -> Option<SystemConfig> {
    let img = boot::get_image_file_system(boot::image_handle());
    if img.is_err() {
        let err_msg: uefi::Error = img.err().unwrap();
        warn!("Error reading system configuration: {err_msg}");
        return None;
    }

    let mut img: boot::ScopedProtocol<fs::SimpleFileSystem> = img.unwrap();
    let root_dir: Result<file::Directory, uefi::Error> = img.open_volume();
    if root_dir.is_err() {
        let err_msg: uefi::Error = root_dir.err().unwrap();
        warn!("Error reading system configuration: {err_msg}");
        return None;
    }

    let mut root_dir: file::Directory = root_dir.unwrap();
    let fs: Result<file::FileHandle, uefi::Error> = root_dir.open(
        cstr16!("dog.cfg"),
        file::FileMode::Read,
        FileAttribute::empty(),
    );
    if fs.is_err() {
        let err_msg: uefi::Error = fs.err().unwrap();
        warn!("Error reading system configuration: {err_msg}");
        return None;
    }

    let fs: file::FileHandle = fs.unwrap();
    let fs: Option<file::RegularFile> = fs.into_regular_file();
    if fs.is_none() {
        warn!("Error reading system configuration: not a file");
        return None;
    }

    let fs: file::RegularFile = fs.unwrap();
    let config: Option<SystemConfig> = interpret_data(fs);

    if config.is_none() {
        return None;
    } else {
        return Some(config.unwrap());
    }
}

fn interpret_data(mut fs: file::RegularFile) -> Option<SystemConfig> {
    let mut buffer: [u8; 1024] = [0; 1024];
    let mut bytes_read_now: usize = 1;
    let mut total_bytes_read: usize = 0;

    while bytes_read_now > 0 {
        let read: Result<usize, uefi::Error> = fs.read(&mut buffer);
        if read.is_err() {
            let err_msg: uefi::Error = read.err().unwrap();
            warn!("Error reading system configuration: {err_msg}");
            return None;
        }

        bytes_read_now = read.unwrap();
        total_bytes_read += bytes_read_now;
        let result: Result<(), uefi::Error> = fs.set_position(total_bytes_read as u64);
        if result.is_err() {
            let err_msg: uefi::Error = result.err().unwrap();
            warn!("Error reading system configuration: {err_msg}");
            return None;
        }
    }

    return Some(SystemConfig::default());
}

/// The system configuration, the kernel preferences for boot. Respect if possible.
#[repr(C)]
pub struct SystemConfig {
    pub preferred_width: u32,
    pub preferred_height: u32,
}

impl SystemConfig {
    pub fn preferred_width(&self) -> u32 {
        self.preferred_width
    }

    pub fn preferred_height(&self) -> u32 {
        self.preferred_height
    }

    pub fn new(width: u32, height: u32) -> Self {
        SystemConfig {
            preferred_width: width,
            preferred_height: height,
        }
    }

    /// Returns a default SystemConfig with a width of 1920 and a height of 1080.
    pub fn default() -> Self {
        SystemConfig {
            preferred_width: 1920,
            preferred_height: 1080,
        }
    }
}
