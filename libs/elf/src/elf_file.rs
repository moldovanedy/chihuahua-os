pub struct ElfFile {
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

impl ElfFile {
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
    }e_phentsize

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
