/*
 * Copyright 2024 Luc Lenôtre
 *
 * This file is part of Maestro.
 *
 * Maestro is free software: you can redistribute it and/or modify it under the
 * terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or (at your option) any later
 * version.
 *
 * Maestro is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
 * A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * Maestro. If not, see <https://www.gnu.org/licenses/>.
 */

//! Under the x86 architecture, the GDT (Global Descriptor Table) is a table of
//! structure that describes the segments of memory.
//!
//! It is a deprecated structure that still must be used in order to switch to protected mode,
//! handle protection rings and load the Task State Segment (TSS).

use core::{
    arch::asm,
    fmt,
    ptr::{self, addr_of},
};
use core::arch::naked_asm;
use crate::sync::mutex::Mutex;

/// The offset of the kernel code segment.
pub const KERNEL_CS: usize = 8;
/// The offset of the kernel data segment.
pub const KERNEL_DS: usize = 16;
/// The offset of the user code segment.
pub const USER_CS: usize = 24;
/// The offset of the user data segment (32 bits).
pub const USER_DS: usize = 32;
/// The offset of the user data segment (64 bits).
pub const USER_CS64: usize = 40;
/// The offset of the Task State Segment (TSS).
pub const TSS_OFFSET: usize = 48;
/// The offset of Thread Local Storage (TLS) entries.
pub const TLS_OFFSET: usize = 64;

/// A GDT entry.
#[repr(C, align(8))]
#[derive(Clone, Copy, Default)]
pub struct Entry(pub u64);

impl Entry {
    /// Creates a new entry with the give information.
    #[inline(always)]
    pub const fn new(base: u32, limit: u32, access_byte: u8, flags: u8) -> Self {
        let mut ent = Self(0);
        ent.set_base(base);
        ent.set_limit(limit);
        ent.set_access_byte(access_byte);
        ent.set_flags(flags);
        ent
    }

    /// Creates a long mode entry, spanning two regular entries.
    pub const fn new64(base: u64, limit: u32, access_byte: u8, flags: u8) -> [Self; 2] {
        [
            Self::new((base & 0xffffffff) as _, limit, access_byte, flags),
            Self((base >> 32) & 0xffffffff),
        ]
    }

    /// Returns the entry's base address.
    #[inline(always)]
    pub const fn get_base(&self) -> u32 {
        (((self.0 >> 16) & 0xffffff) | ((self.0 >> 32) & 0xff000000)) as _
    }

    /// Sets the entry's base address.
    #[inline(always)]
    pub const fn set_base(&mut self, base: u32) {
        self.0 &= !(0xffffff << 16);
        self.0 &= !(0xff << 56);

        self.0 |= (base as u64 & 0xffffff) << 16;
        self.0 |= ((base as u64 >> 24) & 0xff) << 56;
    }

    /// Returns the entry's limit.
    #[inline(always)]
    pub const fn get_limit(&self) -> u32 {
        ((self.0 & 0xffff) | (((self.0 >> 48) & 0xf) << 16)) as _
    }

    /// Sets the entry's limit.
    ///
    /// If the given limit is more than `pow(2, 20) - 1`, the value is truncated.
    #[inline(always)]
    pub const fn set_limit(&mut self, limit: u32) {
        self.0 &= !0xffff;
        self.0 &= !(0xf << 48);

        self.0 |= limit as u64 & 0xffff;
        self.0 |= ((limit as u64 >> 16) & 0xf) << 48;
    }

    /// Returns the value of the access byte.
    #[inline(always)]
    pub const fn get_access_byte(&self) -> u8 {
        ((self.0 >> 40) & 0xff) as _
    }

    /// Sets the value of the access byte.
    #[inline(always)]
    pub const fn set_access_byte(&mut self, byte: u8) {
        self.0 &= !(0xff << 40);
        self.0 |= (byte as u64) << 40;
    }

    /// Returns the flags.
    #[inline(always)]
    pub const fn get_flags(&self) -> u8 {
        ((self.0 >> 52) & 0x0f) as _
    }

    /// Sets the flags.
    #[inline(always)]
    pub const fn set_flags(&mut self, flags: u8) {
        self.0 &= !(0x0f << 52);
        self.0 |= ((flags as u64) & 0x0f) << 52;
    }

    /// Tells whether the entry is present.
    #[inline(always)]
    pub const fn is_present(&self) -> bool {
        (self.0 >> 47) & 1 != 0
    }

    /// Sets the entry present or not.
    #[inline(always)]
    pub const fn set_present(&mut self, present: bool) {
        if present {
            self.0 |= 1 << 47;
        } else {
            self.0 &= !(1 << 47);
        }
    }

    /// Updates the entry at index of the GDT with the current entry.
    pub unsafe fn update_gdt(self, index: usize) {
        unsafe {
            let ptr = get_segment_ptr(index);
            ptr::write_volatile(ptr, self);
        }
    }
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Entry")
            .field("base", &self.get_base())
            .field("limit", &self.get_limit())
            .field("access_byte", &self.get_access_byte())
            .field("flags", &self.get_flags())
            .field("present", &self.is_present())
            .finish()
    }
}

/// Returns the pointer to the segment at index `index` in the GDT.
///
/// # Safety
///
/// The caller must ensure the given `index` is in bounds of the GDT.
pub unsafe fn get_segment_ptr(index: usize) -> *mut Entry {
    addr_of!(GDT.lock()[index]) as *mut Entry
}

pub type InitGdt = [Entry; 11];

static GDT: Mutex<InitGdt> = Mutex::new([
    // First entry, empty
    Entry(0),
    // Kernel code segment
    Entry::new(0, !0, 0b10011010, 0b1010),
    // Kernel data segment
    Entry::new(0, !0, 0b10010010, 0b1100),
    // User code segment (32 bits)
    Entry::new(0, !0, 0b11111010, 0b1100),
    // User data segment (32 bits)
    Entry::new(0, !0, 0b11110010, 0b1100),
    // User code segment (64 bits), unused by 32 bit kernel
    Entry::new(0, !0, 0b11111010, 0b1010),
    // TSS
    Entry(0),
    Entry(0),
    // TLS entries
    Entry(0),
    Entry(0),
    Entry(0),
]);

/// A GDT descriptor.
#[repr(C, packed)]
struct Gdt {
    /// The size of the GDT in bytes, minus `1`.
    size: u16,
    /// The address to the GDT.
    addr: usize,
}

/// Refreshes the GDT's cache.
#[inline(always)]
pub fn flush() {
    let gdt = Gdt {
        size: (size_of::<InitGdt>() - 1) as _,
        addr: GDT.lock().inner_ptr() as usize,
    };
    unsafe {
        asm!("lgdt [{}]",
            in(reg) &gdt);
        asm!(
            "mov ss, {}",
            "push {}",
            "lea {tmp}, [3f + rip]",
            "push {tmp}",
            "retfq",
            "3:",
            in(reg) KERNEL_DS,
            in(reg) KERNEL_CS,
            tmp = lateout(reg) _,
            options(preserves_flags),
        );
    }
}

#[unsafe(naked)]
extern "C" fn reload_segments() {
    naked_asm!(
        "mov ax, 0x10",   // KERNEL_DS
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",

        // Reload stack segment, must be serialized
        "mov ss, ax",
        "jmp 1f",         // serialize SS load
        "1:",

        // Prepare far return
        "pop rax",        // pop original RIP
        "push 0x8",       // push KERNEL_CS
        "push rax",       // push original RIP
        "retf",           // far return to original caller with new CS
    )
}