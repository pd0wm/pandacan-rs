extern crate libusb;
extern crate termion;

extern crate clap;
use clap::{Arg, App};

use std::time::Duration;
use std::thread;
use pandacan::{Panda, SafetyModel};
use std::collections::BTreeMap;

use binascii::bin2hex;
use std::str;


fn main() {
    let matches = App::new("CAN Printer")
        .arg(Arg::with_name("bus")
                .short("b")
                .long("bus")
                .takes_value(true)
                .help("Bus to listen on"))
        .get_matches();

    let mut bus = 0;

    if let Some(s) = matches.value_of("bus") {
        if let Ok(n) = s.parse::<u8>() {
            bus = n;
        }
    }

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
                if msg.src != bus {
                    continue;
                }

                msgs.insert(msg.address, msg);
                let cnt = counts.entry(msg.address).or_insert(0);
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
