extern crate libusb;

use std::time::Duration;

use pandacan::{Panda, SafetyModel, UnsafeMode, HwType};

#[test]
fn safety_model() {
    let context = libusb::Context::new().unwrap();
    let panda = Panda::new(&context, Duration::from_millis(100));

    panda.set_safety_model(SafetyModel::Silent, 0).expect("Error setting safety mode");
    let health = panda.health().expect("Error getting health");
    assert_eq!(SafetyModel::Silent, health.safety_model);

    panda.set_safety_model(SafetyModel::Toyota, 0).expect("Error setting safety mode");
    let health = panda.health().expect("Error getting health");
    assert_eq!(SafetyModel::Toyota, health.safety_model);
}


#[test]
fn unsafe_mode() {
    let context = libusb::Context::new().unwrap();
    let panda = Panda::new(&context, Duration::from_millis(100));
    panda.set_unsafe_mode(UnsafeMode::DISABLE_DISENGAGE_ON_GAS | UnsafeMode::DISABLE_STOCK_AEB).unwrap();
}

#[test]
fn uptime() {
    let context = libusb::Context::new().unwrap();
    let panda = Panda::new(&context, Duration::from_millis(100));
    let health = panda.health().expect("Error getting health");
    assert!(health.uptime > 0);
}

#[test]
fn hw_type() {
    let context = libusb::Context::new().unwrap();
    let panda = Panda::new(&context, Duration::from_millis(100));
    let hw_type = panda.get_hw_type().expect("Error getting hw type");
    assert_eq!(HwType::WhitePanda, hw_type);
}

#[test]
fn fw_version() {
    let context = libusb::Context::new().unwrap();
    let panda = Panda::new(&context, Duration::from_millis(100));
    panda.get_fw_version().expect("Error getting fw version");
}

#[test]
fn serial() {
    let context = libusb::Context::new().unwrap();
    let panda = Panda::new(&context, Duration::from_millis(100));
    panda.get_serial().expect("Error getting serial");
}