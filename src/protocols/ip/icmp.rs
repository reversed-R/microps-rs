use std::fmt::Debug;

use crate::{
    dbg,
    net::ProtocolStackContext,
    protocols::{IpAddr, NetProtocolOutputError, ip::IpUpperProtocolHandler},
};

#[derive(Debug, Clone)]
pub(crate) struct IcmpProtocol;

impl IpUpperProtocolHandler for IcmpProtocol {
    fn protocol(&self) -> &super::IpUpperProtocolType {
        &super::IpUpperProtocolType::Icmp
    }

    fn handle(
        &self,
        ctx: ProtocolStackContext,
        ip_hdr: super::IpHeader,
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

        let hdr_bytes: [u8; SIZE_OF_ICMP_HEADER] =
            payload[..SIZE_OF_ICMP_HEADER].try_into().unwrap();
        let hdr: IcmpHeader = unsafe { core::mem::transmute(hdr_bytes) };

        dbg!("{:?}", hdr);

        match hdr.common.typ()? {
            IcmpType::Echo => {
                dbg!("ICMP ECHO handling...");

                output(
                    ctx,
                    IcmpType::EchoReply,
                    hdr.common
                        .code()
                        .map_err(|error| super::IpProtocolError::IcmpError { error })?,
                    hdr.dep,
                    &payload[SIZE_OF_ICMP_HEADER..],
                    *iface.unicast(),
                    ip_hdr.src(),
                )
                .map_err(|error| super::IpProtocolError::IcmpOutputError { error })
            }

            x => {
                // TODO:
                dbg!("ICMP type={:?}", x);
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum IcmpError {
    TooShortPacket { len: usize },
    BrokenCheckSum,
    UnsurpportedCode { code: u8 },
    UnsurpportedType { typ: u8 },
}

impl From<IcmpError> for super::IpProtocolError {
    fn from(value: IcmpError) -> Self {
        Self::IcmpError { error: value }
    }
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

impl IcmpHeaderCommon {
    fn typ(&self) -> Result<IcmpType, IcmpError> {
        IcmpType::try_from(self.typ)
    }
    fn code(&self) -> Result<IcmpCode, IcmpError> {
        IcmpCode::try_from(self.code)
    }
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

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IcmpType {
    EchoReply = ICMP_TYPE_ECHO_REPLY,
    DestUnreach = ICMP_TYPE_DEST_UNREACH,
    SourceQuench = ICMP_TYPE_SOURCE_QUENCH,
    Redirect = ICMP_TYPE_REDIRECT,
    Echo = ICMP_TYPE_ECHO,
    TimeExceeded = ICMP_TYPE_TIME_EXCEEDED,
    ParamProblem = ICMP_TYPE_PARAM_PROBLEM,
    Timestamp = ICMP_TYPE_TIMESTAMP,
    TimestampReply = ICMP_TYPE_TIMESTAMP_REPLY,
    InfoRequest = ICMP_TYPE_INFO_REQUEST,
    InfoReply = ICMP_TYPE_INFO_REPLY,
}

impl TryFrom<u8> for IcmpType {
    type Error = IcmpError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            ICMP_TYPE_ECHO_REPLY => Ok(Self::EchoReply),
            ICMP_TYPE_DEST_UNREACH => Ok(Self::DestUnreach),
            ICMP_TYPE_SOURCE_QUENCH => Ok(Self::SourceQuench),
            ICMP_TYPE_REDIRECT => Ok(Self::Redirect),
            ICMP_TYPE_ECHO => Ok(Self::Echo),
            ICMP_TYPE_TIME_EXCEEDED => Ok(Self::TimeExceeded),
            ICMP_TYPE_PARAM_PROBLEM => Ok(Self::ParamProblem),
            ICMP_TYPE_TIMESTAMP => Ok(Self::Timestamp),
            ICMP_TYPE_TIMESTAMP_REPLY => Ok(Self::TimestampReply),
            ICMP_TYPE_INFO_REQUEST => Ok(Self::InfoRequest),
            ICMP_TYPE_INFO_REPLY => Ok(Self::InfoReply),
            _ => Err(IcmpError::UnsurpportedType { typ: value }),
        }
    }
}

const ICMP_CODE_NET_UNREACH: u8 = 0;
const ICMP_CODE_HOST_UNREACH: u8 = 1;
const ICMP_CODE_PROTO_UNREACH: u8 = 2;
const ICMP_CODE_PORT_UNREACH: u8 = 3;
const ICMP_CODE_FRAGMENT_NEEDED: u8 = 4;
const ICMP_CODE_SOURCE_ROUTE_FAILED: u8 = 5;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IcmpCode {
    NetUnreach = ICMP_CODE_NET_UNREACH,
    HostUnreach = ICMP_CODE_HOST_UNREACH,
    ProtoUnreach = ICMP_CODE_PROTO_UNREACH,
    PortUnreach = ICMP_CODE_PORT_UNREACH,
    FragmentNeeded = ICMP_CODE_FRAGMENT_NEEDED,
    SourceRouteFailed = ICMP_CODE_SOURCE_ROUTE_FAILED,
}

impl TryFrom<u8> for IcmpCode {
    type Error = IcmpError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            ICMP_CODE_NET_UNREACH => Ok(Self::NetUnreach),
            ICMP_CODE_HOST_UNREACH => Ok(Self::HostUnreach),
            ICMP_CODE_PROTO_UNREACH => Ok(Self::ProtoUnreach),
            ICMP_CODE_PORT_UNREACH => Ok(Self::PortUnreach),
            ICMP_CODE_FRAGMENT_NEEDED => Ok(Self::FragmentNeeded),
            ICMP_CODE_SOURCE_ROUTE_FAILED => Ok(Self::SourceRouteFailed),
            _ => Err(IcmpError::UnsurpportedCode { code: value }),
        }
    }
}

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

#[derive(Debug, Clone)]
pub(crate) enum IcmpOutputError {
    NetProtocolOutputError { error: NetProtocolOutputError },
}

impl Debug for IcmpHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"IcmpHeader{{
    typ: {:?},
    code: {:?},
    sum: {},
    dep: [{}, {}, {}, {}]
}}"#,
            self.common.typ(),
            self.common.code(),
            self.common.sum,
            self.dep[0],
            self.dep[1],
            self.dep[2],
            self.dep[3]
        )
    }
}

