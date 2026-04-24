use macros::bdf_font;
const FONT: Font = bdf_font!("src/display/bdf/ib8x8u.bdf");

#[derive(Copy, Clone, Debug)]
pub struct BoundingBox(i64, i64, i64, i64);

impl BoundingBox {
    pub fn width(&self) -> i64 {self.0}
    pub fn height(&self) -> i64 {self.1}
    pub fn x(&self) -> i64 {self.2}
    pub fn y(&self) -> i64 {self.3}
}

pub struct Char(u32, BoundingBox, &'static [u32]);

impl Char {
    pub fn char(&self) -> char {char::from_u32(self.0).unwrap()}
    pub fn bounds(&self) -> BoundingBox {self.1}
    pub fn bitmap(&self) -> &'static [u32] {self.2}
}

struct Font(BoundingBox, &'static [Char]);

pub fn get_font_bounds() -> BoundingBox { FONT.0 }
pub fn get_char(ch: char) -> Option<&'static Char> {
    let chars = FONT.1;
    chars.iter().find(|c| c.char() == ch)
}