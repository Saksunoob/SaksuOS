#![no_std]
#![no_main]

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(mem_copy_fn)]

mod elf;
mod console;
mod file;
mod pages;

use core::ffi::c_void;
use core::mem;
use core::ptr::null;
use uefi_raw::{Guid, Handle, Status};
use uefi_raw::protocol::file_system::{FileAttribute, FileInfo, FileMode, FileProtocolV1, SimpleFileSystemProtocol};
use uefi_raw::protocol::loaded_image::LoadedImageProtocol;
use uefi_raw::table::boot::{BootServices, PAGE_SIZE};
use uefi_raw::table::system::SystemTable;
use macros::uefistr;
use crate::console::ConsoleOut;
use crate::elf::{Elf64, SegmentType};
use crate::file::{get_file_info, get_root_file_handle, read_file};
use crate::pages::allocate_pages;

static mut STDOUT: *const ConsoleOut = null();

// UEFI entry point
#[unsafe(no_mangle)]
pub extern "efiapi" fn efi_main(image: Handle, st: *mut SystemTable) -> usize {
    unsafe {
        STDOUT = &ConsoleOut::new((*st).stdout)
    }
    let bs = unsafe {(*st).boot_services};

    let filename = uefistr!("KERNEL");
    let kernel_handle = get_root_file_handle(image, bs, filename);
    let file_info = get_file_info::<6>(kernel_handle);

    let kernel_binary = read_file(bs, kernel_handle, file_info);
    let kernel = Elf64::new(kernel_binary).unwrap();

    let kernel_pages = allocate_pages(bs, 3);

    for segment in kernel.program_headers() {
        match segment.segment_type {
            SegmentType::Load => {
                let dst = unsafe { kernel_pages.byte_add(segment.vaddr as usize) };
                let src = unsafe { kernel_binary.byte_add(segment.offset as usize) };
                let bytes = segment.file_size as usize;

                if segment.vaddr as usize+bytes >= 3*PAGE_SIZE {
                    panic!("Needed pages exceeds 3")
                }

                unsafe { core::ptr::copy_nonoverlapping(src, dst, bytes) }
            },
            _ => continue,
        }
    }
    let entry: fn() -> u64 = unsafe { mem::transmute(kernel_pages.byte_add(kernel.entry as usize)) };
    stdout().printdec(entry());

    loop {}
    0
}

fn stdout() -> &'static ConsoleOut {
    unsafe { match STDOUT.as_ref() {
        Some(stdout) => stdout,
        None => panic!("Failed to get stdout"),
    }}
}

const fn as_ptr_to<T, C>(_ref: &mut T) -> *mut C {
    _ref as *mut T as *mut C
}

// Required for no_std
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    if let Some(stdout) = unsafe { STDOUT.as_ref() } {
        match info.message().as_str() {
            Some(msg) => stdout.printstr(msg),
            None => ()
        }
        if let Some(pos) = info.location() {
            stdout.printstr("\n\rat ");
            stdout.printstr(pos.file());
            stdout.printstr(":");
            stdout.printdec(pos.line());
            stdout.printstr(":");
            stdout.printdec(pos.column());
        }
    }
    loop {}
}