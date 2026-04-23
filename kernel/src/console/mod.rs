use bootloader::boot_info::RGBColor;
use crate::display;

pub fn _start() -> ! {
    display::clear(RGBColor::new(255,0,0));

    print!("\x0C\r");
    print!("> ");

    loop {}
}