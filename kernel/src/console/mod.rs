use core::arch::asm;
use bootloader::boot_info::RGBColor;
use crate::{display, keyboard};

pub fn _start() -> ! {
    display::clear(RGBColor::new(255,0,0));

    print!("\x0C\r");
    print!("> ");

    loop {
        match keyboard::poll_event() {
            Some(event) => { println!("{:x}", event); },
            None => unsafe { asm!("hlt") }
        }
    }
}