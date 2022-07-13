use rand::{prelude::ThreadRng, Rng};
use color_eyre::{Result, eyre::eyre};

pub struct PixelGrid {
    active: Vec<usize>,
    inactive: Vec<usize>,
    num_px: usize,
    rng: ThreadRng,
}
impl PixelGrid {
    pub fn new(num: usize) -> Self {
        PixelGrid {
            active: Vec::new(),
            inactive: (0..num).collect(),
            num_px: num,
            rng: rand::thread_rng(),
        }
    }

    fn deactivate(&mut self, n: usize) -> Result<()> {
        if n > self.active.len() {
            return Err(eyre!(
                "Can't deactivate {} pixels when only {} are active.",
                n,
                self.active.len()
            ));
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
            return Err(eyre!(
                "Can't deactivate {} pixels when only {} are active.",
                n,
                self.active.len()
            ));
        }

        for _ in 0..n {
            let index = self.rng.gen_range(0..self.inactive.len());
            let px = self.inactive.remove(index);
            self.active.push(px);
        }

        Ok(())
    }

    fn refresh(&mut self) -> Result<()> {
        if !self.active.is_empty() {
            let mut keeping = Vec::new();
            for index in 0..self.active.len() {
                if self.rng.gen_bool(0.01) {
                    let px = self.active[index];
                    self.inactive.push(px);
                } else {
                    keeping.push(self.active[index]);
                }
            }
            let num_lost = self.active.len() - keeping.len();
            self.active = keeping;
            self.activate(num_lost)
        } else {
            Ok(())
        }
    }

    pub fn update_percentage(&mut self, percentage: f32) -> Result<()> {
        log::info!("RAM {:.1}%", percentage * 100.0);
        let num_px_required = (self.num_px as f32 * percentage) as usize;

        let change = num_px_required as isize - self.active.len() as isize;

        match change.cmp(&0) {
            std::cmp::Ordering::Equal => {
                self.refresh()?;
            }
            std::cmp::Ordering::Less => {
                self.deactivate(-change as usize)?;
                self.refresh()?;
            }
            std::cmp::Ordering::Greater => {
                self.refresh()?;
                self.activate(change as usize)?;
            }
        }

        Ok(())
    }

    pub fn get_status(&self) -> Vec<bool> {
        let mut result = vec![false; self.num_px]; //(0..self.num).collect::<Vec<bool>>();

        for px in self.active.iter() {
            result[*px] = true;
        }

        result
    }
}