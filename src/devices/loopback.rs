use crate::{
    devices::{NET_DEVICE_FLAG_LOOPBACK, NetDevice, NetDeviceInner, NetDeviceType},
    info,
    net::net_input,
    print::debugdump,
};

/// maximum size of IP datagram
const LOOPBACK_MTU: u16 = u16::MAX;

#[derive(Debug)]
pub struct LoopbackDevice {
    inner: NetDeviceInner,
}

impl LoopbackDevice {
    pub fn new() -> Self {
        Self {
            inner: NetDeviceInner {
                typ: NetDeviceType::Loopback,
                mtu: LOOPBACK_MTU,
                flags: NET_DEVICE_FLAG_LOOPBACK,
                hlen: 0,          // non header
                addr: Vec::new(), // non address
                bloadcast: Vec::new(),
            },
        }
    }
}

impl NetDevice for LoopbackDevice {
    fn info(&self) -> &NetDeviceInner {
        &self.inner
    }

    fn open(&self) -> Result<(), super::NetDeviceError> {
        Ok(()) // nothing to do
    }

    fn output(
        &self,
        typ: super::NetProtocolType,
        data: &[u8],
        dst: (),
    ) -> Result<(), super::NetDeviceError> {
        info!("loopback device: type={typ:?}, len={}", data.len());

        debugdump(data);

        net_input(typ, data)
    }

    fn close(&self) -> Result<(), super::NetDeviceError> {
        Ok(()) // nothing to do
    }
}
