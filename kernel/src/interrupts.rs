use core::arch::asm;
use core::ptr::null_mut;
use macros::generic_handlers;
use crate::gdt::KERNEL_CS;
use crate::memory::allocate_page;

// Interrupt Stack Frame structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InterruptStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

// IDT Entry (Gate Descriptor)
#[repr(C, packed)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntry {
    const fn new() -> Self {
        IdtEntry {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    fn set_handler(&mut self, handler: u64) {
        self.offset_low = handler as u16;
        self.offset_mid = (handler >> 16) as u16;
        self.offset_high = (handler >> 32) as u32;
        self.selector = KERNEL_CS as u16; // Kernel code segment selector
        self.ist = 0;
        self.type_attr = 0x8E; // Present, DPL 0, 64-bit interrupt gate
        self.reserved = 0;
    }
}

// IDT Pointer structure
#[repr(C, packed)]
struct IdtPtr {
    limit: u16,
    base: *const IdtEntry,
}

static mut IDT: *mut IdtEntry = null_mut();

pub fn init_idt() {
    unsafe {
        let idt = allocate_page() as *mut IdtEntry;
        IDT = idt;


        // Set default handlers for all interrupts
        for i in 0..256 {
            let mut entry = IdtEntry::new();
            entry.set_handler(get_generic_handler(i as u8));
            IDT.add(i).write(entry);
        }

        set_interrupt_handler_with_error(13, gpf_handler);

        // Load IDT
        let idt_ptr = IdtPtr {
            limit: (size_of::<IdtEntry>()*256 - 1) as u16,
            base: IDT,
        };

        asm!("lidt [{}]", in(reg) &idt_ptr, options(nostack, preserves_flags));
    }
}

fn get_generic_handler(id: u8) -> u64 {
    generic_handlers!(id)
}

pub fn set_interrupt_handler(index: usize, handler: extern "x86-interrupt" fn (InterruptStackFrame)) {
    unsafe {
        IDT.add(index).as_mut().unwrap().set_handler(handler as u64);
    }
}

pub fn set_interrupt_handler_with_error(index: usize, handler: extern "x86-interrupt" fn (InterruptStackFrame, u64)) {
    unsafe {
        IDT.add(index).as_mut().unwrap().set_handler(handler as u64);
    }
}

extern "x86-interrupt" fn gpf_handler(_stack_frame: InterruptStackFrame, error_code: u64) {
    println!("GPF handler error: 0b{:b}", error_code);
    loop {}
}

// Default handler for unhandled interrupts
extern "x86-interrupt" fn default_handler<const I: u8>(_stack_frame: InterruptStackFrame) {
    println!("Unhandled interrupt: {}", I);
}