# panda-rs
<a href="https://crates.io/crates/pandacan" rel="noopener"><img alt="Crates.io" src="https://img.shields.io/crates/v/pandacan?style=flat-square"></a>

Rust library to communicate with a comma.ai panda.

```rust
let context = libusb::Context::new().unwrap();
let panda = Panda::new(&context, Duration::from_millis(100));
panda.set_safety_model(SafetyModel::AllOutput, 0).expect("Error setting safety mode");

loop {
    if let Ok(h) = panda.health() {
        println!("{:?}", h);
    }
    if let Ok(c) = panda.can_receive() {
        for msg in c {
            println!("{:?}", msg);
        }
    }
    thread::sleep(Duration::from_millis(500));
}
```

# Can printer
The examples folder contains a small helper binary to print all traffic on a certain bus:

`cargo run --example can_printer -- --bus=0`


