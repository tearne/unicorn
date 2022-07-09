use std::{process::Command, time::Duration};

use color_eyre::{Result, eyre::Context};
use rgb::RGB8;
use unicorn::pimoroni::{unicornmini::UnicornMini, Display, unicorn::Unicorn};
use clap::Parser;

static GREEN: RGB8 = RGB8::new(0,20,0);
static BLUE: RGB8 = RGB8::new(0,0,20);
static BLACK: RGB8 = RGB8::new(0,0,0);

#[derive(Debug)]
enum State{
    Live{
        percentage: f32,
        voltage: f32
    
    },
    Standby{
        percentage: f32,
        voltage: f32
    },
}
impl State {
    pub fn percentage(&self) -> f32 {
        match self {
            State::Live { percentage , voltage: _} => *percentage,
            State::Standby { percentage, voltage: _} => *percentage,
        }
    }

    pub fn colour(&self) -> RGB8 {
        match self {
            State::Live { percentage: _ , voltage: _ } => GREEN,
            State::Standby { percentage: _, voltage: _ } => BLUE,
        }
    }
}
struct Battery{
    full: f32,
    critical: f32,
    live_denom: f32,
    standby_denom: f32,
    num_px: usize,
}
impl Battery {
    pub fn new<T: Display>(display: &T) -> Self {
        let num_px = {
            let dims = display.dimensions();
            dims.width * dims.height
        };

        Self { 
            full: 3.2, 
            critical: 2.95,
            live_denom: 3.2 - 2.95,
            standby_denom: 3.6 - 3.2,
            num_px
        }
    }

    fn current_state(&self) -> Result<State> {
        let voltage = Self::get_vbat()?;
        if voltage > self.full {
            let percentage = (voltage - self.full) / self.standby_denom;
            let percentage = percentage.min(1.0);
            Ok(State::Standby{percentage, voltage})
        } else {
            let percentage = (voltage - self.critical) / self.live_denom;
            Ok(State::Live{percentage, voltage})
        }
    }

    fn get_vbat() -> Result<f32> {
        let out = Command::new("lifepo4wered-cli")
            .args(["get", "vbat"])
            .output()
            .unwrap();
        let mut s = String::from_utf8(out.stdout).unwrap();
        s.truncate(s.len() - 1);
        s.parse::<f32>()
            .wrap_err("Failed to parse")
            .map(|v| v / 1000.0)
    }

    pub fn get_colours(&self) -> Result<Vec<RGB8>> {
        let mut pixels: Vec<RGB8> = Vec::with_capacity(self.num_px);

        let state = self.current_state()?;
        log::info!("{:?}", state);

        let dots = (self.num_px as f32 * state.percentage()).ceil() as usize;
        
        for _ in 0..dots {
            pixels.push(state.colour());  
        }
        for _ in dots..self.num_px {
            pixels.push(BLACK);
        }

        Ok(pixels)
    }
}


fn go<T: Display>(mut display: T) -> Result<()> {
    let battery: Battery = Battery::new(&display);
    
    loop {
        let px = battery.get_colours()?;
     
        for (idx, rgb) in px.iter().enumerate() {            
            display.set_idx(idx, rgb);
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