use std::time::Duration;

use clap::Parser;
use color_eyre::Result;
use rgb::RGB8;
use unicorn::pimoroni::{Display, unicornmini::UnicornMini, unicorn::Unicorn};
use unicorn_examples::psutil::cpu::Cpu;

static RED: RGB8 = RGB8::new(100,0,0);
static BLACK: RGB8 = RGB8::new(0,0,0);

fn go<T: Display>(mut display: T) -> Result<()> {
    let mut cpu = Cpu::new(&display);
    
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