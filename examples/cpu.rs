use std::{ops::Range, time::Duration};

use clap::{clap_derive::ArgEnum, Parser};
use color_eyre::Result;
use log::info;
use psutil::cpu::CpuTimesPercentCollector;
use rand::Rng;
use rgb::RGB8;
use unicorn::pimoroni::{Display, unicornmini::UnicornMini, unicorn::Unicorn};

static RED: RGB8 = RGB8::new(150,0,0);
static BLACK: RGB8 = RGB8::new(0,0,0);

struct CPU {
    quarters: Vec<Vec<usize>>,
    num_px: usize,
    collector: CpuTimesPercentCollector,
}
impl CPU {
    pub fn new<T: Display>(display: &T) -> Self {
        fn rect_ids(xs: Range<usize>, yx: Range<usize>) -> Vec<usize> {
            let mut res = Vec::with_capacity(256);
            for x in xs {
                for y in yx.clone() {
                    res.push(y * 16 + x);
                }
            }
            res
        }

        let dims = *display.dimensions();
        let half_width = dims.width / 2;
        let half_height = dims.height / 2;

        let quarters = vec![
            rect_ids(
                0..half_width, 
                0..half_height
            ),
            rect_ids(
                half_width..dims.width, 
                0..half_height
            ),
            rect_ids(
                0..half_width, 
                half_height..dims.height
            ),
            rect_ids(
                half_width..dims.width, 
                half_height..dims.height
            ),
        ];

        let mut collector = psutil::cpu::CpuTimesPercentCollector::new().unwrap();
        // let mut rng = rand::thread_rng();
        let _ = collector.cpu_times_percent_percpu();
        
        let num_px = dims.width * dims.height;

        Self { quarters, num_px, collector }
    }

    pub fn get_px(&mut self) -> Vec<bool> {
        let mut pixels: Vec<bool> = vec![false; self.num_px];

        let loads: Vec<f64> = self.collector
            .cpu_times_percent_percpu()
            .unwrap()
            .iter()
            .map(|cpu|{
                ((100.0 - cpu.idle()) / 100.0) as f64
            })
            .collect();

        let mut rng = rand::thread_rng();
        for (cpu_id, quarter) in self.quarters.iter().enumerate() {
            for px in quarter {
                let load = loads[cpu_id];
                pixels[*px] = rng.gen_bool(load as f64);
            }
        };

        pixels
    }
}

fn go<T: Display>(mut display: T) -> Result<()> {
    let mut cpu = CPU::new(&display);
    
    loop {
     let px = cpu.get_px();
     
        for (idx, cpu) in px.iter().enumerate() {
            if *cpu { display.set_idx(idx, &RED); } 
            else { display.set_idx(idx, &BLACK); };
        }
        display.flush();

        std::thread::sleep(Duration::from_millis(1000));
    }
}

#[derive(Parser, Clone)]
enum Mode {
    UnicornMini, Unicorn
}

fn main() -> Result<()>{
    env_logger::init();

    match Mode::parse() {
        Mode::UnicornMini => go(UnicornMini::new())?,
        Mode::Unicorn => go(Unicorn::new())?,
    };

    Ok(())
}