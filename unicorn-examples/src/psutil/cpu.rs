use std::ops::Range;

use psutil::cpu::CpuTimesPercentCollector;
use rand::Rng;
use unicorn::pimoroni::Display;

pub struct Cpu {
    quarters: Vec<Vec<usize>>,
    num_px: usize,
    collector: CpuTimesPercentCollector,
}
impl Cpu {
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