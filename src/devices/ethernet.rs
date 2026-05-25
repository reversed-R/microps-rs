use std::fmt::Debug;

use crate::protocols::{AsHost, NetProtocolType};

pub(crate) const ETHER_HEADER_SIZE: usize = 14;
pub(crate) const ETHER_FRAME_SIZE_MIN: usize = 60;
pub(crate) const ETHER_FRAME_SIZE_MAX: usize = 1514;
pub(crate) const ETHER_PAYLOAD_SIZE_MIN: usize = ETHER_FRAME_SIZE_MIN - ETHER_HEADER_SIZE;
pub(crate) const ETHER_PAYLOAD_SIZE_MAX: usize = ETHER_FRAME_SIZE_MAX - ETHER_HEADER_SIZE;
pub(crate) const ETHER_ADDR_SIZE: usize = 6;
pub(crate) const ETHER_ADDR_ANY: EthernetAddr = EthernetAddr([0; _]);
pub(crate) const ETHER_ADDR_BROADCAST: EthernetAddr = EthernetAddr([0xff; _]);

#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct EthernetHeader {
    dst: [u8; ETHER_ADDR_SIZE],
    src: [u8; ETHER_ADDR_SIZE],
    typ: u16,
}

#[derive(Debug, Clone)]
pub(crate) enum EthernetError {
    UnsurpportedType { typ: u16 },
}

pub(crate) const ETHER_TYPE_IP: u16 = 0x0800;
pub(crate) const ETHER_TYPE_ARP: u16 = 0x0806;
pub(crate) const ETHER_TYPE_IPV6: u16 = 0x86dd;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EthernetType {
    Ip = ETHER_TYPE_IP,
    Arp = ETHER_TYPE_ARP,
    IpV6 = ETHER_TYPE_IPV6,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct EthernetAddr([u8; ETHER_ADDR_SIZE]);

impl EthernetHeader {
    pub(crate) fn new(dst: EthernetAddr, src: EthernetAddr, typ: EthernetType) -> Self {
        Self {
            dst: dst.0,
            src: src.0,
            typ: (typ as u16).to_be(),
        }
    }

    #[inline(always)]
    pub(crate) fn dst(&self) -> EthernetAddr {
        EthernetAddr(self.dst)
    }

    #[inline(always)]
    pub(crate) fn src(&self) -> EthernetAddr {
        EthernetAddr(self.src)
    }

    pub(crate) fn typ(&self) -> Result<EthernetType, EthernetError> {
        match self.typ.as_host() {
            ETHER_TYPE_IP => Ok(EthernetType::Ip),
            ETHER_TYPE_ARP => Ok(EthernetType::Arp),
            ETHER_TYPE_IPV6 => Ok(EthernetType::IpV6),
            x => Err(EthernetError::UnsurpportedType { typ: x }),
        }
    }
}

impl Debug for EthernetAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5],
        )
    }
}

impl From<EthernetType> for NetProtocolType {
    fn from(value: EthernetType) -> Self {
        match value {
            EthernetType::Ip => NetProtocolType::Ip,
            EthernetType::Arp => NetProtocolType::Arp,
            EthernetType::IpV6 => NetProtocolType::IpV6,
        }
    }
}

impl From<NetProtocolType> for EthernetType {
    fn from(value: NetProtocolType) -> Self {
        match value {
            NetProtocolType::Ip => EthernetType::Ip,
            NetProtocolType::Arp => EthernetType::Arp,
            NetProtocolType::IpV6 => EthernetType::IpV6,
        }
    }
}

impl EthernetAddr {
    pub(crate) fn new(addr: [u8; ETHER_ADDR_SIZE]) -> Self {
        Self(addr)
    }
}
