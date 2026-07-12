use std::sync::Arc;

use crate::{
    devices::{DeviceId, NetDeviceError},
    interfaces::{IpIface, NetIface},
    net::ProtocolStackApp,
    print::debugdump,
    protocols::{IpAddr, NetProtocol, NetProtocolKind, arp::ArpProtocol},
};

pub(crate) struct ProtocolStackContext {
    pub(super) app: Arc<ProtocolStackApp>,
}

impl Clone for ProtocolStackContext {
    fn clone(&self) -> Self {
        Self {
            app: Arc::clone(&self.app),
        }
    }
}

impl ProtocolStackContext {
    pub(crate) fn new(app: ProtocolStackApp) -> Self {
        Self { app: Arc::new(app) }
    }

    pub(crate) fn request_rx_irq(&self, dev_id: DeviceId) -> Result<(), NetDeviceError> {
        let ctx = self.clone();
        std::thread::spawn(move || {
            let dev = ctx.app.devices.get(dev_id.value()).unwrap();
            let Some(rx_buf) = (unsafe { dev.dev.rx_next_clean_buf() }) else {
                return;
            };
            // dev から clean すべき DMA buffer を dequeue
            // ここで初めて受信パケットのL2の処理が行われ、
            // L3への入力としてのバッファが返される

            debugdump(rx_buf.buf);

            for proto in &ctx.app.protocols {
                if proto.typ() == rx_buf.typ {
                    let res = proto.handle(ctx.clone(), rx_buf.buf, dev);

                    unsafe {
                        dev.dev.rx_free_buf(rx_buf.desc);
                    }

                    res.inspect_err(|e| {
                        println!("{e:?}");
                    })
                    .unwrap();
                    return;
                }
            }

            unsafe {
                dev.dev.rx_free_buf(rx_buf.desc);
            }
        });

        Ok(())
    }

    pub(crate) fn select_ip_iface(&self, addr: &IpAddr) -> Option<IpIface> {
        for dev in &self.app.devices {
            for iface in dev.state.ifaces.load().iter() {
                match iface {
                    NetIface::Ip(ip_iface) => {
                        if ip_iface.unicast() == addr {
                            return Some(ip_iface.clone());
                        }
                    }
                }
            }
        }

        None
    }

    pub(crate) fn arp(&self) -> Option<&ArpProtocol> {
        for proto in &self.app.protocols {
            if let NetProtocolKind::Arp(arp) = proto {
                return Some(arp);
            }
        }

        None
    }
}
