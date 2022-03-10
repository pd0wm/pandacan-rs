use std::mem;
use std::time::Duration;

extern crate libusb;

#[macro_use]
extern crate bitflags;

const HEALTH_VERSION: u8 = 3;
const CAN_VERSION: u8 = 1;

pub struct Panda<'a> {
    device: libusb::DeviceHandle<'a>,
    timeout: Duration,
    packet_versions: PacketVersions,
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
    pub heartbeat_lost: u8,
    pub unsafe_mode: u16,
    pub blocked_msg_cnt: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PacketVersions {
    pub health_version: u8,
    pub can_version: u8,
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
    PacketVersions = 0xdd,
    UnsafeMode = 0xdf,
    Loopback = 0xe5,
    PowerSaving = 0xe7,
    UsbPowerMode = 0xe6,
    Heartbeat = 0xf3,
    HeartbeatDisabled = 0xf8,
    CanRead = 0x81,
    CanWrite = 0x3,
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
    HondaBosch = 20,
    VolkswagenPq = 21,
    SubaryLegacy = 22,
    HyundaiLegacy = 23,
    HyundaiCommunity = 24,
}

#[allow(non_camel_case_types, dead_code)]
#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum HwType {
  Unknown = 0x0,
  WhitePanda = 0x1,
  GreyPanda = 0x2,
  BlackPanda = 0x3,
  Pedal = 0x4,
  Uno = 0x5,
  Dos = 0x6,
}

#[allow(non_camel_case_types, dead_code)]
#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum UsbPowerMode {
    None  = 0x0,
    Client = 0x1,
    Cdp = 0x2,
    Dcp = 0x3,
}

bitflags! {
    pub struct UnsafeMode: u8 {
        const DISABLE_DISENGAGE_ON_GAS = 0x1;
        const DISABLE_STOCK_AEB = 0x2;
        const RAISE_LONGITUDINAL_LIMITS_TO_MAX = 0x8;
    }
}


#[derive(Debug, Copy, Clone)]
pub struct CanMessage {
    pub address: u32,
    pub bus_time: u16,
    pub src: u8,
    pub len : usize,
    pub dat : [u8; 8],
}



impl PacketVersions {
    pub fn new(device: &libusb::DeviceHandle, timeout: Duration) -> Self {
        const N : usize = mem::size_of::<PacketVersions>();

        let mut buf : [u8; N] = [0; N];
        let tp = libusb::request_type(libusb::Direction::In, libusb::RequestType::Vendor, libusb::Recipient::Device);
        device.read_control(tp, Endpoint::PacketVersions as u8, 0, 0, &mut buf, timeout).unwrap();

        unsafe { std::mem::transmute(buf) }
    }
}


impl<'a> Panda<'a>  {
    pub fn new(context: &'a libusb::Context, timeout: Duration) -> Self {
        let device = context.open_device_with_vid_pid(0xbbaa, 0xddcc).unwrap();
        let packet_versions = PacketVersions::new(&device, timeout);

        Self {
            device,
            timeout,
            packet_versions,
        }
    }

    pub fn health(&self) -> Result<Health, libusb::Error> {
        self.ensure_health_packet_version();

        const N : usize = mem::size_of::<Health>();

        let mut buf : [u8; N] = [0; N];
        let tp = libusb::request_type(libusb::Direction::In, libusb::RequestType::Vendor, libusb::Recipient::Device);
        self.device.read_control(tp, Endpoint::Health as u8, 0, 0, &mut buf, self.timeout)?;

        Ok(unsafe { std::mem::transmute(buf) })
    }

    pub fn set_safety_model(&self, safety_model: SafetyModel, safety_param: u16) -> Result<(), libusb::Error> {
        self.usb_write(Endpoint::SafetyModel, safety_model as u16, safety_param)
    }

    pub fn get_packet_versions(&self) -> PacketVersions {
        self.packet_versions
    }

    pub fn set_unsafe_mode(&self, unsafe_mode: UnsafeMode) -> Result<(), libusb::Error> {
        self.usb_write(Endpoint::UnsafeMode, unsafe_mode.bits as u16, 0)
    }

    pub fn set_fan_speed(&self, fan_speed: u16) -> Result<(), libusb::Error> {
        self.usb_write(Endpoint::SetFanSpeed, fan_speed, 0)
    }

    pub fn get_fan_speed(&self) -> Result<u16, libusb::Error> {
        self.usb_read_u16(Endpoint::GetFanSpeed, 0, 0)
    }

    pub fn set_ir_pwr(&self, ir_pwr: u16) -> Result<(), libusb::Error> {
        self.usb_write(Endpoint::IrPwr, ir_pwr, 0)
    }

    pub fn set_loopback(&self, loopback: bool) -> Result<(), libusb::Error> {
        self.usb_write(Endpoint::Loopback, loopback as u16, 0)
    }
    
