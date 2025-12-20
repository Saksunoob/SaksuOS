#![no_std]
#![no_main]

mod display;
mod memory;
mod serial;

#[cfg(not(test))]
use core::panic::PanicInfo;
use bootloader::boot_info::{BootInfo, RGBColor, Color};
use crate::memory::{allocate_page, PAGE_SIZE};

#[unsafe(no_mangle)]
extern "C" fn _start(boot_info: *mut BootInfo) -> u64 {
    let boot_info = unsafe { boot_info.as_mut().unwrap() };
    display::init(&mut boot_info.framebuffer);
    display::debug_dot(RGBColor::new(255, 0, 0));
    memory::init(boot_info.memory_map);
    display::debug_dot(RGBColor::new(0, 255, 0));
    let page = allocate_page();
    for i in 0..PAGE_SIZE {
        unsafe { page.add(i).write(1) }
    }
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
