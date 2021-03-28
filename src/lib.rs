use std::mem;
use std::time::Duration;
use std::str;


extern crate libusb;

#[macro_use]
extern crate bitflags;

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

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RtcTime {
    year: u16,
    month: u8,
    day: u8,
    weekday: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

#[allow(non_camel_case_types, dead_code)]
#[repr(u8)]
enum Endpoint {
    Rtc = 0xa0,
    IrPwr = 0xb0,
    SetFanSpeed = 0xb1,
    GetFanSpeed = 0xb2,
    HwType = 0xc1,
    Serial = 0xd0,
    Health = 0xd2,
    FirmwareVersionLower = 0xd3,
    FirmwareVersionHigher = 0xd4,
    SafetyModel = 0xdc,
    UnsafeMode = 0xdf,
    Loopback = 0xe5,
}

#[allow(non_camel_case_types, dead_code)]
#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SafetyModel {
    Silent = 0,
    HondaNidec = 1,
    Toyota = 2,
    Elm327 = 3,
    GM = 4,
    HondaBoschGiraffe = 5,
    Ford = 6,
    Hyundai = 8, 
    Chrysler = 9, 
    Tesla = 10,
    Subaru = 11,
    Mazda = 13,
    Nissan = 14,
    VolkswagenMQB = 15,
    AllOutput = 17,
    GMAscm = 18,
    NoOutput = 19,
    HondaBoschHarness = 20,
    VolkswagenPq = 21,
    SubaryLegacy = 22,
    HyundaiLegacy = 23,
    HyundaiCommunity = 24,
}

#[allow(non_camel_case_types, dead_code)]
#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum HwType {
  Unkwown = 0x0,
  WhitePanda = 0x1,
  GreyPanda = 0x2,
  BlackPanda = 0x3,
  Pedal = 0x4,
  Uno = 0x5,
  Dos = 0x6,
}

bitflags! {
    pub struct UnsafeMode: u8 {
        const DISABLE_DISENGAGE_ON_GAS = 0x1;
        const DISABLE_STOCK_AEB = 0x2;
        const RAISE_LONGITUDINAL_LIMITS_TO_MAX = 0x8;
    }
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
        self.device.read_control(tp, Endpoint::Health as u8, 0, 0, &mut buf, self.timeout)?;

        Ok(unsafe { std::mem::transmute(buf) })
    }

    pub fn set_safety_model(&self, safety_model: SafetyModel, safety_param: u16) -> Result<(), libusb::Error> {
        self.usb_write(Endpoint::SafetyModel, safety_model as u16, safety_param)
    }

    pub fn set_unsafe_mode(&self, unsafe_mode: UnsafeMode) -> Result<(), libusb::Error> {
        self.usb_write(Endpoint::UnsafeMode, unsafe_mode.bits as u16, 0)
    }

    pub fn set_fan_speed(&self, fan_speed: u16) -> Result<(), libusb::Error> {
        self.usb_write(Endpoint::SetFanSpeed, fan_speed, 0)
    }

    pub fn set_ir_pwr(&self, ir_pwr: u16) -> Result<(), libusb::Error> {
        self.usb_write(Endpoint::IrPwr, ir_pwr, 0)
    }

    pub fn set_loopback(&self, loopback: bool) -> Result<(), libusb::Error> {
        self.usb_write(Endpoint::Loopback, loopback as u16, 0)
    }

    pub fn get_fan_speed(&self) -> Result<u16, libusb::Error> {
        self.usb_read_u16(Endpoint::GetFanSpeed, 0, 0)
    }

    pub fn get_fw_version(&self) -> Result<[u8; 128], libusb::Error> {
        let tp = libusb::request_type(libusb::Direction::In, libusb::RequestType::Vendor, libusb::Recipient::Device);
        let mut buf : [u8; 128] = [0; 128];

        self.device.read_control(tp, Endpoint::FirmwareVersionLower as u8, 0, 0, &mut buf[0..64], self.timeout)?;
        self.device.read_control(tp, Endpoint::FirmwareVersionHigher as u8, 0, 0, &mut buf[65..128], self.timeout)?;
        Ok(buf)
    }

    pub fn get_serial(&self) -> Result<String, libusb::Error> {
        let tp = libusb::request_type(libusb::Direction::In, libusb::RequestType::Vendor, libusb::Recipient::Device);
        let mut buf : [u8; 16] = [0; 16];

        self.device.read_control(tp, Endpoint::Serial as u8, 0, 0, &mut buf, self.timeout)?;
        Ok(String::from_utf8(buf.to_vec()).unwrap())
    }

    pub fn get_hw_type(&self) -> Result<HwType, libusb::Error> {
        const N : usize = mem::size_of::<HwType>();

        let mut buf : [u8; N] = [0; N];
        let tp = libusb::request_type(libusb::Direction::In, libusb::RequestType::Vendor, libusb::Recipient::Device);
        self.device.read_control(tp, Endpoint::HwType as u8, 0, 0, &mut buf, self.timeout)?;

        Ok(unsafe { std::mem::transmute(buf) })
    }

    pub fn get_rtc(&self) -> Result<RtcTime, libusb::Error> {
        const N : usize = mem::size_of::<RtcTime>();

        let mut buf : [u8; N] = [0; N];
        let tp = libusb::request_type(libusb::Direction::In, libusb::RequestType::Vendor, libusb::Recipient::Device);
        self.device.read_control(tp, Endpoint::Rtc as u8, 0, 0, &mut buf, self.timeout)?;

        Ok(unsafe { std::mem::transmute(buf) })
    }

    fn usb_write(&self, request : Endpoint, value: u16, index : u16) -> Result<(), libusb::Error> {
        let tp = libusb::request_type(libusb::Direction::Out, libusb::RequestType::Vendor, libusb::Recipient::Device);
        self.device.write_control(tp, request as u8, value, index, &mut [], self.timeout)?;
        Ok(())
    }

    fn usb_read_u16(&self, request : Endpoint, value: u16, index : u16) -> Result<u16, libusb::Error> {
        let mut buf : [u8; 2] = [0; 2];
        let tp = libusb::request_type(libusb::Direction::Out, libusb::RequestType::Vendor, libusb::Recipient::Device);
        self.device.write_control(tp, request as u8, value, index, &mut buf, self.timeout)?;
        Ok(unsafe { std::mem::transmute(buf) })
    }

}
