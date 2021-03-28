use std::mem;
use std::time::Duration;

extern crate libusb;

pub struct Panda<'a> {
    device : libusb::DeviceHandle<'a>,
    timeout : Duration,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Health {
    uptime: u32,
    voltage: u32,
    current: u32,
    can_rx_errs: u32,
    can_send_errs: u32,
    can_fwd_errs: u32,
    gmlan_send_errs: u32,
    faults: u32,
    ignition_line: u8,
    ignition_can: u8,
    controls_allowed: u8,
    gas_interceptor_detected: u8,
    car_harness_status: u8,
    usb_power_mode: u8,
    safety_model: u8,
    safety_param: i16,
    fault_status: u8,
    power_save_enabled: u8,
}

#[allow(non_camel_case_types, dead_code)]
#[repr(u8)]
enum Endpoints {
    HEALTH = 0xd2,
}


impl<'a> Panda<'a>  {
    pub fn new(context: &'a libusb::Context) -> Panda<'a>  {
        let device = context.open_device_with_vid_pid(0xbbaa, 0xddcc).unwrap();
        let timeout = Duration::from_millis(100);

        Panda {
            device,
            timeout,
        }
    }

    pub fn health(&self) -> Result<Health, libusb::Error> {
        const N : usize = mem::size_of::<Health>();

        let mut buf : [u8; N] = [0; N];
        let tp = libusb::request_type(libusb::Direction::In, libusb::RequestType::Vendor, libusb::Recipient::Device);
        self.device.read_control(tp, Endpoints::HEALTH as u8, 0, 0, &mut buf, self.timeout)?;

        Ok(unsafe { std::mem::transmute(buf) })
    }
}