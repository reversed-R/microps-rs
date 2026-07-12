mod rx_ring;

use std::sync::Arc;

use crate::{
    dbg,
    devices::{
        DeviceId, NET_DEVICE_FLAG_LOOPBACK, NetDevice, NetDeviceAddr, NetDeviceInner,
        NetDeviceType, loopback::rx_ring::LoRxRing,
    },
    info,
    net::ProtocolStackContext,
    print::debugdump,
    protocols::{IP_ADDR_BROADCAST, IP_ADDR_LOOPBACK, NetProtocolType},
};

/// maximum size of IP datagram
const LOOPBACK_MTU: u16 = u16::MAX;

#[derive(Debug)]
pub struct LoopbackDevice {
    inner: Arc<NetDeviceInner>,
    rx_ring: Arc<LoRxRing>,
}

impl LoopbackDevice {
    pub fn new(dev_id: DeviceId) -> Self {
        Self {
            inner: Arc::new(NetDeviceInner {
                dev_id,
                typ: NetDeviceType::Loopback,
                mtu: LOOPBACK_MTU,
                flags: NET_DEVICE_FLAG_LOOPBACK,
                hlen: 0,                                   // non header
                addr: NetDeviceAddr::Ip(IP_ADDR_LOOPBACK), // non address
                bloadcast: NetDeviceAddr::Ip(IP_ADDR_BROADCAST),
            }),
            rx_ring: Arc::new(LoRxRing::new()),
        }
    }
}

impl NetDevice for LoopbackDevice {
    fn info(&self) -> Arc<NetDeviceInner> {
        Arc::clone(&self.inner)
    }

    fn open(&self, _ctx: ProtocolStackContext) -> Result<(), super::NetDeviceError> {
        dbg!("opening loopback device...");
        Ok(()) // nothing to do
    }

    fn output(
        &self,
        ctx: ProtocolStackContext,
        typ: crate::protocols::NetProtocolType,
        data: &[u8],
        _dst: super::EthernetAddr,
    ) -> Result<(), super::NetDeviceError> {
        info!("loopback device: type={typ:?}, len={}", data.len());

        debugdump(data);

        // SAFETY: only this thread call rx_ring.buf_to_write()
        match unsafe { self.rx_ring.buf_to_write() } {
            Some(buf) => {
                // rx_ring に書き込みのみする
                buf.buf.copy_from_slice(data);
                buf.commit(data.len());

                // soft irq をリクエストして直ちに終了
                ctx.request_rx_irq(self.inner.dev_id())
            }
            None => {
                // 空きバッファがない場合、パケットを破棄
                Ok(())
            }
        }
    }

    fn close(&self) -> Result<(), super::NetDeviceError> {
        dbg!("closing loopback device...");
        Ok(()) // nothing to do
    }

    unsafe fn rx_next_clean_buf<'a>(&'a self) -> Option<super::RxBuf<'a>> {
        unsafe { self.rx_ring.buf_to_clean() }.map(|(desc, buf)| super::RxBuf {
            desc,
            buf,
            typ: NetProtocolType::Ip, // 必ずタイプはIP
        })
    }

    unsafe fn rx_free_buf(&self, desc: super::RxBufDesc) {
        unsafe { self.rx_ring.free_buf(desc) };
    }
}
