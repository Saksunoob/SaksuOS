use core::fmt::{Arguments, Write};
use core::ptr::null_mut;
use bootloader::boot_info::{FrameBuffer, PixelFormat, RGBColor, Color};
use crate::display::font::{get_char, get_font_bounds};
use crate::memory::{allocate_pages, PAGE_SIZE};

mod font;

static mut FRAMEBUFFER: *mut FrameBuffer = null_mut();
static mut BACKBUFFER: *mut u8 = null_mut();
static mut TERMINAL: Terminal = Terminal::new();
static mut COUNT: usize = 0;
const DEBUG_DOT_SIZE: usize = 4;

fn framebuffer() -> &'static mut FrameBuffer {
    unsafe { FRAMEBUFFER.as_mut().unwrap() }
}

fn backbuffer() -> &'static mut [[u8; 4]] {
    let framebuffer = framebuffer();
    let size = framebuffer.width * framebuffer.height;
    unsafe { core::slice::from_raw_parts_mut(BACKBUFFER as *mut [u8; 4], size) }
}

pub fn init(frame_buffer: &mut FrameBuffer) {
    unsafe {
        FRAMEBUFFER = frame_buffer;
        let size = frame_buffer.stride * frame_buffer.height * 4;
        let pages = size.div_ceil(PAGE_SIZE);
        BACKBUFFER = allocate_pages(pages);
    }
}

pub fn debug_dot(color: RGBColor) {
    let y = unsafe { COUNT } * DEBUG_DOT_SIZE / framebuffer().width * DEBUG_DOT_SIZE;
    let x = unsafe { COUNT } * DEBUG_DOT_SIZE % framebuffer().width;

    let color = match framebuffer().pixel_format {
        PixelFormat::RGB => color.get_rgba(),
        PixelFormat::BGR => color.get_bgra(),
        _ => color.get_rgba(),
    };

    rect(color, x, y, DEBUG_DOT_SIZE, DEBUG_DOT_SIZE);
    flush_rows(y, DEBUG_DOT_SIZE);
    unsafe { COUNT += 1 }
}

fn rect(color: [u8; 4], x: usize, y: usize, width: usize, height: usize) {
    for y in y..y+height {
        for x in x..x+width {
            backbuffer()[y*framebuffer().stride+x] = color
        }
    }
}

pub fn flush() {
    let framebuffer = framebuffer();
    let backbuffer = backbuffer();
    let pixels = framebuffer.stride * framebuffer.height;
    unsafe { core::ptr::copy_nonoverlapping(backbuffer.as_ptr(), framebuffer.buffer.as_mut_ptr(), pixels) }
}

pub fn flush_rows(start: usize, count: usize) {
    let framebuffer = framebuffer();
    let backbuffer = backbuffer();
    let offset = start * framebuffer.stride;
    let pixels = framebuffer.stride * count;
    unsafe {
        let src = backbuffer.as_ptr().byte_add(offset);
        let dst = framebuffer.buffer.as_mut_ptr().byte_add(offset);
        core::ptr::copy_nonoverlapping(src, dst, pixels);
    }
}

pub fn draw_char(ch: char, x: usize, y: usize, foreground_color: RGBColor, background_color: RGBColor) {
    let fg = match framebuffer().pixel_format {
        PixelFormat::RGB => foreground_color.get_rgba(),
        PixelFormat::BGR => foreground_color.get_bgra(),
        _ => foreground_color.get_rgba(),
    };
    let bg = match framebuffer().pixel_format {
        PixelFormat::RGB => background_color.get_rgba(),
        PixelFormat::BGR => background_color.get_bgra(),
        _ => background_color.get_rgba(),
    };

    let bounds = get_font_bounds();
    let x = x as i64*bounds.width();
    let y = y as i64*bounds.height();

    rect(bg, x as usize, y as usize, bounds.width() as usize, bounds.height() as usize);

    let ch = get_char(ch);
    match ch {
        Some(ch) => {
            let ch_bounds = ch.bounds();
            let x_offset = (x+ch_bounds.x()-bounds.x()) as usize;
            let y_offset = (y+ch_bounds.y()-bounds.y()) as usize;

            for (l, line) in ch.bitmap().iter().enumerate() {
                for p in 0..ch.bounds().width() {
                    let v = (line >> (7-p)) & 1 > 0;
                    if v {
                        rect(fg, x_offset + p as usize, y_offset+l, 1, 1)
                    }
                }
            }
        },
        None => {debug_dot(RGBColor::new(255, 0, 0));}
    }
}

pub fn clear(color: RGBColor) {
    let color = match framebuffer().pixel_format {
        PixelFormat::RGB => color.get_rgba(),
        PixelFormat::BGR => color.get_bgra(),
        _ => color.get_rgba(),
    };
    backbuffer().fill(color);
    flush();
}

struct Terminal {
    cursor: (usize, usize)
}

impl Terminal {
    const fn new() -> Terminal { Terminal { cursor: (0, 0) } }

    fn max_cursor_x() -> usize {
        framebuffer().width / get_font_bounds().width() as usize
    }

    fn print_char(&mut self, ch: char) {
        match ch {
            '\x08' => {
                if self.cursor.0 > 0 {
                    self.cursor.0 -= 1;
                    draw_char('\0', self.cursor.0, self.cursor.1,
                              RGBColor::new(255, 255, 255),
                              RGBColor::new(0, 0, 0));
                }
            }
            '\t' => {
                self.cursor.0 = (self.cursor.0+3)/3*3;
                if self.cursor.0 >= Self::max_cursor_x() {
                    self.cursor.1 += 1;
                    self.cursor.0 = 0;
                }
            }
            '\n' => {
                self.cursor.1 += 1;
                self.cursor.0 = 0;
            },
            '\x0C' => self.cursor.1 = 0,
            '\r' => self.cursor.0 = 0,
            ch => {
                draw_char(ch, self.cursor.0, self.cursor.1,
                          RGBColor::new(255, 255, 255),
                          RGBColor::new(0, 0, 0)
                );
                self.cursor.0 += 1;

                if self.cursor.0 >= Terminal::max_cursor_x() {
                    self.cursor.0 = 0;
                    self.cursor.1 += 1;
                }
            }
        }
    }

    pub fn print_str(&mut self, string: &str) {
        for c in string.chars() {
            self.print_char(c);
        }
        flush();
    }
}

impl Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.print_str(s);
        Ok(())
    }
}

#[allow(static_mut_refs)]
#[doc(hidden)]
pub fn _print(args: Arguments) {
    unsafe { TERMINAL.write_fmt(args).unwrap(); }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::display::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}