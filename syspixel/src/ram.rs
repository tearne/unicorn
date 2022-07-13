use psutil::memory::os::linux::VirtualMemoryExt;

pub fn percentage_used() -> f32 {
    let ram = psutil::memory::virtual_memory().unwrap();
    let percentage = (ram.used() + ram.shared()) as f32 / ram.total() as f32;
    percentage.min(1.0)
}
