use core::arch::asm;
use core::sync::atomic::{AtomicBool, Ordering};
use crate::interrupts::{set_interrupt_handler, InterruptStackFrame};

#[inline(always)]
unsafe fn outb(port: u16, val: u8) { unsafe {
    asm!(
        "out dx, al",
        in("dx") port,
        in("al") val,
        options(nomem, nostack, preserves_flags)
    );
}}

#[inline(always)]
unsafe fn inb(port: u16) -> u8 { unsafe {
    let res: u8;
    asm!(
        "in al, dx",
        out("al") res,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    res
}}

/// This function must be called only once
pub unsafe fn init() {
    unsafe {
        pic_remap();
        set_interrupt_handler(33, keyboard_handler)
    }
}

unsafe fn pic_remap() {
    unsafe {
        // Save masks
        let m1 = inb(0x21);
        let m2 = inb(0xA1);

        // Start initialization
        outb(0x20, 0x11);
        outb(0xA0, 0x11);

        outb(0x21, 0x20); // Master offset (32)
        outb(0xA1, 0x28); // Slave offset (40)

        outb(0x21, 0x04); // Tell Master about Slave
        outb(0xA1, 0x02); // Tell Slave its identity

        // 8086 mode
        outb(0x21, 0x01);
        outb(0xA1, 0x01);

        // Restore masks
        outb(0x21, m1 & 0xFD); // 0xFD = 11111101 (only IRQ1 enabled)
        outb(0xA1, m2);
    }
}

const SCANCODE_MAP: [char; 0x3A] = [
    '\0', '\0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '+', '´', '\x08',
    '\t', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', 'å', '¨', '\n',
    '\0', 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', 'ö', 'ä', '\0',
    '\0', '\'', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '-',
    '\0', '\0', '\0', ' '
];

static EXTENDED: AtomicBool = AtomicBool::new(false);

extern "x86-interrupt" fn keyboard_handler(_: InterruptStackFrame) {
    unsafe {
        // Read the scancode from the PS/2 data port
        let scancode = inb(0x60);
        let extended = EXTENDED.swap(false, Ordering::Relaxed);

        // Filter out "break" codes (key releases) and out-of-bounds
        if !extended && scancode < 0x3A {
            let key = SCANCODE_MAP[scancode as usize];
            if key != '\0' {
                println!("DOWN {} ({:x})", key, scancode);
            }
        }
        else if !extended && scancode >= 0x80 && scancode < 0x80+0x3A {
            let key = SCANCODE_MAP[scancode as usize - 0x80];
            if key != '\0' {
                println!("UP   {} ({:x})", key, scancode);
            }
        } else if scancode == 0xE0 {
            EXTENDED.store(true, Ordering::Relaxed);
        } else if extended {
            println!("UNKNOWN EXTENDED: {:x}", scancode);
        } else {
            println!("UNKNOWN: {:x}", scancode);
        }

        // Send EOI to the PIC
        outb(0x20, 0x20);
    }
}