use rgb::RGB8;

pub mod unicorn;
pub mod unicornmini;

#[derive(Copy, Clone)]
pub struct Dimensions {
    pub width: usize,
    pub height: usize,
}
impl Dimensions {
    pub fn num_px(&self) -> usize {
        self.width * self.height
    }
}

pub trait Display {
    fn set_xy(&mut self, x: usize, y: usize, rgb: &RGB8);
    fn set_idx(&mut self, idx: usize, rgb: &RGB8);
    fn flush(&mut self);
    fn reset(&mut self);
    fn dimensions(&self) -> &Dimensions;
}