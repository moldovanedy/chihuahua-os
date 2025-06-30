///Size = 40 bytes.
#[repr(C)]
pub struct MemoryMapEntry {
    mem_type: MemoryType,
    ///One of the `memory_attributes::*` constants.
    attributes: u64,
    physical_addr: u64,
    virtual_addr: u64,
    num_pages: u64,
}

impl MemoryMapEntry {
    pub fn mem_type(&self) -> MemoryType {
        self.mem_type
    }

    pub fn attributes(&self) -> u64 {
        self.attributes
    }

    pub fn physical_addr(&self) -> u64 {
        self.physical_addr
    }

    pub fn virtual_addr(&self) -> u64 {
        self.virtual_addr
    }

    pub fn num_pages(&self) -> u64 {
        self.num_pages
    }

    pub fn new(
        mem_type: MemoryType,
        attributes: u64,
        physical_addr: u64,
        virtual_addr: u64,
        num_pages: u64,
    ) -> Self {
        MemoryMapEntry {
            mem_type,
            attributes,
            physical_addr,
            virtual_addr,
            num_pages,
        }
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MemoryType {
    /// Not usable.
    Reserved = 0,
    /// The code portions of a loaded UEFI application.
    LoaderCode = 1,
    /// The data portions of a loaded UEFI application,
    /// as well as any memory allocated by it.
    LoaderData = 2,
    /// Code of the boot drivers.
    ///
    /// Can be reused after OS is loaded.
    BootServicesCode = 3,
    /// Memory used to store boot drivers' data.
    ///
    /// Can be reused after OS is loaded.
    BootServicesData = 4,
    /// Runtime drivers' code.
    RuntimeServicesCode = 5,
    /// Runtime services' code.
    RuntimeServicesData = 6,
    /// Free usable memory.
    Conventional = 7,
    /// Memory in which errors have been detected.
    Unusable = 8,
    /// Memory that holds ACPI tables.
    /// Can be reclaimed after they are parsed.
    AcpiReclaim = 9,
    /// Firmware-reserved addresses.
    AcpiNonVolatile = 10,
    /// A region used for memory-mapped I/O.
    Mmio = 11,
    /// Address space used for memory-mapped port I/O.
    MmioPortSpace = 12,
    /// Address space which is part of the processor.
    PalCode = 13,
    /// Memory region which is usable and is also non-volatile.
    PersistentMemory = 14,
    /// Memory that must be accepted by the boot target before it can be used.
    Unaccepted = 15,
    /// End of the defined memory types. Higher values are possible, though.
    Max = 16,
}

impl From<u32> for MemoryType {
    fn from(value: u32) -> Self {
        match value { 
            0 => {MemoryType::Reserved}
            1 => {MemoryType::LoaderCode}
            2 => {MemoryType::LoaderData}
            3 => {MemoryType::BootServicesCode}
            4 => {MemoryType::BootServicesData}
            5 => {MemoryType::RuntimeServicesCode}
            6 => {MemoryType::RuntimeServicesData}
            7 => {MemoryType::Conventional}
            8 => {MemoryType::Unusable}
            9 => {MemoryType::AcpiReclaim}
            10 => {MemoryType::AcpiNonVolatile}
            11 => {MemoryType::Mmio}
            12 => {MemoryType::MmioPortSpace}
            13 => {MemoryType::PalCode}
            14 => {MemoryType::PersistentMemory}
            15 => {MemoryType::Unaccepted}
            _ => {MemoryType::Unusable} 
        }
    }
}

pub mod memory_attributes {
    /// Supports marking as uncacheable.
    pub const UNCACHEABLE: u64 = 0x1;
    /// Supports write-combining.
    pub const WRITE_COMBINE: u64 = 0x2;
    /// Supports write-through.
    pub const WRITE_THROUGH: u64 = 0x4;
    /// Support write-back.
    pub const WRITE_BACK: u64 = 0x8;
    /// Supports marking as uncacheable, exported and
    /// supports the "fetch and add" semaphore mechanism.
    pub const UNCACHABLE_EXPORTED: u64 = 0x10;
    /// Supports write-protection.
    pub const WRITE_PROTECT: u64 = 0x1000;
    /// Supports read-protection.
    pub const READ_PROTECT: u64 = 0x2000;
    /// Supports disabling code execution.
    pub const EXECUTE_PROTECT: u64 = 0x4000;
    /// Persistent memory.
    pub const NON_VOLATILE: u64 = 0x8000;
    /// This memory region is more reliable than other memory.
    pub const MORE_RELIABLE: u64 = 0x10000;
    /// This memory range can be set as read-only.
    pub const READ_ONLY: u64 = 0x20000;
    /// This memory is earmarked for specific purposes such as for specific
    /// device drivers or applications. This serves as a hint to the OS to
    /// avoid this memory for core OS data or code that cannot be relocated.
    pub const SPECIAL_PURPOSE: u64 = 0x4_0000;
    /// This memory region is capable of being protected with the CPU's memory
    /// cryptography capabilities.
    pub const CPU_CRYPTO: u64 = 0x8_0000;
    /// This memory must be mapped by the OS when a runtime service is called.
    pub const RUNTIME: u64 = 0x8000_0000_0000_0000;
    /// This memory region is described with additional ISA-specific memory
    /// attributes as specified in `MemoryAttribute::ISA_MASK`.
    pub const ISA_VALID: u64 = 0x4000_0000_0000_0000;
    /// These bits are reserved for describing optional ISA-specific cache-
    /// ability attributes that are not covered by the standard UEFI Memory
    /// Attribute cacheability bits such as `UNCACHEABLE`, `WRITE_COMBINE`,
    /// `WRITE_THROUGH`, `WRITE_BACK`, and `UNCACHEABLE_EXPORTED`.
    ///
    /// See Section 2.3 "Calling Conventions" in the UEFI Specification
    /// for further information on each ISA that takes advantage of this.
    pub const ISA_MASK: u64 = 0x0FFF_F000_0000_0000;
}
