use std::{
    ffi::OsStr,
    fs::File,
    io::Read,
    io::{BufRead, BufReader},
    mem,
    os::unix::io::AsRawFd,
};

use inotify::{Event, EventMask, Inotify, WatchMask};
use libc::{c_int, input_event};
use nix::ioctl_write_ptr;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::error::{AppError, BoxedError};

ioctl_write_ptr!(eviocgrab, b'E', 0x90, c_int);
const SIZE_OF_INPUT_EVENT: usize = mem::size_of::<input_event>();

pub fn grab_all_keyboards() -> Receiver<input_event> {
    let (tx, rx) = tokio::sync::mpsc::channel::<input_event>(16);

    tokio::task::spawn_blocking(move || {
        let mut inotify = Inotify::init().expect("Failed to initialize inotify");
        inotify
            .add_watch("/dev/input/", WatchMask::CREATE)
            .expect("Failed to add inotify watch on /dev/input");

        for dev_name in all_keyboard_device_filenames() {
            tokio::spawn(grab_device(tx.clone(), dev_name));
        }

        let mut buffer = [0u8; 4096];
        loop {
            if let Ok(events) = inotify.read_events_blocking(&mut buffer) {
                for event in events {
                    if let Some(device) = represents_kbd(event) {
                        tokio::spawn(grab_device(tx.clone(), device.to_owned()));
                    }
                }
            }
        }
    });

    rx
}

fn represents_kbd(event: Event<&OsStr>) -> Option<&str> {
    if !event.mask.contains(EventMask::ISDIR) {
        if let Some(file_name) = event.name.and_then(|name| name.to_str()) {
            let device_files = all_keyboard_device_filenames();
            if device_files.contains(&file_name.to_string()) {
                return Some(file_name);
            }
        }
    }
    None
}

// grep -E 'Handlers|EV' /proc/bus/input/devices | grep -B1 -E "120013|100013" | grep -Eo event[0-9]+
fn all_keyboard_device_filenames() -> Vec<String> {
    let f = File::open("/proc/bus/input/devices").ok();

    f.map(|file| {
        let reader = BufReader::new(file);
        let mut filename = None;
        let mut filenames = Vec::new();

        for line in reader.lines() {
            if let Ok(line) = line {
                if line.starts_with("H: Handlers=") {
                    if let Some(event_index) = line.find("event") {
                        let last_index = line[event_index..line.len() - 1]
                            .find(" ")
                            .and_then(|i| Some(i + event_index))
                            .unwrap_or(line.len() - 1);
                        filename = Some(line[event_index..last_index].to_owned());
                    }
                } else if line.starts_with("B: EV=")
                    && (line.contains("120013") || line.contains("100013"))
                {
                    if let Some(ref filename) = filename {
                        filenames.push(filename.clone());
                    }
                }
            }
        }

        filenames
    })
    .unwrap_or_default()
}

struct InputDevice {
    device_file: File,
    buf: [u8; SIZE_OF_INPUT_EVENT],
}

impl InputDevice {
    pub fn open(device_file: &str) -> Result<Self, BoxedError> {
        let device_file = File::open(device_file)?;
        Ok(InputDevice {
            device_file: device_file,
            buf: [0u8; SIZE_OF_INPUT_EVENT],
        })
    }

    pub fn read_event(&mut self) -> Result<input_event, BoxedError> {
        let num_bytes = self.device_file.read(&mut self.buf)?;
        if num_bytes != SIZE_OF_INPUT_EVENT {
            return Err(AppError::boxed("ShortRead".into()));
        }
        let event: input_event = unsafe { mem::transmute(self.buf) };
        Ok(event)
    }

    pub fn grab(&mut self) -> Result<(), BoxedError> {
        unsafe {
            eviocgrab(self.device_file.as_raw_fd(), 1 as *const c_int)?;
        }
        Ok(())
    }

    pub fn release(&mut self) -> Result<(), BoxedError> {
        unsafe {
            eviocgrab(self.device_file.as_raw_fd(), 0 as *const c_int)?;
        }
        Ok(())
    }
}

impl Drop for InputDevice {
    fn drop(&mut self) {
        self.release().ok();
    }
}

async fn grab_device(tx: Sender<input_event>, device_file: String) {
    if let Err(e) = inner(tx, &device_file).await {
        println!("Lost device {} with error: {}", device_file, e)
    };

    async fn inner(tx: Sender<input_event>, device_file: &String) -> Result<(), BoxedError> {
        println!("Grabbing {}", device_file);

        let mut input_device = InputDevice::open(&format!("/dev/input/{}", device_file))?;
        input_device.grab()?;

        loop {
            let event = input_device.read_event()?;
            tx.send(event).await?;
        }
    }
}
