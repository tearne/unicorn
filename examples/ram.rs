use std::time::Duration;

use clap::Parser;
use psutil::memory::{VirtualMemory, os::linux::VirtualMemoryExt, os::unix::VirtualMemoryExt as UnixExt};
use rgb::RGB8;
use unicorn::pimoroni::{Display, unicornmini::UnicornMini, unicorn::Unicorn};
use color_eyre::Result;

static GREEN: RGB8 = RGB8::new(0,20,0);

#[derive(Debug)]
struct RamStats{
    used: f32,
    free: f32,
    active: f32,
    inactive: f32,
    buffers: f32,
    cached: f32,
    shared: f32,
    added_up: f32,
    
}
impl From<VirtualMemory> for RamStats {
    fn from(v: VirtualMemory) -> Self {
        let tot = v.total() as f32;

        let used = v.used() as f32 / tot;
        let free = v.free() as f32 / tot;
        let active = v.active() as f32 / tot;
        let inactive = v.inactive() as f32 / tot;
        let buffers = v.buffers() as f32 / tot;
        let cached = v.cached() as f32 / tot;
        let shared = v.shared() as f32 / tot;
        let added_up = active + inactive + buffers
            + cached + shared;
        

        Self { 
            used,
            free,
            active,
            inactive,
            buffers,
            cached,
            shared,
            added_up,
        }
    }
}

struct RAM {
    num_px: usize,
}
impl RAM {
    pub fn new<T: Display>(display: &T) -> Self {
        Self { num_px: display.dimensions().num_px() }
    }

    pub fn get(&self) {
        let t = psutil::memory::virtual_memory().unwrap();
        let t: RamStats = t.into();
        println!("{:?}", t);

    }
}

fn go<T: Display>(mut display: T) -> Result<()> {
    let ram = RAM::new(&display);
    
    loop {
        ram.get();

        // let px = cpu.get_px();
     
        // for (idx, cpu) in px.iter().enumerate() {
        //     if *cpu { display.set_idx(idx, &RED); } 
        //     else { display.set_idx(idx, &BLACK); };
        // }
        // display.flush();

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