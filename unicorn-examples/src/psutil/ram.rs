use psutil::memory::{os::linux::VirtualMemoryExt, VirtualMemory};
use rgb::RGB8;
use unicorn::pimoroni::Display;

static RED: RGB8 = RGB8::new(100, 0, 0);
static GREEN: RGB8 = RGB8::new(0, 100, 0);
static BLUE: RGB8 = RGB8::new(0, 0, 100);
static BLACK: RGB8 = RGB8::new(0, 0, 0);

pub fn percentage_used() -> f32 {
    let ram = psutil::memory::virtual_memory().unwrap();
    let percentage = (ram.used() + ram.shared()) as f32 / ram.total() as f32;
    percentage.min(1.0)
}

pub fn colour_array(num_px: usize) -> Vec<RGB8> {
    let ram = psutil::memory::virtual_memory().unwrap();

    let used = ram.used() + ram.shared();
    let buffers = ram.buffers();
    let cached = ram.cached();

    let total = ram.total() as f32;

    let used_percent = used as f32 / total;
    let buffers_percent = buffers as f32 / total;
    let cached_percent = cached as f32 / total;
    let result: Vec<RGB8> = (0..num_px)
        .map(|idx| {
            let px_percent_limit = (1.0 + idx as f32) / num_px as f32;
            if px_percent_limit < used_percent {
                GREEN
            } else if px_percent_limit < used_percent + buffers_percent {
                BLUE
            } else {
                BLACK
            }
        })
        .collect();

    log::info!(
        "Used {} ({:.3}%), buffers {} ({:.3}%), cached {} ({:.3}%)",
        used,
        used_percent,
        buffers,
        buffers_percent,
        cached,
        cached_percent
    );

    result
}
