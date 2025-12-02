use core::ffi::c_void;
use core::ops::{Deref, DerefMut};
use uefi_raw::{Guid, Handle, Status};
use uefi_raw::protocol::file_system::{FileAttribute, FileInfo, FileMode, FileProtocolV1, SimpleFileSystemProtocol};
use uefi_raw::protocol::loaded_image::LoadedImageProtocol;
use uefi_raw::table::boot::{BootServices, PAGE_SIZE};
use crate::as_ptr_to;
use crate::pages::allocate_pages;

pub fn get_root_file_handle(image: Handle, bs: *mut BootServices, filename: *mut u16) -> &'static mut FileProtocolV1 {
    let mut loaded_image: *mut LoadedImageProtocol = core::ptr::null_mut();
    let status = unsafe {
        ((*bs).handle_protocol)(
            image,
            &LoadedImageProtocol::GUID as *const Guid,
            &mut loaded_image as *mut _ as *mut *mut c_void,
        )
    };
    if status != Status::SUCCESS {
        panic!("uefi failed to get loaded image protocol");
    }

    //
    // 2. Get SimpleFileSystemProtocol from the device that loaded us
    //
    let mut fs: *mut SimpleFileSystemProtocol = core::ptr::null_mut();
    let status = unsafe { ((*bs).handle_protocol)(
        (*loaded_image).device_handle,
        &SimpleFileSystemProtocol::GUID as *const Guid,
        &mut fs as *mut _ as *mut *mut c_void,
    )};
    if status != Status::SUCCESS {
        panic!("uefi failed to get loaded image protocol");
    }

    let mut root: *mut FileProtocolV1 = core::ptr::null_mut();
    let status = unsafe { ((*fs).open_volume)(fs, &mut root) };
    if status != Status::SUCCESS {
        panic!("uefi failed to open volume");
    }

    let mut file: *mut FileProtocolV1 = core::ptr::null_mut();
    let status = unsafe { ((*root).open)(
        root,
        &mut file,
        filename,
        FileMode::READ,
        FileAttribute::empty(),
    )};
    if status != Status::SUCCESS {
        panic!("uefi failed to open kernel file");
    }
    match unsafe { file.as_mut() } {
        Some(file) => file,
        None => panic!("uefi failed to open kernel file")
    }
}

#[repr(C)]
pub struct SizedFileInfo<const SIZE: usize> {
    file_info: FileInfo,
    name: [u16; SIZE],
    null_terminator: u16,
}

impl<const SIZE: usize> Deref for SizedFileInfo<SIZE> {
    type Target = FileInfo;

    fn deref(&self) -> &Self::Target {
        &self.file_info
    }
}
impl<const SIZE: usize> DerefMut for SizedFileInfo<SIZE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.file_info
    }
}

pub fn get_file_info<const SIZE: usize>(file: &mut FileProtocolV1) -> SizedFileInfo<SIZE> {
    let mut file_info: SizedFileInfo<SIZE> = unsafe { core::mem::zeroed() };

    let status = unsafe {
        let mut size = size_of::<SizedFileInfo<SIZE>>();
        let mut guid = FileInfo::ID;
        ((*file).get_info)(file, &mut guid, &mut size, as_ptr_to(&mut file_info))
    };
    match status {
        Status::SUCCESS => file_info,
        Status::UNSUPPORTED => panic!("Unsupported kernel handle!"),
        Status::NO_MEDIA => panic!("No media info available"),
        Status::DEVICE_ERROR => panic!("Device error"),
        Status::VOLUME_CORRUPTED => panic!("Volume is corrupted"),
        Status::BUFFER_TOO_SMALL => panic!("Buffer is too small"),
        _ => panic!("Unexpected status: {:?}", status),
    }
}

pub fn read_file<const SIZE: usize>(bs: *mut BootServices, file: &mut FileProtocolV1, file_info: SizedFileInfo<SIZE>) -> *mut u8 {
    let mut file_size = file_info.file_size as usize;
    let pages = file_size.div_ceil(PAGE_SIZE);
    let buffer = allocate_pages(bs, pages);

    let status = unsafe {
        (file.read)(file, &mut file_size, buffer as *mut c_void)
    };

    if status != Status::SUCCESS {
        panic!("uefi failed to read file");
    }
    buffer
}