    pub fn set_power_saving(&self, power_saving: bool) -> Result<(), libusb::Error> {
        self.usb_write(Endpoint::PowerSaving, power_saving as u16, 0)
    }

    pub fn set_usb_power_mode(&self, power_mode: UsbPowerMode) -> Result<(), libusb::Error> {
        self.usb_write(Endpoint::UsbPowerMode, power_mode as u16, 0)
    }

    pub fn send_heartbeat(&self) -> Result<(), libusb::Error> {
        self.usb_write(Endpoint::Heartbeat, 1, 0)
    }

    pub fn set_heartbeat_disabled(&self) -> Result<(), libusb::Error> {
        self.usb_write(Endpoint::HeartbeatDisabled, 0, 0)
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
    
    pub fn can_receive(&self) -> Result<Vec<CanMessage>, libusb::Error> {
        self.ensure_can_packet_version();

        const N : usize = 0x1000;
        let mut buf : [u8; N] = [0; N];
        let recv : usize = self.device.read_bulk(Endpoint::CanRead as u8, &mut buf, self.timeout)?;

        let num_msg : usize = recv / 0x10;
        let data : [u32; N / 4] = unsafe {std::mem::transmute(buf)};

        let mut r : Vec<CanMessage> = Vec::new();
        for i in 0..num_msg {
            let address;
            if data[i*4] & 4 != 0 {
                // Extended
                address = data[i*4] >> 3;
            } else {
                // Normal
                address = data[i*4] >> 21;
            }

            let bus_time = (data[i*4 + 1] >> 16) as u16;
            let src = (data[i*4 + 1] >> 4) as u8;
            let len = (data[i*4 + 1] & 0xF) as usize;

            let dat = [
                ((data[i*4 + 2] >> 0) & 0xFF) as u8,
                ((data[i*4 + 2] >> 8) & 0xFF) as u8,
                ((data[i*4 + 2] >> 16) & 0xFF) as u8,
                ((data[i*4 + 2] >> 24) & 0xFF) as u8,
                ((data[i*4 + 3] >> 0) & 0xFF) as u8,
                ((data[i*4 + 3] >> 8) & 0xFF) as u8,
                ((data[i*4 + 3] >> 16) & 0xFF) as u8,
                ((data[i*4 + 3] >> 24) & 0xFF) as u8,
            ];

            r.push(CanMessage {
                address,
                bus_time,
                dat,
                len,
                src,
            });
        }

        Ok(r)
    }

    pub fn can_send(&self, can_data : Vec<CanMessage>) -> Result<(), libusb::Error> {
        self.ensure_can_packet_version();

        let mut send: Vec<u32> = Vec::new();
        send.resize(can_data.len() * 0x10, 0);

        for i in 0..can_data.len() {
            let msg = can_data[i];
            if msg.address >= 0x800 {
                // Extended
                send[i*4] = ((msg.address as u32) << 3) | 5;
            } else {
                // Normal
                send[i*4] = ((msg.address as u32) << 21) | 1;
            }
            send[i*4+1] = (msg.len as u32) | ((msg.src as u32) << 4);
            send[i*4+2] = ((msg.dat[0] as u32) << 0) |
                          ((msg.dat[1] as u32) << 8) |
                          ((msg.dat[2] as u32) << 16) |
                          ((msg.dat[3] as u32) << 24);
            send[i*4+3] = ((msg.dat[4] as u32) << 0) |
                          ((msg.dat[5] as u32) << 8) |
                          ((msg.dat[6] as u32) << 16) |
                          ((msg.dat[7] as u32) << 24);
        }

        let dat = unsafe {std::slice::from_raw_parts(send.as_ptr() as *const u8, send.len() * 4)};
        self.device.write_bulk(Endpoint::CanWrite as u8, &dat, self.timeout)?;
        Ok(())
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

    fn ensure_health_packet_version(&self) -> () {
        if self.packet_versions.health_version > HEALTH_VERSION {
            panic!("Library outdated! Panda health packet version is {}, but library version is {}", self.packet_versions.health_version, HEALTH_VERSION);
        } else if self.packet_versions.health_version < HEALTH_VERSION {
            panic!("Panda outdated! Panda health packet version is {}, but library version is {}", self.packet_versions.health_version, HEALTH_VERSION);
        }
    }

    fn ensure_can_packet_version(&self) -> () {
        if self.packet_versions.can_version > CAN_VERSION {
            panic!("Library outdated! Panda CAN packet version is {}, but library version is {}", self.packet_versions.can_version, CAN_VERSION);
        } else if self.packet_versions.can_version < CAN_VERSION {
            panic!("Panda outdated! Panda CAN packet version is {}, but library version is {}", self.packet_versions.can_version, CAN_VERSION);
        }
    }

}
