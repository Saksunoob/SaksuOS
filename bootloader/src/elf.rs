use core::ptr::slice_from_raw_parts;

#[repr(u8)]
#[derive(PartialEq, Clone, Copy)]
pub enum Bits {
    Bit32 = 1,
    Bit64 = 2
}

#[repr(u8)]
#[derive(PartialEq, Clone, Copy)]
pub enum Endian {
    Little = 1,
    Big = 2
}

#[repr(u16)]
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum FileType {
    Unknown,
    Relocatable,
    Executable,
    SharedObject,
    CoreFile
}

#[repr(C)]
#[derive(PartialEq, Clone, Copy)]
pub struct Ident {
    pub magic: [u8; 4],
    pub bits: Bits,
    pub endian: Endian,
    pub version: u8,
    pub target_os: u8,
    pub abi_version: u8,
    pub padding: [u8; 7],
}

pub struct Elf64<'a> {
    pub ident: &'a Ident,
    pub binary: *const u8,
    pub f_type: FileType,
    pub machine: u16,
    pub version: u32,
    pub entry: u64,
    pub ph_offset: u64,
    pub sh_offset: u64,
    pub flags: u32,
    pub hdr_size: u16,
    pub ph_entry_size: u16,
    pub ph_entry_count: u16,
    pub sh_entry_size: u16,
    pub sh_entry_count: u16,
    pub sh_name_index: u16,
}

#[derive(Debug)]
pub enum ElfParseError {
    Magic([u8; 4]),
    Bits32,
    Version(u32),
    TargetOS(u8),
    FileType(FileType),
    Machine(u16),
    PHOffset(u64),
    PHSize(u16),
    SHSize(u16),
    HeaderSize(u16)

}

#[repr(u32)]
pub enum SegmentType {
    Null = 0x00,
    Load = 0x01,
    Dynamic = 0x02,
    Interpreter = 0x03,
    Note = 0x04,
    Shlib = 0x05,
    ProgramHeader = 0x06,
    ThreadLocals = 0x07
}

#[repr(C)]
pub struct ProgramHeader {
    pub segment_type: SegmentType,
    pub flags: u32,
    pub offset: u64,
    pub vaddr: u64,
    pub paddr: u64,
    pub file_size: u64,
    pub mem_size: u64,
    pub align: u64,
}

#[repr(u32)]
pub enum SectionType {
    Null = 0,
    Progbits = 1,
    Symtab = 2,
    Strtab = 3,
    Rela = 4,
    Hash = 5,
    Dynamic = 6,
    Note = 7,
    Nobits = 8,
    Rel = 9,
    Shlib = 10,
    Dynsym = 11,
    InitArray = 14,
    FiniArray = 15,
    PreinitArray = 16,
    Group = 17,
    SymtabShndx = 18,
}

#[repr(C, packed)]
pub struct SectionHeader {
    pub name: u32,
    pub section_type: SectionType,
    pub flags: u64,
    pub vaddr: u64,
    pub offset: u64,
    pub size: u64,
    pub link_index: u32,
    pub info: u32,
    pub addr_align: u64,
    pub entry_size: u64,
}

#[repr(C)]
pub struct RELEntry {
    pub offset: u64,
    pub relocation_type: RelocationType,
    pub symbol_index: u32,
}

