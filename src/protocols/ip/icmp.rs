use crate::{dbg, protocols::ip::IpUpperProtocolHandler};

#[derive(Debug, Clone)]
pub(crate) struct IcmpProtocol;

impl IpUpperProtocolHandler for IcmpProtocol {
    fn protocol(&self) -> &super::IpUpperProtocolType {
        &super::IpUpperProtocolType::Icmp
    }

    fn handle(
        &self,
        hdr: super::IpHeader,
        payload: &[u8],
        iface: &crate::interfaces::IpIface,
    ) -> Result<(), super::IpProtocolError> {
        dbg!("ICMP handling...");

        if payload.len() < SIZE_OF_ICMP_HEADER {
            return Err(super::IpProtocolError::IcmpError {
                error: IcmpError::TooShortPacket { len: payload.len() },
            });
        }

        if super::cksum16_from_bytes(payload, 0) != 0 {
            return Err(super::IpProtocolError::IcmpError {
                error: IcmpError::BrokenCheckSum,
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) enum IcmpError {
    TooShortPacket { len: usize },
    BrokenCheckSum,
}

#[repr(C)]
#[derive(Clone)]
struct IcmpHeader {
    common: IcmpHeaderCommon,

    /// message dependent field
    dep: [u8; 4],
}

const SIZE_OF_ICMP_HEADER: usize = 8;
const _: () = assert!(SIZE_OF_ICMP_HEADER == core::mem::size_of::<IcmpHeader>());

#[repr(C)]
#[derive(Debug, Clone)]
struct IcmpHeaderCommon {
    typ: u8,
    code: u8,
    sum: u16,
}

const ICMP_TYPE_ECHO_REPLY: u8 = 0;
const ICMP_TYPE_DEST_UNREACH: u8 = 3;
const ICMP_TYPE_SOURCE_QUENCH: u8 = 4;
const ICMP_TYPE_REDIRECT: u8 = 5;
const ICMP_TYPE_ECHO: u8 = 8;
const ICMP_TYPE_TIME_EXCEEDED: u8 = 11;
const ICMP_TYPE_PARAM_PROBLEM: u8 = 12;
const ICMP_TYPE_TIMESTAMP: u8 = 13;
const ICMP_TYPE_TIMESTAMP_REPLY: u8 = 14;
const ICMP_TYPE_INFO_REQUEST: u8 = 15;
const ICMP_TYPE_INFO_REPLY: u8 = 16;

#[derive(Debug, Clone)]
struct IcmpHeaderEcho {
    common: IcmpHeaderCommon,
    id: u16,
    seq: u16,
}

#[derive(Debug, Clone)]
struct IcmpHeaderUnreach {
    common: IcmpHeaderCommon,

    /// unused (zero)
    unused: u32,
}
