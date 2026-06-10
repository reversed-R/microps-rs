mod ip;

use std::sync::Arc;

pub(crate) use ip::IpIface;

use crate::protocols::NetProtocolError;

#[derive(Debug, Clone)]
pub(crate) enum NetIface {
    Ip(IpIface),
    // IpV6, // TODO
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IfaceFamilyKind {
    Ip,
    // IpV6,
}

impl NetIface {
    pub(crate) fn family_kind(&self) -> IfaceFamilyKind {
        match self {
            Self::Ip(_) => IfaceFamilyKind::Ip,
            // Self::IpV6 => IfaceFamilyKind::IpV6,
        }
    }

    pub(crate) fn dev(&self) -> Option<Arc<crate::net::NetDeviceContainer>> {
        match self {
            NetIface::Ip(ip_iface) => ip_iface.dev.upgrade(),
        }
    }

    pub(crate) fn set_dev(&mut self, dev: &Arc<crate::net::NetDeviceContainer>) {
        match self {
            NetIface::Ip(ip_iface) => {
                ip_iface.dev = Arc::downgrade(dev);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum NetIfaceError {}

impl From<NetIfaceError> for NetProtocolError {
    fn from(value: NetIfaceError) -> Self {
        Self::IfaceError { error: value }
    }
}
