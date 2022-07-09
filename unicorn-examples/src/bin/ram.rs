use std::time::Duration;

use clap::Parser;
use rand::{prelude::ThreadRng, Rng};
use rgb::RGB8;
use unicorn::pimoroni::{Display, unicornmini::UnicornMini, unicorn::Unicorn};
use color_eyre::{Result, eyre::eyre};
use unicorn_examples::psutil::ram;

static GREEN: RGB8 = RGB8::new(0,20,0);
static BLACK: RGB8 = RGB8::new(0,0,0);

struct Pixels {
    active: Vec<usize>,
    inactive: Vec<usize>,
    num: usize,
    rng: ThreadRng,
}
impl Pixels {
    pub fn new(num: usize) -> Self {
        Pixels { 
            active: Vec::new(), 
            inactive: (0..num).collect(), 
            num,
            rng: rand::thread_rng(),
        }
    }

    fn deactivate(&mut self, n: usize)  -> Result<()> {
        if n > self.active.len() {
            return Err(eyre!("Can't deactivate {} pixels when only {} are active.", n, self.active.len()));
        }

        for _ in 0..n {
            let index = self.rng.gen_range(0..self.active.len());
            let px = self.active.remove(index);
            self.inactive.push(px);
        }

        Ok(())
    }

    fn activate(&mut self, n: usize) -> Result<()> {
        if n > self.inactive.len() {
            return Err(
                eyre!("Can't deactivate {} pixels when only {} are active.", 
                n, 
                self.active.len())
            );
        }

        for _ in 0..n {
            let index = self.rng.gen_range(0..self.inactive.len());
            let px = self.inactive.remove(index);
            self.active.push(px);
        }

        Ok(())
    }

    fn refresh(&mut self) -> Result<()> {
        if self.active.len() > 0 {
            let mut keeping = Vec::new();
            for index in 0..self.active.len() {
                if self.rng.gen_bool(0.1) {
                    let px = self.active[index];
                    self.inactive.push(px);
                } else {
                    keeping.push(self.active[index]);
                }
            }
            let num_lost = self.active.len() - keeping.len();
            self.active = keeping;
            self.activate(num_lost)
        }
        else { Ok(()) }
    }

    pub fn update(&mut self, n: isize) -> Result<()> {
        
        let change = n - self.active.len() as isize;
        println!("Asked for {}, currently {}, change {}", n, self.active.len(), change);
        
        if change == 0 {
            println!("==0");
            self.refresh();
        } else if change < 0 {
            println!("<0");
            self.deactivate((-change) as usize);
            self.refresh();
        } else {
            println!(">0");
            self.refresh();
            self.activate(change as usize);
        }

        // println!("  active {:?}", &self.active);
        // println!("inactive {:?}", &self.inactive);
        println!("now {}", self.active.len());


        Ok(())
    }

    pub fn get_pixel_status(&self) -> Vec<bool> {
        let mut result = vec![false; self.num];//(0..self.num).collect::<Vec<bool>>();

        for px in self.active.iter() {
            result[*px] = true;
        }

        result
    }
}

fn go_dots<T: Display>(mut display: T) -> Result<()> {
    let num_dots = display.dimensions().num_px();
    let mut num_live_dots = (num_dots as f32 * ram::percentage_used()) as usize;
    // let active_indexes: Vec<usize> = Vec::new();
    // let inactive_indexes: Vec<usize> = (0..num_dots).collect();

    // let mut rng = rand::thread_rng();

    let mut pixels = Pixels::new(num_dots);

    loop {
        let num = (num_dots as f32 * ram::percentage_used()) as isize;
        println!("num = {}", num);
        pixels.update(num);
        // println!("{:?}", pixels.get_pixel_status());
        for (idx, state) in pixels.get_pixel_status().iter().enumerate() {
            display.set_idx(idx, if *state {&GREEN} else {&BLACK});
        }

        display.flush();

        std::thread::sleep(Duration::from_millis(1000));
    }

    Ok(())
}

fn go_bar<T: Display>(mut display: T) -> Result<()> {
    let height = display.dimensions().height;

    loop {
        let px = ram::colour_array(height);

        for (idx, colour) in px.iter().enumerate() {
            display.set_xy(1, height - idx -1, colour);
            display.set_xy(2, height - idx -1, colour); 
            display.set_xy(3, height - idx -1, colour); 
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
        Mode::UnicornMini => go_dots(UnicornMini::new())?,
        Mode::Unicorn => go_dots(Unicorn::new())?,
    };

    Ok(())
}