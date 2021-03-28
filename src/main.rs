extern crate libusb;

use std::time::Duration;
use std::thread;
use panda::Panda;


fn main() {
    let context = libusb::Context::new().unwrap();
    let panda = Panda::new(&context);
    loop {
        if let Ok(h) = panda.health() {
            println!("{:?}", h);
        }
        thread::sleep(Duration::from_millis(500));
    }
}
