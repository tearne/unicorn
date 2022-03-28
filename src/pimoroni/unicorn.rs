use std::{io::Write, time::Duration};

use rgb::RGB8;
use spidev::{SpiModeFlags, Spidev, SpidevOptions};

// Based on: https://github.com/pimoroni/unicorn-hat-hd/blob/master/library/unicornhathd/__init__.py

const SOF: u8 = 0x72;
const BUF_SIZE: usize = 256 * 3 + 1;
const DELAY: u64 = 9;
pub struct Unicorn {
    spi: Spidev,
    buffer: [u8; BUF_SIZE],
}

impl Unicorn {
    pub fn new() -> Self {
        let mut spi = Spidev::open("/dev/spidev0.0")
            .expect("Do you have sufficient permissions to '/dev/spidev0.0' ?");
        let options = SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(9_000_000)
            .mode(SpiModeFlags::SPI_MODE_0)
            .build();
        spi.configure(&options).expect("SPI config error");

        let mut display = Unicorn {
            spi,
            buffer: [0; BUF_SIZE],
        };
        display.reset();
        display
    }

    pub fn set_xy(&mut self, x: usize, y: usize, rgb: &RGB8) {
        let idx = x + y * 16;
        assert!(x < 16, "LED x index out of range: {}", idx);
        assert!(y < 16, "LED y index out of range: {}", idx);

        self.set_idx(idx, rgb);
    }

    pub fn set_idx(&mut self, idx: usize, rgb: &RGB8) {
        // Buffer indexes are offset by 1 because of 0x72 at start
        let i = idx * 3 + 1;
        self.buffer[i] = rgb.r;
        self.buffer[i + 1] = rgb.g;
        self.buffer[i + 2] = rgb.b;
    }

    pub fn flush(&mut self) {
        self.spi.write_all(&self.buffer).expect("SPI write error");
        std::thread::sleep(Duration::from_millis(DELAY));
    }

    pub fn reset(&mut self) {
        self.buffer = [0; BUF_SIZE];
        self.buffer[0] = SOF;
        self.flush();
    }
}

impl Default for Unicorn {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Unicorn {
    fn drop(&mut self) {
        self.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::{Unicorn, RGB8};
    use std::time::Duration;

    #[test]
    fn test_unicorn() {
        let mut display = Unicorn::new();
        let r = RGB8::new(255, 0, 0);
        let g = RGB8::new(0, 255, 0);
        let b = RGB8::new(0, 0, 255);

        display.set_xy(0, 0, &r);
        display.set_xy(1, 0, &r);
        display.set_xy(2, 0, &r);

        display.set_xy(3, 0, &r);
        display.set_xy(4, 0, &g);
        display.set_xy(4, 1, &b);

        display.set_xy(4, 2, &b);
        display.set_xy(4, 3, &b);
        display.set_xy(4, 4, &b);
        display.flush();
        std::thread::sleep(Duration::from_millis(10000));
    }
}
