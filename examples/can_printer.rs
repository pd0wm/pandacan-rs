extern crate libusb;
extern crate termion;


use std::time::Duration;
use std::thread;
use panda::{Panda, SafetyModel};
use std::collections::BTreeMap;

use binascii::bin2hex;
use std::str;


fn main() {
    let context = libusb::Context::new().unwrap();
    let panda = Panda::new(&context, Duration::from_millis(100));
    let fw = panda.get_fw_version().expect("Error getting fw version");
    println!("FW version: {:x?}", fw);

    let serial = panda.get_serial().expect("Error getting serial");
    println!("Serial: {:}", serial);

    panda.set_safety_model(SafetyModel::AllOutput, 0).expect("Error setting safety mode");
  
    let mut msgs = BTreeMap::new();
    let mut counts = BTreeMap::new();

    loop {
        if let Ok(c) = panda.can_receive() {
            for msg in c {
                if msg.src != 0 {
                    continue;
                }
                msgs.insert(msg.address, msg);
                let mut cnt = counts.entry(msg.address).or_insert(0);
                *cnt = *cnt + 1;
            }
        }

        println!("{}", termion::clear::All);
        for (addr, msg) in msgs.iter() {
            let mut dat = [0u8; 16];
            bin2hex(&msg.dat, &mut dat).unwrap();

            let cnt = counts.get(addr).unwrap();
            println!("{:04X}({:>4})({:>6}) {:}", msg.address, msg.address, cnt, str::from_utf8(&dat[0..msg.len * 2]).unwrap());
        }
        thread::sleep(Duration::from_millis(100));
    }
}
