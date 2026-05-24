use std::fmt::Debug;

use crate::{
    TcpIpError,
    protocols::{NetProtocolError, NetProtocolType},
};

pub(crate) mod ethernet;
pub(crate) mod loopback;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct DeviceId(usize);

impl DeviceId {
    pub(crate) fn new(value: usize) -> Self {
        Self(value)
    }

    pub(crate) fn value(&self) -> usize {
        self.0
    }
}

pub(crate) trait NetDevice: Debug + Send + Sync + 'static {
    fn info(&self) -> &NetDeviceInner;
    fn open(&mut self) -> Result<(), NetDeviceError>;
    fn output(
        &self,
        typ: NetProtocolType,
        data: &[u8],
        dst: &HardwareAddr,
    ) -> Result<(), NetDeviceError>;
    fn close(&mut self) -> Result<(), NetDeviceError>;
}

#[derive(Debug, Clone)]
pub(crate) enum NetDeviceError {
    ProtocolError { err: NetProtocolError },
    EtherTapOpenFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NetDeviceType {
    Dummy,
    Loopback,
    Ethernet,
}

impl From<NetDeviceError> for TcpIpError {
    fn from(value: NetDeviceError) -> Self {
        Self::DeviceError { error: value }
    }
}

pub(crate) struct HardwareAddr(Vec<u8>);

impl HardwareAddr {
    pub(crate) fn new(addr: Vec<u8>) -> Self {
        Self(addr)
    }
}

// pub(crate) struct HardwareAddr<'a>(&'a [u8]);
//
// impl<'a> HardwareAddr<'a> {
//     pub(crate) fn new(addr: &'a [u8]) -> Self {
//         Self(addr)
//     }
// }

// const NET_DEVICE_FLAG_UP: u16 = 0b0000_0000_0000_0001;
pub(crate) const NET_DEVICE_FLAG_LOOPBACK: u16 = 0b0000_0000_0000_0010;
pub(crate) const NET_DEVICE_FLAG_BROADCAST: u16 = 0b0000_0000_0000_0100;
pub(crate) const NET_DEVICE_FLAG_P2P: u16 = 0b0000_0000_0000_1000;
pub(crate) const NET_DEVICE_FLAG_NEED_ARP: u16 = 0b0000_0000_0001_0000;

#[derive(Debug, Clone)]
pub(crate) struct NetDeviceInner {
    dev_id: DeviceId,
    typ: NetDeviceType,
    mtu: u16,
    flags: u16,
    hlen: u16,
    addr: Vec<u8>,
    bloadcast: Vec<u8>,
}

impl NetDeviceInner {
    #[inline(always)]
    pub(crate) fn new(
        dev_id: DeviceId,
        typ: NetDeviceType,
        mtu: u16,
        flags: u16,
        hlen: u16,
        addr: Vec<u8>,
        bloadcast: Vec<u8>,
    ) -> Self {
        Self {
            dev_id,
            typ,
            mtu,
            flags,
            hlen,
            addr,
            bloadcast,
        }
    }

    #[inline(always)]
    pub(crate) fn dev_id(&self) -> DeviceId {
        self.dev_id
    }
    #[inline(always)]
    pub(crate) fn typ(&self) -> NetDeviceType {
        self.typ
    }
    #[inline(always)]
    pub(crate) fn mtu(&self) -> u16 {
        self.mtu
    }
    #[inline(always)]
    pub(crate) fn flags(&self) -> u16 {
        self.flags
    }
    #[inline(always)]
    pub(crate) fn flags_mut(&mut self) -> &mut u16 {
        &mut self.flags
    }
    #[inline(always)]
    pub(crate) fn hlen(&self) -> u16 {
        self.hlen
    }
    #[inline(always)]
    pub(crate) fn addr(&self) -> &[u8] {
        &self.addr
    }
    #[inline(always)]
    pub(crate) fn bloadcast(&self) -> &[u8] {
        &self.bloadcast
    }
}
