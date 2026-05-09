mod ip;

pub(crate) use ip::IpIface;

use crate::protocols::NetProtocolError;

#[derive(Debug)]
pub(crate) enum NetIface {
    Ip(IpIface),
    IpV6, // TODO
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IfaceFamilyKind {
    Ip,
    IpV6,
}

impl NetIface {
    pub(crate) fn family_kind(&self) -> IfaceFamilyKind {
        match self {
            Self::Ip(_) => IfaceFamilyKind::Ip,
            Self::IpV6 => IfaceFamilyKind::IpV6,
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
