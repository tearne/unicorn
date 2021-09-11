pub mod error;
pub mod keyboard;
pub mod pimoroni;

pub static BLACK: RGB = RGB { r: 0, g: 0, b: 0 };

#[derive(Debug, Clone, Copy)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        RGB { r, g, b }
    }
}
