use std::sync::{Arc, Weak};

use crate::protocols::{IP_ADDR_BROADCAST, IpAddr, IpHeader};

#[derive(Debug)]
pub(crate) struct IpIface {
    pub(crate) dev: Weak<crate::net::NetDeviceContainer>,
    unicast: IpAddr,
    netmask: IpAddr,
    broadcast: IpAddr,
}

impl IpIface {
    pub(crate) fn new(unicast: IpAddr, netmask: IpAddr) -> Self {
        Self {
            dev: Weak::new(),
            unicast,
            netmask,
            broadcast: IpAddr::broadcast(unicast, netmask),
        }
    }

    pub(crate) fn dev(&self) -> Option<Arc<crate::net::NetDeviceContainer>> {
        self.dev.upgrade()
    }

    #[inline]
    pub(crate) fn unicast(&self) -> &IpAddr {
        &self.unicast
    }

    #[inline]
    pub(crate) fn netmask(&self) -> &IpAddr {
        &self.netmask
    }

    #[inline]
    pub(crate) fn broadcast(&self) -> &IpAddr {
        &self.broadcast
    }

    pub(crate) fn should_proceed_packet(&self, hdr: &IpHeader) -> bool {
        hdr.dst() == self.unicast || hdr.dst() == self.broadcast || hdr.dst() == IP_ADDR_BROADCAST
    }
}
