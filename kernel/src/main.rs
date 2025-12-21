#![no_std]
#![no_main]

mod display;
mod memory;

#[cfg(not(test))]
use core::panic::PanicInfo;
use bootloader::boot_info::{BootInfo, Color};
use crate::memory::{allocate_page, PAGE_SIZE};

#[unsafe(no_mangle)]
extern "C" fn _start(boot_info: *mut BootInfo) -> u64 {
    let boot_info = unsafe { boot_info.as_mut().unwrap() };
    display::init(&mut boot_info.framebuffer);
    println!("Hello, World!");
    memory::init(boot_info.memory_map);
    let page = allocate_page();
    for i in 0..PAGE_SIZE {
        unsafe { page.add(i).write(1) }
    }
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    match info.message().as_str() {
        Some(msg) => println!("Panic: {}", msg),
        None => ()
    }
    if let Some(pos) = info.location() {
        println!("at {}:{}:{}", pos.file(), pos.line(), pos.column());
    }
    loop {}
}
