use std::sync::Arc;

use crate::{
    devices::{DeviceId, NetDeviceError},
    interfaces::{IpIface, NetIface},
    net::ProtocolStackApp,
    print::debugdump,
    protocols::{IpAddr, NetProtocolType},
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

    pub(crate) fn input(
        &self,
        dev_id: DeviceId,
        typ: NetProtocolType,
        data: &[u8],
    ) -> Result<(), NetDeviceError> {
        dbg!("net_input: type={typ:?}, len={}", data.len());

        debugdump(data);

        let dev = self.app.devices.get(dev_id.value()).unwrap();

        for proto in &self.app.protocols {
            if proto.typ() == typ {
                proto.handle(self.clone(), data, dev)?;

                return Ok(());
            }
        }

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
}
