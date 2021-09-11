use rppal::gpio::{Gpio, InputPin, Trigger};
use spidev::{SpiModeFlags, Spidev, SpidevOptions};
use std::{
    io::Write,
    ops::Range,
    time::{Duration, SystemTime},
};
use tokio::{
    sync::watch::{channel, Receiver},
    task::JoinHandle,
};

use crate::RGB;

// Based on:
// https://github.com/pimoroni/unicornhatmini-python/blob/master/library/unicornhatmini/__init__.py

// Useful:
// https://pinout.xyz/pinout/unicorn_hat_mini#
// https://stackoverflow.com/questions/62376239/rust-tokio-infinite-loop-for-multiple-buttons-listening

// SPI stuff
// https://raspberrypi.stackexchange.com/questions/121713/what-should-i-look-for-to-find-the-proper-gpio-chip-on-the-system
// https://github.com/dotnet/iot/blob/main/Documentation/raspi-spi.md
// https://github.com/raspberrypi/firmware/blob/7b99da75f55a5ad7d572ec4ebe4e8f9573deaee7/boot/overlays/README#L2437

// Holtek HT16D35 to drive the LEDs
const CMD_SOFT_RESET: [u8; 1] = [0xCC];
const CMD_GLOBAL_BRIGHTNESS: [u8; 2] = [0x37, 0x01];
const CMD_COM_PIN_CTRL: [u8; 2] = [0x41, 0xff];
const CMD_ROW_PIN_CTRL: [u8; 5] = [0x42, 0xff, 0xff, 0xff, 0xff];
const CMD_WRITE_DISPLAY: [u8; 2] = [0x80, 0x00];
// const CMD_READ_DISPLAY:[u8;1] = [0x81];
const CMD_SYSTEM_CTRL_OFF: [u8; 2] = [0x35, 0x00];
const CMD_SYSTEM_CTRL_ON: [u8; 2] = [0x35, 0x03];
const CMD_SCROLL_CTRL: [u8; 2] = [0x20, 0x00];

const LUT: [[usize; 3]; 119] = [
    [139, 138, 137],
    [223, 222, 221],
    [167, 166, 165],
    [195, 194, 193],
    [111, 110, 109],
    [55, 54, 53],
    [83, 82, 81],
    [136, 135, 134],
    [220, 219, 218],
    [164, 163, 162],
    [192, 191, 190],
    [108, 107, 106],
    [52, 51, 50],
    [80, 79, 78],
    [113, 115, 114],
    [197, 199, 198],
    [141, 143, 142],
    [169, 171, 170],
    [85, 87, 86],
    [29, 31, 30],
    [57, 59, 58],
    [116, 118, 117],
    [200, 202, 201],
    [144, 146, 145],
    [172, 174, 173],
    [88, 90, 89],
    [32, 34, 33],
    [60, 62, 61],
    [119, 121, 120],
    [203, 205, 204],
    [147, 149, 148],
    [175, 177, 176],
    [91, 93, 92],
    [35, 37, 36],
    [63, 65, 64],
    [122, 124, 123],
    [206, 208, 207],
    [150, 152, 151],
    [178, 180, 179],
    [94, 96, 95],
    [38, 40, 39],
    [66, 68, 67],
    [125, 127, 126],
    [209, 211, 210],
    [153, 155, 154],
    [181, 183, 182],
    [97, 99, 98],
    [41, 43, 42],
    [69, 71, 70],
    [128, 130, 129],
    [212, 214, 213],
    [156, 158, 157],
    [184, 186, 185],
    [100, 102, 101],
    [44, 46, 45],
    [72, 74, 73],
    [131, 133, 132],
    [215, 217, 216],
    [159, 161, 160],
    [187, 189, 188],
    [103, 105, 104],
    [47, 49, 48],
    [75, 77, 76],
    [363, 362, 361],
    [447, 446, 445],
    [391, 390, 389],
    [419, 418, 417],
    [335, 334, 333],
    [279, 278, 277],
    [307, 306, 305],
    [360, 359, 358],
    [444, 443, 442],
    [388, 387, 386],
    [416, 415, 414],
    [332, 331, 330],
    [276, 275, 274],
    [304, 303, 302],
    [337, 339, 338],
    [421, 423, 422],
    [365, 367, 366],
    [393, 395, 394],
    [309, 311, 310],
    [253, 255, 254],
    [281, 283, 282],
    [340, 342, 341],
    [424, 426, 425],
    [368, 370, 369],
    [396, 398, 397],
    [312, 314, 313],
    [256, 258, 257],
    [284, 286, 285],
    [343, 345, 344],
    [427, 429, 428],
    [371, 373, 372],
    [399, 401, 400],
    [315, 317, 316],
    [259, 261, 260],
    [287, 289, 288],
    [346, 348, 347],
    [430, 432, 431],
    [374, 376, 375],
    [402, 404, 403],
    [318, 320, 319],
    [262, 264, 263],
    [290, 292, 291],
    [349, 351, 350],
    [433, 435, 434],
    [377, 379, 378],
    [405, 407, 406],
    [321, 323, 322],
    [265, 267, 266],
    [293, 295, 294],
    [352, 354, 353],
    [436, 438, 437],
    [380, 382, 381],
    [408, 410, 409],
    [324, 326, 325],
    [268, 270, 269],
    [296, 298, 297],
];
const BUF_SIZE: usize = 28 * 8;
pub const NUM_LEDS: usize = 119;

