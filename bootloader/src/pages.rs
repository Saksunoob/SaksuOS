use uefi_raw::{PhysicalAddress, Status};
use uefi_raw::table::boot::{AllocateType, BootServices, MemoryType};

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