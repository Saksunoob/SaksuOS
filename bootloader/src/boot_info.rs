use uefi_raw::table::runtime::RuntimeServices;
use crate::memory::EfiMemoryDescriptor;

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum PixelFormat {
    RGB = 0,
    BGR = 1,
    BitMask = 2,
    BltOnly = 3,
}

pub trait Color {
    fn get_rgba(&self) -> [u8; 4];
    fn get_bgra(&self) -> [u8; 4];
}

#[repr(C)]
pub struct RGBColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl RGBColor {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

impl Color for RGBColor {
    fn get_rgba(&self) -> [u8; 4] {[self.red, self.green, self.blue, 0]}
    fn get_bgra(&self) -> [u8; 4] {[self.blue, self.green, self.red, 0]}
}

#[repr(C)]
pub struct FrameBuffer {
    pub buffer: &'static mut [[u8; 4]],
    pub stride: usize,
    pub height: usize,
    pub width: usize,
    pub pixel_format: PixelFormat,
}

impl FrameBuffer {
    pub fn clear<C: Color>(&mut self, color: C) {
        let color = match self.pixel_format {
            PixelFormat::RGB => color.get_rgba(),
            PixelFormat::BGR => color.get_bgra(),
            _ => panic!("unsupported pixel format"),
        };

        self.buffer.fill(color);
    }
}

#[repr(C)]
pub struct BootInfo {
    pub framebuffer: FrameBuffer,
    pub memory_map: &'static [EfiMemoryDescriptor],
    pub uefi_runtime_services: *mut RuntimeServices
}