#[repr(C)]
pub struct RELAEntry {
    pub offset: u64,
    pub relocation_type: RelocationType,
    pub symbol_index: u32,
    pub addend: u64
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum RelocationType {
    NONE = 0,
    B64 = 1,
    PC32 = 2,
    GOT32 = 3,
    PLT32 = 4,
    COPY = 5,
    GlobDat = 6,
    JumpSlot = 7,
    RELATIVE = 8,
    GOTPCREL = 9,
    B32 = 10,
    B32S = 11,
    B16 = 12,
    PC16 = 13,
    B8 = 14,
    PC8 = 15,
    PC64 = 24,
    GOTOFF64 = 25,
    GOTPC32 = 26,
    SIZE32 = 32,
    SIZE64 = 33,
}

#[repr(u64)]
#[derive(Eq, PartialEq)]
pub enum DynamicEntryType {
    Null = 0,
    Needed = 1,
    PLTRelSize = 2,
    PLTGOT = 3,
    Hash = 4,
    StringTable = 5,
    SymbolTable = 6,
    Rela = 7,
    RelaSize = 8,
    RelaEntry = 9,
    StringSize = 10,
    SymbolEntry = 11,
    Init = 12,
    Fini = 13,
    SONAME = 14,
    RPATH = 15,
    Symbolic = 16,
    Rel = 17,
    RelSize = 18,
    RelEntry = 19,
    PLTRel = 20,
    Debug = 21,
    TextRel = 22,
    JumpRel = 23,
    BindNow = 24,
    InitArray = 25,
    FiniArray = 26,
    InitArraySize = 27,
    FiniArraySize = 28,
    RunPath = 29,
    Flags = 30,
    Encoding = 31,
    PreInitArray = 32,
    PreInitArraySize = 33,
}

pub struct DynamicSectionEntry{
    pub d_tag: DynamicEntryType,
    pub d_val: u64,
}

impl<'a> Elf64<'a> {
    pub fn new(binary: *const u8) -> Result<Self, ElfParseError> {
        let ident = unsafe {(binary as *const Ident).as_ref().unwrap()};

        if ident.magic != [0x7F, 0x45, 0x4c, 0x46] {return Err(ElfParseError::Magic(ident.magic))}
        if ident.bits != Bits::Bit64 {return Err(ElfParseError::Bits32)}
        if ident.version != 1 {return Err(ElfParseError::Version(ident.version as u32))}
        if ident.target_os != 0 {return Err(ElfParseError::TargetOS(ident.target_os))}
        let flip_bytes = ident.endian == Endian::Big;
        let elf = unsafe {
            let f_type = Self::read_value(binary, 0x10, flip_bytes);
            let machine = Self::read_value(binary, 0x12, flip_bytes);
            let version = Self::read_value(binary, 0x14, flip_bytes);
            let entry = Self::read_value(binary, 0x18, flip_bytes);
            let ph_offset = Self::read_value(binary, 0x20, flip_bytes);
            let sh_offset = Self::read_value(binary, 0x28, flip_bytes);
            let flags = Self::read_value(binary, 0x30, flip_bytes);
            let hdr_size = Self::read_value(binary, 0x34, flip_bytes);
            let ph_entry_size = Self::read_value(binary, 0x36, flip_bytes);
            let ph_entry_count = Self::read_value(binary, 0x38, flip_bytes);
            let sh_entry_size = Self::read_value(binary, 0x3A, flip_bytes);
            let sh_entry_count = Self::read_value(binary, 0x3C, flip_bytes);
            let sh_name_index = Self::read_value(binary, 0x3E, flip_bytes);
            Elf64 {
                ident,
                binary,
                f_type,
                machine,
                version,
                entry,
                ph_offset,
                sh_offset,
                flags,
                hdr_size,
                ph_entry_size,
                ph_entry_count,
                sh_entry_size,
                sh_entry_count,
                sh_name_index,
            }
        };

        if elf.f_type != FileType::SharedObject {return Err(ElfParseError::FileType(elf.f_type))}
        if elf.machine != 0x3E {return Err(ElfParseError::Machine(elf.machine))}
        if elf.version != 1 {return Err(ElfParseError::Version(elf.version))}
        if elf.ph_offset != 0x40 {return Err(ElfParseError::PHOffset(elf.ph_offset))}
        if elf.hdr_size != 0x40 {return Err(ElfParseError::HeaderSize(elf.hdr_size))}
        if elf.ph_entry_size != 0x38 {return Err(ElfParseError::PHSize(elf.ph_entry_size))}
        if elf.sh_entry_size != 0x40 {return Err(ElfParseError::SHSize(elf.sh_entry_size))}
        Ok(elf)
    }

    pub fn program_headers(&self) -> &'static [ProgramHeader] {
        let ptr = unsafe {self.binary.byte_add(self.ph_offset as usize)} as *const ProgramHeader;
        let size = self.ph_entry_count as usize;
        match unsafe { slice_from_raw_parts(ptr, size).as_ref() } {
            Some(slice) => slice,
            None => panic!("program header pointer out of range")
        }
    }

    pub fn section_headers(&self) -> &'static [SectionHeader] {
        let ptr = unsafe {self.binary.byte_add(self.sh_offset as usize)} as *const SectionHeader;
        let size = self.sh_entry_count as usize;
        match unsafe { slice_from_raw_parts(ptr, size).as_ref() } {
            Some(slice) => slice,
            None => panic!("section header pointer out of range")
        }
    }

    unsafe fn read_value<T: Sized>(binary: *const u8, offset: usize, flip_bytes: bool) -> T where [u8; size_of::<T>()]: Sized {
        let buffer_ptr = unsafe {binary.byte_add(offset)} as *const [u8; size_of::<T>()];
        if flip_bytes {
            let mut buffer = [0; size_of::<T>()];
            for i in 0..size_of::<T>() {
                buffer[i] = unsafe{*(buffer_ptr as *const u8).byte_add(size_of::<T>()-i-1)};
            }
            unsafe {core::mem::transmute_copy(&buffer)}
        } else {
            unsafe {core::mem::transmute_copy(buffer_ptr.as_ref().unwrap())}
        }
    }
}