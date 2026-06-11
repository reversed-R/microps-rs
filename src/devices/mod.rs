use std::{fmt::Debug, sync::Arc};

use crate::{
    AppError,
    net::ProtocolStackContext,
    protocols::{IpAddr, NetProtocolError, NetProtocolType},
};

pub(crate) mod ethernet;
pub(crate) mod loopback;

pub(crate) use ethernet::EthernetAddr;

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
    fn info(&self) -> Arc<NetDeviceInner>;
    fn open(&self, ctx: ProtocolStackContext) -> Result<(), NetDeviceError>;
    fn output(
        &self,
        ctx: ProtocolStackContext,
        typ: NetProtocolType,
        data: &[u8],
        dst: EthernetAddr,
    ) -> Result<(), NetDeviceError>;
    fn close(&self) -> Result<(), NetDeviceError>;
}

#[derive(Debug, Clone)]
pub(crate) enum NetDeviceError {
    ProtocolError { err: NetProtocolError },
    EtherTapOpenFailed,
    OutOfPayloadSize { size: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NetDeviceType {
    Dummy,
    Loopback,
    Ethernet,
}

impl From<NetDeviceError> for AppError {
    fn from(value: NetDeviceError) -> Self {
        Self::DeviceError { error: value }
    }
}

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
    addr: NetDeviceAddr,
    bloadcast: NetDeviceAddr,
}

#[derive(Debug, Clone)]
pub(crate) enum NetDeviceAddr {
    Ethernet(EthernetAddr),
    Ip(IpAddr),
}

impl NetDeviceInner {
    #[inline(always)]
    pub(crate) fn new(
        dev_id: DeviceId,
        typ: NetDeviceType,
        mtu: u16,
        flags: u16,
        hlen: u16,
        addr: NetDeviceAddr,
        bloadcast: NetDeviceAddr,
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
    pub(crate) fn addr(&self) -> &NetDeviceAddr {
        &self.addr
    }
    #[inline(always)]
    pub(crate) fn set_addr(&mut self, addr: NetDeviceAddr) {
        self.addr = addr;
    }
    #[inline(always)]
    pub(crate) fn bloadcast(&self) -> &NetDeviceAddr {
        &self.bloadcast
    }
}
