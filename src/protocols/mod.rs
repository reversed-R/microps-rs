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
    fn handle(&self, data: &[u8], dev: &dyn NetDevice);
}

#[derive(Debug, Clone)]
pub(crate) enum NetProtocolError {
    UnsurpportedProtocol { proto: u16 },
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
