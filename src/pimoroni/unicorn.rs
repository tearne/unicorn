use std::{io::Write, time::Duration};

use super::RGB;
use spidev::{SpiModeFlags, Spidev, SpidevOptions};

// Baed on: https://github.com/pimoroni/unicorn-hat-hd/blob/master/library/unicornhathd/__init__.py

const SOF: u8 = 0x72;
const BUF_SIZE: usize = 256 * 3 + 1;
const DELAY: u64 = 9;
pub struct Unicorn {
    spi: Spidev,
    buffer: [u8; BUF_SIZE],
}

impl Unicorn {
    pub fn new() -> Self {
        let mut spi = Spidev::open("/dev/spidev0.0").expect("Do you have sufficient permissions to /dev/spidev0.0?");
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
    pub fn set_idx(&mut self, idx: usize, rgb: &RGB) {
        // Buffer indexes are offset by 1 because of 0x72 at start
        let i = idx * 3 + 1;
        self.buffer[i] = rgb.r;
        self.buffer[i + 1] = rgb.g;
        self.buffer[i + 2] = rgb.b;
    }

    pub fn flush(&mut self) {
        self.spi.write(&self.buffer).expect("SPI write error");
        std::thread::sleep(Duration::from_millis(DELAY));
    }

    pub fn reset(&mut self) {
        self.buffer = [0; BUF_SIZE];
        self.buffer[0] = SOF;
        self.flush();
    }
}

impl Drop for Unicorn {
    fn drop(&mut self) {
        self.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::{Unicorn, RGB};
    use std::time::Duration;

    #[test]
    fn test_unicorn() {
        let mut display = Unicorn::new();
        let r = RGB::new(255, 0, 0);
        let g = RGB::new(0, 255, 0);
        let b = RGB::new(0, 0, 255);

        display.set_idx(0, &r);
        display.set_idx(1, &g);
        display.set_idx(2, &b);

        display.set_idx(118, &r);
        display.set_idx(119, &g);
        display.set_idx(120, &b);

        display.set_idx(253, &r);
        display.set_idx(254, &g);
        display.set_idx(255, &b);
        display.flush();
        std::thread::sleep(Duration::from_millis(5000));
    }
}
