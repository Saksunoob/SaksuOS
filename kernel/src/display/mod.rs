use core::fmt::{Arguments, Write};
use core::ptr::null_mut;
use bootloader::boot_info::{FrameBuffer, PixelFormat, RGBColor, Color};
use crate::display::font::{get_char, get_font_bounds};

mod font;

static mut FRAMEBUFFER: *mut FrameBuffer = null_mut();
static mut TERMINAL: Terminal = Terminal::new();
static mut COUNT: usize = 0;
const DEBUG_DOT_SIZE: usize = 4;

fn framebuffer() -> &'static mut FrameBuffer {
    unsafe { FRAMEBUFFER.as_mut().unwrap() }
}

pub fn init(frame_buffer: &mut FrameBuffer) {
    unsafe {
        FRAMEBUFFER = frame_buffer;
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

    unsafe { COUNT += 1 }
}

fn rect(color: [u8; 4], x: usize, y: usize, width: usize, height: usize) {
    for y in y..y+height {
        for x in x..x+width {
            framebuffer().buffer[y*framebuffer().stride+x] = color
        }
    }
}

pub fn draw_char(ch: char, x: usize, y: usize, color: RGBColor) {
    let color = match framebuffer().pixel_format {
        PixelFormat::RGB => color.get_rgba(),
        PixelFormat::BGR => color.get_bgra(),
        _ => color.get_rgba(),
    };

    let bounds = get_font_bounds();
    let x = x as i64*bounds.width()+bounds.x();
    let y = y as i64*bounds.height()-bounds.y();

    let ch = get_char(ch);
    match ch {
        Some(ch) => {
            let ch_bounds = ch.bounds();
            let x_offset = (x+ch_bounds.x()) as usize;
            let y_offset = (y+ch_bounds.y()) as usize;

            for (l, line) in ch.bitmap().iter().enumerate() {
                for p in 0..ch.bounds().width() {
                    let v = (line >> (7-p)) & 1 > 0;
                    if v {
                        rect(color, x_offset + p as usize, y_offset+l, 1, 1)
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
    framebuffer().buffer.fill(color);
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
            '\n' => {
                self.cursor.1 += 1;
                self.cursor.0 = 0;
            },
            '\r' => self.cursor.0 = 0,
            ch => {
                draw_char(ch, self.cursor.0, self.cursor.1, RGBColor::new(255, 255, 255));
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