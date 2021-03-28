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
    pub uptime: u32,
    pub voltage: u32,
    pub current: u32,
    pub can_rx_errs: u32,
    pub can_send_errs: u32,
    pub can_fwd_errs: u32,
    pub gmlan_send_errs: u32,
    pub faults: u32,
    pub ignition_line: u8,
    pub ignition_can: u8,
    pub controls_allowed: u8,
    pub gas_interceptor_detected: u8,
    pub car_harness_status: u8,
    pub usb_power_mode: u8,
    pub safety_model: SafetyModel,
    pub safety_param: i16,
    pub fault_status: u8,
    pub power_save_enabled: u8,
}

#[allow(non_camel_case_types, dead_code)]
#[repr(u8)]
enum Endpoint {
    HEALTH = 0xd2,
    SAFETY_MODEL = 0xdc,
}

#[allow(non_camel_case_types, dead_code)]
#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SafetyModel {
    SILENT = 0,
    HONDA_NIDEC = 1,
    TOYOTA = 2,
    ELM327 = 3,
    GM = 4,
    HONDA_BOSCH_GIRAFFE = 5,
    FORD = 6,
    HYUNDAI = 8, 
    CHRYSLER = 9, 
    TESLA = 10,
    SUBARU = 11,
    MAZDA = 13,
    NISSAN = 14,
    VOLKSWAGEN_MQB = 15,
    ALLOUTPUT = 17,
    GM_ASCM = 18,
    NOOUTPUT = 19,
    HONDA_BOSCH_HARNESS = 20,
    VOLKSWAGEN_PQ = 21,
    SUBARU_LEGACY = 22,
    HYUNDAI_LEGACY = 23,
    HYUNDAI_COMMUNITY = 24,
}


impl<'a> Panda<'a>  {
    pub fn new(context: &'a libusb::Context, timeout: Duration) -> Panda<'a>  {
        let device = context.open_device_with_vid_pid(0xbbaa, 0xddcc).unwrap();

        Panda {
            device,
            timeout
        }
    }

    pub fn health(&self) -> Result<Health, libusb::Error> {
        const N : usize = mem::size_of::<Health>();

        let mut buf : [u8; N] = [0; N];
        let tp = libusb::request_type(libusb::Direction::In, libusb::RequestType::Vendor, libusb::Recipient::Device);
        self.device.read_control(tp, Endpoint::HEALTH as u8, 0, 0, &mut buf, self.timeout)?;

        Ok(unsafe { std::mem::transmute(buf) })
    }

    pub fn set_safety_model(&self, safety_model: SafetyModel, safety_param: u16) -> Result<(), libusb::Error> {
        self.usb_write(Endpoint::SAFETY_MODEL, safety_model as u16, safety_param)
    }

    fn usb_write(&self, request : Endpoint, value: u16, index : u16) -> Result<(), libusb::Error> {
        let tp = libusb::request_type(libusb::Direction::Out, libusb::RequestType::Vendor, libusb::Recipient::Device);
        self.device.write_control(tp, request as u8, value, index, &mut [], self.timeout)?;
        Ok(())
    }
}