pub(crate) fn output(
    ctx: ProtocolStackContext,
    typ: IcmpType,
    code: IcmpCode,
    val: [u8; 4],
    data: &[u8],
    src: IpAddr,
    dst: IpAddr,
) -> Result<(), IcmpOutputError> {
    // FIXME: buffer size now hard coded as MTU 1500
    let mut buf = [0u8; 1500];
    let len = SIZE_OF_ICMP_HEADER + data.len();

    let hdr = IcmpHeader {
        common: IcmpHeaderCommon {
            typ: typ as u8,
            code: code as u8,
            sum: 0,
        },
        dep: val,
    };

    dbg!("icmp output... src={:?}, dst={:?}", src, dst);
    dbg!("{:?}", hdr);

    buf[..SIZE_OF_ICMP_HEADER].copy_from_slice(&unsafe {
        core::mem::transmute::<IcmpHeader, [u8; SIZE_OF_ICMP_HEADER]>(hdr)
    });
    buf[SIZE_OF_ICMP_HEADER..len].copy_from_slice(data);

    // calculate check sum and set
    let sum = super::cksum16_from_bytes(&buf[..len], 0);
    let sum_bytes = sum.to_be_bytes();
    buf[2] = sum_bytes[0];
    buf[3] = sum_bytes[1];

    super::output(ctx, super::IpUpperProtocolType::Icmp, &buf[..len], src, dst)
        .map_err(|error| IcmpOutputError::NetProtocolOutputError { error })
}
