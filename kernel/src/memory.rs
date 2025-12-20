use bootloader::memory::{EfiMemoryDescriptor, MemoryType};

pub const PAGE_SIZE: usize = 4096;

struct MemoryRegion {
    start: usize,
    end: usize,
}

impl MemoryRegion {
    pub fn new(start: usize, end: usize) -> MemoryRegion {
        MemoryRegion { start, end }
    }

    pub fn size(&self) -> usize { self.end - self.start }
}

type PageRegion = MemoryRegion;

static mut FREE_PAGES_START: *mut PageRegion = 0 as *mut PageRegion;
static mut FREE_PAGE_REGIONS_COUNT: usize = 0;

pub fn init(map: &'static [EfiMemoryDescriptor]) {
    for descriptor in map {
        if descriptor.m_type != MemoryType::ConventionalMemory || descriptor.p_addr == 0 {
            continue;
        }

        let region_end = descriptor.p_addr+descriptor.num_pages*PAGE_SIZE;
        unsafe { if FREE_PAGES_START.is_null() {
            FREE_PAGES_START = descriptor.p_addr as *mut PageRegion;
            FREE_PAGE_REGIONS_COUNT += 1;
            FREE_PAGES_START.write(PageRegion::new(descriptor.p_addr+PAGE_SIZE, region_end));
            continue;
        } }
        let last_region = unsafe { FREE_PAGES_START.add(FREE_PAGE_REGIONS_COUNT-1).as_mut().unwrap() };
        if last_region.end == descriptor.p_addr {
            last_region.end = descriptor.p_addr+descriptor.num_pages*PAGE_SIZE;
        } else {
            unsafe {
                FREE_PAGE_REGIONS_COUNT += 1;
                FREE_PAGES_START.add(FREE_PAGE_REGIONS_COUNT-1).write(PageRegion::new(descriptor.p_addr, region_end))
            }
        }
    }
}

pub fn allocate_page() -> *mut u8 {
    let last_region = unsafe { FREE_PAGES_START.add(FREE_PAGE_REGIONS_COUNT-1).as_mut().unwrap() };
    last_region.end -= PAGE_SIZE;
    let ptr = last_region.end as *mut u8;

    if last_region.end == last_region.start {
        unsafe { FREE_PAGE_REGIONS_COUNT -= 1 }
    }
    ptr
}