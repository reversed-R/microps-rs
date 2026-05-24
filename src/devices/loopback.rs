use crate::{
    dbg,
    devices::{DeviceId, NET_DEVICE_FLAG_LOOPBACK, NetDevice, NetDeviceInner, NetDeviceType},
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
    pub fn new(dev_id: DeviceId) -> Self {
        Self {
            inner: NetDeviceInner {
                dev_id,
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

    fn open(&mut self) -> Result<(), super::NetDeviceError> {
        dbg!("opening loopback device...");
        Ok(()) // nothing to do
    }

    fn output(
        &self,
        typ: crate::protocols::NetProtocolType,
        data: &[u8],
        _dst: &super::HardwareAddr,
    ) -> Result<(), super::NetDeviceError> {
        info!("loopback device: type={typ:?}, len={}", data.len());

        debugdump(data);

        input_to_app(self.inner.dev_id(), typ, data)
    }

    fn close(&mut self) -> Result<(), super::NetDeviceError> {
        dbg!("closing loopback device...");
        Ok(()) // nothing to do
    }
}
