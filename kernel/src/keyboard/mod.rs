use core::arch::asm;
use core::sync::atomic::{AtomicU8, Ordering};
use crate::interrupts::{set_interrupt_handler, InterruptStackFrame};
use crate::sync::{CircularBuffer, OnceInit};

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
        set_interrupt_handler(33, keyboard_handler);
        EVENT_BUFFER.init(CircularBuffer::allocate(1));
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

static EXTENDED: AtomicU8 = AtomicU8::new(0);

static EVENT_BUFFER: OnceInit<CircularBuffer<u16>> = unsafe { OnceInit::new() };

pub fn poll_event() -> Option<u16> {
    let mut event_buffer = EVENT_BUFFER.get().reader();
    if event_buffer.size() == 0 {
        return None;
    }
    Some(event_buffer.pop_front())
}

extern "x86-interrupt" fn keyboard_handler(_: InterruptStackFrame) {
    unsafe {
        // Read the scancode from the PS/2 data port
        let scancode = inb(0x60);
        let extended = EXTENDED.swap(0, Ordering::Relaxed);

        if scancode == 0xE0 {
            EXTENDED.store(scancode, Ordering::Relaxed);
            outb(0x20, 0x20);
            return;
        }

        let mut event_buffer = EVENT_BUFFER.get().writer();
        event_buffer.push_back((extended as u16) << 8 | scancode as u16);
        outb(0x20, 0x20);
    }
}