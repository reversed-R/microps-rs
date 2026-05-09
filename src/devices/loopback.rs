use crate::{
    dbg,
    devices::{NET_DEVICE_FLAG_LOOPBACK, NetDevice, NetDeviceInner, NetDeviceType},
    info,
    net::input_to_app,
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
        dbg!("opening loopback device...");
        Ok(()) // nothing to do
    }

    fn output(
        &self,
        typ: crate::protocols::NetProtocolType,
        data: &[u8],
        dst: (),
        dev: &crate::net::NetDeviceContainer,
    ) -> Result<(), super::NetDeviceError> {
        info!("loopback device: type={typ:?}, len={}", data.len());

        debugdump(data);

        input_to_app(typ, data, dev)
    }

    fn close(&self) -> Result<(), super::NetDeviceError> {
        dbg!("closing loopback device...");
        Ok(()) // nothing to do
    }
}
