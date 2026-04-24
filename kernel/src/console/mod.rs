pub mod scancode;

use core::arch::asm;
use bootloader::boot_info::RGBColor;
use crate::{display, keyboard};
use crate::console::scancode::{get_finnish_char, KeyCode};
use crate::sync::mutex::Mutex;

struct Modifiers {
    l_shift: bool,
    r_shift: bool,
    l_ctrl: bool,
    r_ctrl: bool,
    l_alt: bool,
    r_alt: bool,
}

static MODIFIERS: Mutex<Modifiers> = Mutex::new(Modifiers {
    l_shift: false,
    r_shift: false,
    l_ctrl: false,
    r_ctrl: false,
    l_alt: false,
    r_alt: false,
});

pub fn _start() -> ! {
    display::clear(RGBColor::new(0,0,0));
    print!("\x0C\r");
    print!("> ");

    loop {
        match keyboard::poll_event() {
            Some(event) => keyboard_event(event),
            None => unsafe { asm!("hlt") }
        }
    }
}

fn keyboard_event(event: u16) {
    let keycode = KeyCode::from_u16(event);
    let is_release = KeyCode::is_release(event);
    match keycode {
        Some(KeyCode::LeftShift) => {MODIFIERS.lock().l_shift = !is_release;},
        Some(KeyCode::RightShift) => {MODIFIERS.lock().r_shift = !is_release;},
        Some(KeyCode::LeftControl) => {MODIFIERS.lock().l_ctrl = !is_release;},
        Some(KeyCode::RightControl) => {MODIFIERS.lock().r_ctrl = !is_release;},
        Some(KeyCode::LeftAlt) => {MODIFIERS.lock().l_alt = !is_release;},
        Some(KeyCode::RightAlt) => {MODIFIERS.lock().r_alt = !is_release;},
        Some(keycode) => {
            if is_release {return}

            let modifiers = MODIFIERS.lock();
            let shift = modifiers.l_shift || modifiers.r_shift;
            let altgr = modifiers.r_alt;
            if let Some(ch) = get_finnish_char(keycode, shift, altgr) {
                input_char(ch);
            }
        }
        None => {panic!("Unknown keycode: 0x{:X?}", event)}
    }
}

fn input_char(ch: char) {
    print!("{}", ch);
}