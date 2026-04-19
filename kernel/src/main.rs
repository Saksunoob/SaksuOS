#![feature(abi_x86_interrupt)]
#![feature(negative_impls)]
#![no_std]
#![no_main]

#[macro_use]
mod display;
mod memory;
mod interrupts;
mod gdt;
mod sync;
mod keyboard;

use core::arch::asm;
#[cfg(not(test))]
use core::panic::PanicInfo;
use bootloader::boot_info::{BootInfo, Color};
use crate::interrupts::InterruptStackFrame;

#[unsafe(no_mangle)]
extern "C" fn _start(boot_info: *mut BootInfo) -> u64 {
    let boot_info = unsafe { boot_info.as_mut().unwrap() };
    display::init(&mut boot_info.framebuffer);
    println!("Hello, World!");
    memory::init(boot_info.memory_map);
    println!("Initializing gdt");
    gdt::flush();
    println!("Initializing idt");
    interrupts::init_idt();
    println!("set handler");
    interrupts::set_interrupt_handler(0, divide_by_zero);
    unsafe {
        keyboard::init();
        asm!("sti")
    };
    panic!("Successful initialization!");
}

extern "x86-interrupt" fn divide_by_zero(_: InterruptStackFrame) {
    println!("Divide by zero interrupt");
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
    loop { unsafe {
        asm!("hlt")
    } }
}
