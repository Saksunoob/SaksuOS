use core::ptr::null_mut;
use uefi_raw::{PhysicalAddress, Status, VirtualAddress};
use uefi_raw::table::boot::{AllocateType, BootServices, MemoryType};
use x86_64::PhysAddr;
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::structures::paging::PhysFrame;

pub fn allocate_pages(bs: *mut BootServices, count: usize) -> *mut u8 {
    let mut addr = PhysicalAddress::default();
    let status = unsafe {
        ((*bs).allocate_pages)(AllocateType::ANY_PAGES, MemoryType::LOADER_DATA, count, &mut addr)
    };
    match status {
        Status::SUCCESS => addr as *mut u8,
        Status::NOT_FOUND | Status::OUT_OF_RESOURCES => panic!("Out of memory"),
        _ => panic!("Unexpected status: {:?}", status),
    }
}

static mut L4_TABLE: *mut PageTable = null_mut();

pub fn init_paging(bs: *mut BootServices) {
    let l4 = allocate_pages(bs, 1) as *mut PageTable;
    let current_l4 = Cr3::read().0.start_address().as_u64() as *mut PageTable;

    unsafe {
        core::ptr::copy_nonoverlapping(current_l4, l4, 1);

        (*l4).set_entry(511, PageTableEntry::new(l4 as usize)
            .with(PageTableBits::Write)
            .with(PageTableBits::UserAccessible));
        L4_TABLE = l4
    }
}

pub fn switch_l4() {
    unsafe {
        Cr3::write(PhysFrame::containing_address(PhysAddr::new(L4_TABLE as u64)), Cr3Flags::empty());
    }
}

pub fn map_page(bs: *mut BootServices, physical_address: PhysicalAddress, virtual_address: VirtualAddress) {
    unsafe {
        // 1. Extract indices
        let l4_index = (virtual_address >> 39) & 0x1FF;
        let l3_index = (virtual_address >> 30) & 0x1FF;
        let l2_index = (virtual_address >> 21) & 0x1FF;
        let l1_index = (virtual_address >> 12) & 0x1FF;

        // Root table (already allocated)
        let l4 = L4_TABLE;

        // ----- Helper closure: ensure child table exists -----
        unsafe fn get_or_alloc(
            parent: *mut PageTable,
            index: usize,
            bs: *mut BootServices,
        ) -> *mut PageTable {
            let entry = unsafe { (*parent).entry(index) };

            if entry.is_present() {
                entry.address() as *mut PageTable
            } else {
                // allocate a fresh page for this table
                let new_table =
                    allocate_pages(bs, 1) as *mut PageTable;

                unsafe { core::ptr::write_bytes(new_table, 0, 1) };
                // install entry
                let phys = new_table as usize;
                unsafe { (*parent).set_entry(
                    index,
                    PageTableEntry::new(phys)
                        .with(PageTableBits::Write)
                        .with(PageTableBits::UserAccessible),
                )};

                new_table
            }
        }

        // 2. Walk / allocate levels
        let l3 = get_or_alloc(l4, l4_index as usize, bs);
        let l2 = get_or_alloc(l3, l3_index as usize, bs);
        let l1 = get_or_alloc(l2, l2_index as usize, bs);
        
        // 3. Install the leaf PTE (4-KiB page)
        (*l1).set_entry(
            l1_index as usize,
            PageTableEntry::new((physical_address & !0xFFF) as usize)
                .with(PageTableBits::Present)
                .with(PageTableBits::Write)
                .with(PageTableBits::UserAccessible),
        );
    }
}

#[repr(u64)]
#[allow(dead_code)]
enum PageTableBits {
    Present = 1,
    Write = 2,
    UserAccessible = 4,
    WriteThrough = 8,
    CacheDisable = 16,
    Accessed = 32,
    Dirty = 64,
    AttributeTable = 128,
    Global = 256,
    Address =       0x000ffffffffff000,
    ProtectionKey = 0x7800000000000000,
    ExecuteDisable = 1<<63,
}

#[repr(C)]
struct PageTable {
    entries: [PageTableEntry; 512],
}

impl PageTable {
    pub fn set_entry(&mut self, index: usize, entry: PageTableEntry) {
        self.entries[index] = entry;
    }

    pub fn entry(&self, index: usize) -> &PageTableEntry { &self.entries[index] }
}

struct PageTableEntry {
    entry: u64
}
impl PageTableEntry {
    pub fn new(address: usize) -> Self {
        PageTableEntry { entry: address as u64 & PageTableBits::Address as u64 | PageTableBits::Present as u64 }
    }

    pub fn with(&self, attr: PageTableBits) -> Self {
        PageTableEntry { entry: self.entry | attr as u64 }
    }

    pub fn is_present(&self) -> bool { self.entry & PageTableBits::Present as u64 != 0 }
    pub fn address(&self) -> usize { (self.entry & PageTableBits::Address as u64) as usize }
}