#[derive(Debug)]
pub enum Button {
    A,
    B,
    X,
    Y,
}
impl Button {
    pub fn pin(&self) -> u8 {
        match self {
            Button::A => 5,
            Button::B => 6,
            Button::X => 16,
            Button::Y => 24,
        }
    }
}

pub struct UnicornMini {
    data_buf: [u8; BUF_SIZE * 2],
    spi: [Spidev; 2],
    button_rx: Receiver<Option<Button>>,
    pub button_join_handle: JoinHandle<()>,
}
impl UnicornMini {
    pub fn new() -> Self {
        let mut gpio = Gpio::new().unwrap();

        fn get_pin(gpio: &mut Gpio, id: u8) -> InputPin {
            let mut pin = gpio.get(id).unwrap().into_input_pullup();
            pin.set_interrupt(Trigger::Both).unwrap();
            pin
        }

        let pins = [
            get_pin(&mut gpio, Button::A.pin()),
            get_pin(&mut gpio, Button::B.pin()),
            get_pin(&mut gpio, Button::X.pin()),
            get_pin(&mut gpio, Button::Y.pin()),
        ];

        let (tx, button_rx) = channel(Option::<Button>::None);

        let button_join_handle: JoinHandle<()> = tokio::task::spawn_blocking(move || {
            let p: [&InputPin; 4] = [&pins[0], &pins[1], &pins[2], &pins[3]];

            let mut prev_time = SystemTime::now();

            loop {
                let result = gpio.poll_interrupts(&p, true, None);

                let elapsed = prev_time.elapsed().unwrap_or_default();

                if elapsed > Duration::from_millis(500) {
                    prev_time = SystemTime::now();
                    let result = result.unwrap();
                    let (pressed_pin, _) = result.as_ref().unwrap();

                    if *pressed_pin == p[0] {
                        tx.send(Button::A.into()).unwrap();
                    } else if *pressed_pin == p[1] {
                        tx.send(Button::B.into()).unwrap();
                    } else if *pressed_pin == p[2] {
                        tx.send(Button::X.into()).unwrap();
                    } else if *pressed_pin == p[3] {
                        tx.send(Button::Y.into()).unwrap();
                    }
                }
            }
        });

        fn get_spi(address: &str) -> Spidev {
            let mut spi = Spidev::open(address).unwrap();
            let options = SpidevOptions::new()
                .max_speed_hz(600_000)
                .bits_per_word(8)
                .mode(SpiModeFlags::SPI_MODE_0)
                .build();
            spi.configure(&options).expect("SPI config error");
            spi
        }

        let mut um = Self {
            data_buf: [0; BUF_SIZE * 2],
            spi: [get_spi("/dev/spidev0.0"), get_spi("/dev/spidev0.1")],
            button_rx,
            button_join_handle,
        };

        um.reset();

        um
    }

    pub fn reset(&mut self) {
        self.write_prefix(&CMD_SOFT_RESET, Some(&[]));
        self.write_prefix(&CMD_GLOBAL_BRIGHTNESS, Some(&[]));
        self.write_prefix(&CMD_SCROLL_CTRL, Some(&[]));
        self.write_prefix(&CMD_SYSTEM_CTRL_OFF, Some(&[]));
        self.write_prefix(&CMD_WRITE_DISPLAY, None); //TODO without clone
        self.write_prefix(&CMD_COM_PIN_CTRL, Some(&[]));
        self.write_prefix(&CMD_ROW_PIN_CTRL, Some(&[]));
        self.write_prefix(&CMD_SYSTEM_CTRL_ON, Some(&[]));
    }

    pub fn button_subscribe(&self) -> Receiver<Option<Button>> {
        self.button_rx.clone()
    }

    pub fn set_xy(&mut self, x: usize, y: usize, rgb: &RGB) {
        let idx = x * 7 + y;
        assert!(x < 17, "LED x index out of range: {}", idx);
        assert!(y < 7, "LED y index out of range: {}", idx);

        self.set_idx(idx, rgb);
    }

    pub fn set_idx(&mut self, idx: usize, rgb: &RGB) {
        assert!(idx < 119, "LED index out of range: {}", idx);
        let [ir, ig, ib] = LUT[idx];
        self.data_buf[ir] = rgb.r;
        self.data_buf[ig] = rgb.g;
        self.data_buf[ib] = rgb.b;
    }
    pub fn flush(&mut self) {
        self.write(None);
    }

    fn buf_offset(buffer_idx: usize) -> Range<usize> {
        buffer_idx * BUF_SIZE..(buffer_idx + 1) * BUF_SIZE
    }

    fn write(&mut self, data: Option<&[u8]>) {
        self.write_prefix(&CMD_WRITE_DISPLAY, data)
    }
    fn write_prefix(&mut self, prefix: &[u8], data: Option<&[u8]>) {
        fn concat(a: &[u8], b: &[u8]) -> Vec<u8> {
            let mut d = a.to_owned();
            d.extend(b);
            d
        }

        let data = data.unwrap_or(&self.data_buf);

        // Send data to both chips
        for i in 0..2 {
            let spi = &mut self.spi[i];
            if data.len() > 0 {
                let chunk = &data[Self::buf_offset(i)];
                spi.write(&concat(&prefix, &chunk))
                    .expect("SPI write error");
            } else {
                spi.write(&prefix).expect("SPI write error");
            }
        }
    }
}

impl Drop for UnicornMini {
    fn drop(&mut self) {
        self.reset();
    }
}
