use std::fmt::Debug;

use crate::{
    TcpIpError,
    net::NetDeviceContainer,
    protocols::{NetProtocolError, NetProtocolType},
};

mod loopback;

pub use loopback::LoopbackDevice;

pub(crate) trait NetDevice: Debug + Send + Sync + 'static {
    fn info(&self) -> &NetDeviceInner;
    fn open(&self) -> Result<(), NetDeviceError>;
    fn output(
        &self,
        typ: NetProtocolType,
        data: &[u8],
        dst: (),
        dev: &NetDeviceContainer, // self が含まれるdevice container
    ) -> Result<(), NetDeviceError>;
    fn close(&self) -> Result<(), NetDeviceError>;
}

#[derive(Debug, Clone)]
pub(crate) enum NetDeviceError {
    ProtocolError { err: NetProtocolError },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NetDeviceType {
    Dummy,
    Loopback,
    Ethernet,
}

impl From<NetDeviceError> for TcpIpError {
    fn from(value: NetDeviceError) -> Self {
        Self::DeviceError { error: value }
    }
}

// const NET_DEVICE_FLAG_UP: u16 = 0b0000_0000_0000_0001;
const NET_DEVICE_FLAG_LOOPBACK: u16 = 0b0000_0000_0000_0010;
const NET_DEVICE_FLAG_BROADCAST: u16 = 0b0000_0000_0000_0100;
const NET_DEVICE_FLAG_P2P: u16 = 0b0000_0000_0000_1000;
const NET_DEVICE_FLAG_NEED_ARP: u16 = 0b0000_0000_0001_0000;

#[derive(Debug, Clone)]
pub(crate) struct NetDeviceInner {
    typ: NetDeviceType,
    mtu: u16,
    flags: u16,
    hlen: u16,
    addr: Vec<u8>,
    bloadcast: Vec<u8>,
}

impl NetDeviceInner {
    pub(crate) fn typ(&self) -> NetDeviceType {
        self.typ
    }
    pub(crate) fn mtu(&self) -> u16 {
        self.mtu
    }
    pub(crate) fn flags(&self) -> u16 {
        self.flags
    }
    pub(crate) fn flags_mut(&mut self) -> &mut u16 {
        &mut self.flags
    }
    pub(crate) fn hlen(&self) -> u16 {
        self.hlen
    }
    pub(crate) fn addr(&self) -> &[u8] {
        &self.addr
    }
    pub(crate) fn bloadcast(&self) -> &[u8] {
        &self.bloadcast
    }
}
