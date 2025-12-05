#![no_std]
#![no_main]

#[cfg(not(test))]
use core::panic::PanicInfo;
use bootloader::boot_info::{BootInfo, PixelFormat, RGBColor, Color};

#[unsafe(no_mangle)]
extern "C" fn _start(boot_info: *mut BootInfo) -> u64 {
    let boot_info = unsafe { boot_info.as_mut().unwrap() };
    let color = RGBColor::new(255, 0, 0);

    let color = match boot_info.framebuffer.pixel_format {
        PixelFormat::RGB => color.get_rgba(),
        PixelFormat::BGR => color.get_bgra(),
        _ => color.get_rgba(),
    };
    boot_info.framebuffer.buffer.fill(color);
    0
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
