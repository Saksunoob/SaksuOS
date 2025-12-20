#[derive(PartialEq, Eq)]
#[repr(u32)]
pub enum MemoryType {
    ReservedMemoryType = 0,
    LoaderCode = 1,
    LoaderData = 2,
    BootServicesCode = 3,
    BootServicesData = 4,
    RuntimeServicesCode = 5,
    RuntimeServicesData = 6,
    ConventionalMemory = 7,
    UnusableMemory = 8,
    ACPIReclaimMemory = 9,
    ACPIMemoryNVS = 10,
    MemoryMappedIO = 11,
    MemoryMappedIOPortSpace = 12,
    PalCode = 13,
    PersistentMemory = 14,
    UnacceptedMemoryType = 15,
    MaxMemoryType = 16
}

#[repr(C)]
pub struct EfiMemoryDescriptor {
    pub m_type: MemoryType,
    pub p_addr: usize,
    pub v_addr: usize,
    pub num_pages: usize,
    pub attribute: u64
}