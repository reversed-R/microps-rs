use crate::{
    dbg,
    interfaces::NetIfaceError,
    protocols::{IP_ADDR_BROADCAST, IpAddr, IpHeader},
};

#[derive(Debug)]
pub(crate) struct IpIface {
    unicast: IpAddr,
    netmask: IpAddr,
    broadcast: IpAddr,
}

impl IpIface {
    pub(crate) fn new(unicast: IpAddr, netmask: IpAddr) -> Self {
        Self {
            unicast,
            netmask,
            broadcast: IpAddr::broadcast(unicast, netmask),
        }
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

    pub(crate) fn handle(&self, hdr: IpHeader, payload: &[u8]) -> Result<(), NetIfaceError> {
        // packet filtering
        if hdr.dst() == self.unicast
            || hdr.dst() == self.broadcast
            || hdr.dst() == IP_ADDR_BROADCAST
        {
            // for me
            dbg!("ip packet for me filtered!");
            Ok(())
        } else {
            // ignore: for other hosts
            dbg!("ip packet for other hosts ignored.");
            Ok(())
        }
    }
}
