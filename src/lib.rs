pub mod error;
pub mod keyboard;
pub mod pimoroni;

#[derive(Debug, Clone, Copy)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl RGB {
    pub const BLACK: RGB = RGB { r: 0, g: 0, b: 0 };

    pub fn new(r: u8, g: u8, b: u8) -> Self {
        RGB { r, g, b }
    }
}
