extern crate libusb;

use std::time::Duration;
use std::thread;
use pandacan::Panda;


fn main() {
    let context = libusb::Context::new().unwrap();
    let panda = Panda::new(&context, Duration::from_millis(100));
    let fw = panda.get_fw_version().expect("Error getting fw version");
    println!("FW version: {:x?}", fw);

    let serial = panda.get_serial().expect("Error getting serial");
    println!("Serial: {:}", serial);
  

    loop {
        if let Ok(h) = panda.health() {
            println!("{:?}", h);
        }
        thread::sleep(Duration::from_millis(500));
    }
}
