use crate::TcpIpError;

mod loopback;

pub(crate) trait NetDevice {
    fn typ(&self) -> NetDeviceType;
    fn mtu(&self) -> u16;
    fn flags(&self) -> u16;
    fn flags_mut(&mut self) -> &mut u16;
    fn hlen(&self) -> u16;
    fn addr(&self) -> &[u8];
    fn bloadcast(&self) -> &[u8];

    fn is_open(&self) -> bool {
        (self.flags() & NET_DEVICE_FLAG_UP) != 0
    }
    fn is_close(&self) -> bool {
        (self.flags() & NET_DEVICE_FLAG_UP) == 0
    }
    fn set_open_flag(&mut self) {
        *self.flags_mut() &= NET_DEVICE_FLAG_UP;
    }
    fn set_close_flag(&mut self) {
        *self.flags_mut() &= !NET_DEVICE_FLAG_UP;
    }

    fn open(&self) -> Result<(), NetDeviceError>;
    fn output(&self, typ: NetProtocolType, data: &[u8], dst: ()) -> Result<(), NetDeviceError>;
    fn close(&self) -> Result<(), NetDeviceError>;
}

#[derive(Debug, Clone)]
pub(crate) enum NetDeviceError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NetProtocolType {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NetDeviceType {
    Dummy,
    Loopback,
    Ethernet,
}

impl From<NetDeviceError> for TcpIpError {
    fn from(value: NetDeviceError) -> Self {
        todo!()
    }
}

const NET_DEVICE_FLAG_UP: u16 = 0b0000_0000_0000_0001;
const NET_DEVICE_FLAG_LOOPBACK: u16 = 0b0000_0000_0000_0010;
const NET_DEVICE_FLAG_BROADCAST: u16 = 0b0000_0000_0000_0100;
const NET_DEVICE_FLAG_P2P: u16 = 0b0000_0000_0000_1000;
const NET_DEVICE_FLAG_NEED_ARP: u16 = 0b0000_0000_0001_0000;

#[derive(Debug, Clone)]
struct NetDeviceInner<const ALEN: usize> {
    typ: NetDeviceType,
    mtu: u16,
    flags: u16,
    hlen: u16,
    addr: [u8; ALEN],
    bloadcast: [u8; ALEN],
}
