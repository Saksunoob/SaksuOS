#![no_std]
#![no_main]

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(mem_copy_fn)]

mod elf;
mod console;
mod file;
mod pages;
mod boot_info;
mod memory;

use core::arch::asm;
use core::ptr::{null, null_mut, slice_from_raw_parts, slice_from_raw_parts_mut};
use core::slice;
use uefi_raw::{Handle, PhysicalAddress, Status, VirtualAddress};
use uefi_raw::protocol::console::{GraphicsOutputModeInformation, GraphicsOutputProtocol};
use uefi_raw::table::boot::{BootServices, MemoryDescriptor, PAGE_SIZE};
use uefi_raw::table::system::SystemTable;
use macros::uefistr;
use crate::boot_info::{BootInfo, FrameBuffer};
use crate::console::ConsoleOut;
use crate::elf::{DynamicSectionEntry, DynamicEntryType, Elf64, RELAEntry, RelocationType, SegmentType};
use crate::file::{get_file_info, get_root_file_handle, read_file};
use crate::memory::EfiMemoryDescriptor;
use crate::pages::{allocate_pages, init_paging, map_page, switch_l4};

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

    let mut min_page = usize::MAX;
    let mut max_page = usize::MIN;

    for segment in kernel.program_headers() {
        match segment.segment_type {
            SegmentType::Load | SegmentType::Dynamic => {
                let start = segment.vaddr as usize;
                let end = (segment.vaddr + segment.mem_size) as usize;

                min_page = min_page.min(start/PAGE_SIZE);
                max_page = max_page.max((end-1)/PAGE_SIZE);
            },
            _ => continue,
        }
    }

    let kernel_page_count = max_page+1;
    let kernel_pages = allocate_pages(bs, kernel_page_count);

    let base = 0xffff800000000000;

    let relocate = |offset: u64, r_type: RelocationType, addend: u64| {
        stdout().printhex(offset);
        stdout().printstr(": ");
        stdout().printhex(addend);
        stdout().printstr("=>");
        stdout().printhex(base + addend);
        let addr = unsafe { kernel_pages.byte_add(offset as usize) };

        unsafe { match r_type {
            //RelocationType::B64 => (addr as *mut u64).write((addr as *mut u64).read() + addend),
            RelocationType::RELATIVE => {
                stdout().printstr(" | ");
                stdout().printhex(addr as u64);
                stdout().printstr(": ");
                stdout().printhex((addr as *mut u64).read());
                (addr as *mut u64).write(base + addend);
                stdout().printstr(" => ");
                stdout().printhex((addr as *mut u64).read());
            },
            _ => {
                stdout().printdec::<u32>(core::mem::transmute(r_type));
                panic!("Unsupported relocation type ")
            }
        } }
        stdout().printstr("\r\n");
    };

    for segment in kernel.program_headers() {
        match segment.segment_type {
            SegmentType::Load => {
                let dst = unsafe { kernel_pages.byte_add(segment.vaddr as usize) };
                let src = unsafe { kernel_binary.byte_add(segment.offset as usize) };
                let bytes = segment.file_size as usize;

                if segment.vaddr as usize+bytes >= (kernel_page_count)*PAGE_SIZE {
                    stdout().printhex(((kernel_page_count)*PAGE_SIZE) as u64);
                    panic!("Needed pages exceeds allocated");
                }

                unsafe { core::ptr::copy_nonoverlapping(src, dst, bytes) }
                unsafe { core::ptr::write_bytes(dst.byte_add(bytes), 0, segment.mem_size as usize - bytes) }
            },
            SegmentType::Dynamic => {
                let mut rela_ptr = null();
                let mut rela_size = 0;
                let mut start = segment.offset as usize;
                loop {
                    let entry = unsafe { (kernel_binary.byte_add(start) as *const DynamicSectionEntry).as_ref().unwrap() };
                    stdout().printstr("Tag: 0x");
                    stdout().printhex::<u64>( unsafe { core::mem::transmute_copy(&entry.d_tag) });
                    stdout().printstr(" = ");
                    stdout().printdec::<u64>( unsafe { core::mem::transmute_copy(&entry.d_tag) });
                    stdout().printstr("\n\r");
                    match entry.d_tag {
                        DynamicEntryType::Null => break,
                        DynamicEntryType::Rela => {
                            unsafe { rela_ptr = kernel_binary.byte_add(entry.d_val as usize) as *const RELAEntry }
                        },
                        DynamicEntryType::RelaSize => {
                            rela_size = entry.d_val as usize;
                        },
                        DynamicEntryType::Rel => panic!("Unexpected DT_REL"),
                        DynamicEntryType::JumpRel => panic!("Unexpected DT_JMPREL"),
                        _ => {
                        }
                    }

                    start += size_of::<DynamicSectionEntry>();
                }

                if rela_size != 0 {
                    let entries = unsafe {
                        let size = rela_size / size_of::<RELAEntry>();
                        slice_from_raw_parts(rela_ptr, size).as_ref().unwrap()
                    };

                    for entry in entries {
                        relocate(entry.offset, entry.relocation_type, entry.addend);
                    }
                }
            }
            _ => continue,
        }
    }

    stdout().printstr("\n\r");

    init_paging(bs);
    stdout().printhex(kernel_page_count as u64);
    stdout().printstr("\n\r");
    for i in 0..kernel_page_count {
        map_page(bs, (kernel_pages as usize + i*PAGE_SIZE) as PhysicalAddress,
                 (0xffff800000000000+i*PAGE_SIZE) as VirtualAddress);
        stdout().printhex((0xffff800000000000+i*PAGE_SIZE) as u64);
        stdout().printstr("=>");
        stdout().printhex((kernel_pages as usize + i*PAGE_SIZE) as u64);
        stdout().printstr("\n\r");
    }

    let kernel_stack_pages = 4;
    let kernel_stack = allocate_pages(bs, kernel_stack_pages);
    for i in 0..kernel_stack_pages {
        map_page(bs, (kernel_stack as usize + i*PAGE_SIZE) as PhysicalAddress,
                 (0xffff800000000000+(kernel_page_count+i)*PAGE_SIZE) as VirtualAddress)
    }

    let framebuffer = unsafe {
        let mut handles_ptr: *mut Handle = null_mut();
        let mut handle_count = 0;

        let _ = ((*bs).locate_handle_buffer)(
            2,
            &GraphicsOutputProtocol::GUID,
            null_mut(),
            &mut handle_count,
            &mut handles_ptr,
        );

        let handles = slice::from_raw_parts(handles_ptr, handle_count);

        // Use the first GOP handle
        let mut graphics_handle: *mut GraphicsOutputProtocol = null_mut();
        let status = ((*bs).handle_protocol)(
            handles[0],
            &GraphicsOutputProtocol::GUID,
            as_ptr_to(&mut graphics_handle),
        );
        if status != Status::SUCCESS {
            panic!("Failed to get graphics handle");
        }

        let mut max_res: (u32, u64) = (0, 0);
        let mut info: *const GraphicsOutputModeInformation = null();
        let mut size = size_of::<GraphicsOutputProtocol>();
        let mut status;
        let mut i = 0;
        loop {
            status = ((*graphics_handle).query_mode)(graphics_handle, i, &mut size, as_ptr_to(&mut info));
            if status != Status::SUCCESS {
                break;
            }
            let width = (*info).horizontal_resolution as u64;
            let height = (*info).vertical_resolution as u64;
            let size =  width * height;
            if width/16 == height/9 && size > max_res.1 {
                max_res = (i, size);
            }
            i += 1;
        }
        stdout().printdec(max_res.0);
        let _ = ((*graphics_handle).set_mode)(graphics_handle, max_res.0);

        let ptr = (*(*graphics_handle).mode).frame_buffer_base;
        let size = (*(*graphics_handle).mode).frame_buffer_size;
        let info = (*(*graphics_handle).mode).info;
        let width = (*info).horizontal_resolution as usize;
        let height = (*info).vertical_resolution as usize;
        let stride = (*info).pixels_per_scan_line as usize;
        let format = (*info).pixel_format;

        let slice = slice_from_raw_parts_mut(ptr as *mut [u8; 4], size/4).as_mut().unwrap();

        FrameBuffer {
            buffer: slice,
            stride,
            height,
            width,
            pixel_format: core::mem::transmute(format.0)
        }
    };

    switch_l4();

    let entry_addr: u64 = 0xffff800000000000 + kernel.entry;
    let stack_addr: usize = 0xffff800000000000 + (kernel_page_count+kernel_stack_pages-1)*PAGE_SIZE;

    //stdout().printhex(core::ptr::from_ref(boot_info.framebuffer.buffer).addr() as u64);

    //boot_info.framebuffer.clear(RGBColor::new(255, 255, 255));

    let (memory_map, map_key) = get_memory_map(bs);
    let status = unsafe { ((*bs).exit_boot_services)(image, map_key) };
    if status != Status::SUCCESS {
        panic!("Failed to exit boot services");
    }

    let boot_info = BootInfo {
        framebuffer,
        memory_map,
        uefi_runtime_services: unsafe {(*st).runtime_services}
    };

    unsafe {
        asm!(
            "push rbp",
            "mov rbp, rsp",
            "mov rsp, {stack_addr}",
            "mov rdi, {bi}",
            "call {entry}",
            stack_addr = in(reg) stack_addr,
            entry = in(reg) entry_addr,
            bi = in(reg) &boot_info,
        )
    }
    loop {}
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

fn get_memory_map(bs: *mut BootServices) -> (&'static [EfiMemoryDescriptor], usize) {
    let mut size: usize = 0;
    let mut map_ptr: *mut MemoryDescriptor = null_mut();
    let mut key = 0;
    let mut desc_size = 0;
    let mut desc_version = 0;

    unsafe {
        let _ = ((*bs).get_memory_map)(&mut size, map_ptr, &mut key, &mut desc_size, &mut desc_version);

        let needed_pages = (size+2*size_of::<EfiMemoryDescriptor>()).div_ceil(PAGE_SIZE);
        map_ptr = allocate_pages(bs, needed_pages) as *mut MemoryDescriptor;

        let _ = ((*bs).get_memory_map)(&mut size, map_ptr, &mut key, &mut desc_size, &mut desc_version);

        (slice::from_raw_parts(map_ptr as *mut EfiMemoryDescriptor, size).as_ref(), key)
    }
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