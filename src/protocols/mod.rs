mod ip;

use std::fmt::Debug;

use crate::devices::NetDevice;

pub(crate) use ip::IpProtocol;

const NET_PROTOCOL_TYPE_IP: u16 = 0x0800;
const NET_PROTOCOL_TYPE_ARP: u16 = 0x0806;
const NET_PROTOCOL_TYPE_IPV6: u16 = 0x86dd;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NetProtocolType {
    Ip,
    Arp,
    IpV6,
}

pub(crate) trait NetProtocol: Debug + Send + Sync + 'static {
    fn typ(&self) -> NetProtocolType;
    fn handle(&self, data: &[u8], dev: &dyn NetDevice) -> Result<(), NetProtocolError>;
}

#[derive(Debug, Clone)]
pub(crate) enum NetProtocolError {
    UnsurpportedProtocol { proto: u16 },
    TooShortPacket { len: usize },
    UnsurpportedIpVersion { version: u8 },
    BrokenCheckSum,
    FragmentUnsurpported,
}

impl TryFrom<u16> for NetProtocolType {
    type Error = NetProtocolError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            NET_PROTOCOL_TYPE_IP => Ok(Self::Ip),
            NET_PROTOCOL_TYPE_ARP => Ok(Self::Arp),
            NET_PROTOCOL_TYPE_IPV6 => Ok(Self::IpV6),
            _ => Err(NetProtocolError::UnsurpportedProtocol { proto: value }),
        }
    }
}

trait AsHost<T> {
    fn as_host(&self) -> T;
}

trait AsNet<T> {
    fn as_net(&self) -> T;
}

impl AsNet<u16> for u16 {
    fn as_net(&self) -> u16 {
        self.to_be()
    }
}

impl AsHost<u16> for u16 {
    fn as_host(&self) -> u16 {
        u16::from_be(*self)
    }
}

impl AsNet<u32> for u32 {
    fn as_net(&self) -> u32 {
        self.to_be()
    }
}

impl AsHost<u32> for u32 {
    fn as_host(&self) -> u32 {
        u32::from_be(*self)
    }
}
