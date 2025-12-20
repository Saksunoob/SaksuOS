use uefi_raw::protocol::console::SimpleTextOutputProtocol;

pub type UefiStr = *mut u16;

pub struct ConsoleOut {
    con_out: * mut SimpleTextOutputProtocol
}

impl ConsoleOut {
    pub fn new(con_out: *mut SimpleTextOutputProtocol) -> Self {
        Self {
            con_out
        }
    }

    pub fn print(&self, str: UefiStr) {
        unsafe {
            let _ = ((*self.con_out).output_string)(self.con_out, str);
        }
    }

    pub fn printstr(&self, str: &str) {
        for char in str.encode_utf16() {
            if char == '\n' as u16 {
                let mut buffer = [char, '\r' as u16, 0];
                self.print(buffer.as_mut_ptr());
            } else {
                let mut buffer = [char, 0];
                self.print(buffer.as_mut_ptr());
            }
        }
    }

    pub fn printhex<T>(&self, num: T) where T: Into<u128> {
        let mut num: u128 = num.into();
        if num == 0 {
            let mut buffer = [0x30, 0x0];
            self.print(buffer.as_mut_ptr());
            return
        }
        let mut zeroes = false;
        for _ in 0..128/4 {
            num = num.rotate_left(4);
            let val = (num&0xF) as u8;
            if val == 0 && !zeroes {
                continue;
            }
            zeroes = true;
            let mut char = (val+0x30) as u16;
            if val >= 10 {
                char += 7;
            }
            let mut buffer = [char, 0x0];
            self.print(buffer.as_mut_ptr());
        }
    }

    pub fn printdec<T>(&self, num: T) where T: Into<u128> {
        let num: u128 = num.into();
        if num == 0 {
            let mut buffer = [0x30, 0x0];
            self.print(buffer.as_mut_ptr());
            return
        }
        let mut zeroes = false;
        let mut prev_div = 1;
        for d in (0..39).rev() {
            let div = 10_u128.pow(d);
            let val = (num%prev_div)/div;
            prev_div = div;

            if val == 0 && !zeroes {
                continue;
            }

            zeroes = true;
            let mut buffer = [val as u16+0x30, 0x0];
            self.print(buffer.as_mut_ptr());
        }
    }

}