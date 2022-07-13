use std::time::Duration;

use color_eyre::eyre::Result;
use rand::Rng;
use rgb::RGB8;
use tokio::runtime::Runtime;
use unicorn::pimoroni::{
    unicornmini::{Button, UnicornMini},
    Display,
};

fn main() -> Result<()> {
    color_eyre::install()?;

    let _ = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { go().await })?;

    Ok(())
}

fn fill_with_random_colour(um: &mut UnicornMini, rng: &mut impl Rng) {
    let r = rng.gen();
    let g = rng.gen();
    let b = rng.gen();
    for i in 0..17usize {
        for j in 0..7usize {
            um.set_xy(i, j, &RGB8::new(r, g, b));
        }
    }
    um.flush();
}

async fn go() -> Result<()> {
    let rt = Runtime::new().unwrap();
    let mut um = UnicornMini::new();

    let mut rng = rand::thread_rng();

    let mut h = um.button_subscribe(&rt);

    loop {
        h.changed().await.unwrap();
        let b_opt = h.borrow_and_update();
        let t = b_opt.as_ref().unwrap();
        println!("==> {:?}", t);

        match *t {
            Button::A | Button::B | Button::X | Button::Y => {
                fill_with_random_colour(&mut um, &mut rng);
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}
