#[repr(C)]
pub struct ElfHeader {
    e_ident_mag: u32,
    e_ident_class: u8,
    e_ident_data: u8,
    e_ident_version: u8,
    e_ident_osabi: u8,
    e_ident_abiversion: u8,
    e_ident_pad: [u8; 7],

    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

impl ElfHeader {
    pub fn e_ident_class(&self) -> u8 {
        self.e_ident_class
    }

    pub fn e_ident_data(&self) -> u8 {
        self.e_ident_data
    }

    pub fn e_ident_version(&self) -> u8 {
        self.e_ident_version
    }

    pub fn e_ident_osabi(&self) -> u8 {
        self.e_ident_osabi
    }

    pub fn e_ident_abiversion(&self) -> u8 {
        self.e_ident_abiversion
    }

    pub fn e_type(&self) -> u16 {
        self.e_type
    }

    pub fn e_machine(&self) -> u16 {
        self.e_machine
    }

    pub fn e_version(&self) -> u32 {
        self.e_version
    }

    pub fn e_entry(&self) -> u64 {
        self.e_entry
    }

    pub fn e_phoff(&self) -> u64 {
        self.e_phoff
    }

    pub fn e_shoff(&self) -> u64 {
        self.e_shoff
    }

    pub fn e_flags(&self) -> u32 {
        self.e_flags
    }

    pub fn e_ehsize(&self) -> u16 {
        self.e_ehsize
    }

    pub fn e_phentsize(&self) -> u16 {
        self.e_phentsize
    }

    pub fn e_phnum(&self) -> u16 {
        self.e_phnum
    }

    pub fn e_shentsize(&self) -> u16 {
        self.e_shentsize
    }

    pub fn e_shnum(&self) -> u16 {
        self.e_shnum
    }

    pub fn e_shstrndx(&self) -> u16 {
        self.e_shstrndx
    }
}

pub const ELF_MAG: u32 = 0x7f454c46;

/// The only OK value for [e_ident_osabi](ElfHeader::e_ident_osabi).
pub const OS_ABI_SYS_V: u8 = 0;
/// The only supported value for [e_machine](ElfHeader::e_machine).
pub const MACHINE_X86_64: u16 = 0x3E;
/// 64-bit file.
pub const CLASS_64_BIT: u8 = 2;
/// Little-endian file.
pub const DATA_LITTLE_ENDIAN: u8 = 1;

#[allow(dead_code)]
pub mod et_types {
    pub const ET_NONE: u16 = 0;
    /// Relocatable file
    pub const ET_REL: u16 = 1;
    /// Executable file
    pub const ET_EXEC: u16 = 2;
    /// Dynamic library file
    pub const ET_DYN: u16 = 3;
    /// Core file
    pub const ET_CORE: u16 = 4;
    pub const ET_LOOS: u16 = 0xfe00;
    pub const ET_HIOS: u16 = 0xfeff;
    pub const ET_LOPROC: u16 = 0xff00;
    pub const ET_HIPROC: u16 = 0xffff;
}

impl ElfHeader {
    #[unsafe(no_mangle)]
    pub extern "C" fn parse(data: &[u8]) -> Option<Self> {
        unsafe {
            let ptr = data.as_ptr() as *const Self;
            let header = ptr.read_unaligned();

            if header.e_ident_mag != ELF_MAG {
                return None;
            }
            if header.e_ident_class != CLASS_64_BIT {
                return None;
            }
            if header.e_ident_data != DATA_LITTLE_ENDIAN {
                return None;
            }
            if header.e_machine != MACHINE_X86_64 {
                return None;
            }

            return Some(header);
        }
    }
}
