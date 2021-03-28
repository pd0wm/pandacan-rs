extern crate libusb;

use std::time::Duration;

use panda::{Panda, SafetyModel};

#[test]
fn safety_model() {
    let context = libusb::Context::new().unwrap();
    let panda = Panda::new(&context, Duration::from_millis(100));

    panda.set_safety_model(SafetyModel::SILENT, 0).expect("Error setting safety mode");
    let health = panda.health().expect("Error getting health");
    assert_eq!(SafetyModel::SILENT, health.safety_model);
}