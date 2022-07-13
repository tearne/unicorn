mod cpu;
mod pixel;
mod ram;

use std::time::Duration;

use clap::{Parser, Subcommand};
use color_eyre::Result;
use cpu::Cpu;
use pixel::PixelGrid;
use rgb::RGB8;
use unicorn::pimoroni::{unicorn::Unicorn, unicornmini::UnicornMini, Display};

static GREEN: RGB8 = RGB8::new(0, 20, 0);
static RED: RGB8 = RGB8::new(70, 00, 0);
static BLACK: RGB8 = RGB8::new(0, 0, 0);

fn go<T: Display>(mut display: T) -> Result<()> {
    let mut pixels = {
        let num_dots = display.dimensions().num_px();
        PixelGrid::new(num_dots)
    };
    let mut cpu = Cpu::new(&display);

    loop {
        pixels.update_percentage(ram::percentage_used())?;
        let ram_px = pixels.get_status();
        let cpu_px = cpu.get_px();

        let ram_cpu_px = ram_px.iter().zip(cpu_px.iter());

        for (idx, (ram, cpu)) in ram_cpu_px.enumerate() {
            let mut rgb = if *ram { GREEN } else { BLACK };
            if *cpu { rgb += RED; };

            display.set_idx(idx, &rgb);
        }

        display.flush();

        std::thread::sleep(Duration::from_millis(1000));
    }
}

#[derive(Parser)]
// #[clap(author, version, about, long_about = None)]
// #[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Mode>,
}

// #[derive(Clone)]
#[derive(Subcommand)]
enum Mode {
    UnicornMini,
    Unicorn,
}

fn main() -> Result<()> {
    env_logger::init();

    match Cli::parse().command {
        Some(Mode::UnicornMini ) => go(UnicornMini::new())?,
        Some(Mode::Unicorn ) => go(Unicorn::new())?,
        None => {
            log::info!("Defaulting to Unicorn mode");
            go(Unicorn::new())?
        },
    }

    Ok(())
}
