#![no_std]
#![no_main]

#[cfg(not(test))]
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
fn _start() -> u64 {
    32
}

#[unsafe(no_mangle)]
pub static TEST_VALUE: u64 = 0x12345678;

#[unsafe(no_mangle)]
pub extern "C" fn test_function() -> u64 {
    TEST_VALUE
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
