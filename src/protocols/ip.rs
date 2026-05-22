use std::fmt::Debug;

use crate::{
    dbg,
    interfaces::NetIface,
    print::debugdump,
    protocols::{AsHost, NetProtocol, NetProtocolError, NetProtocolType},
};

#[derive(Debug, Clone)]
pub(crate) struct IpProtocol;

impl IpProtocol {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl NetProtocol for IpProtocol {
    fn typ(&self) -> super::NetProtocolType {
        NetProtocolType::Ip
    }

    fn handle(
        &self,
        data: &[u8],
        dev: &crate::net::NetDeviceContainer,
    ) -> Result<(), NetProtocolError> {
        dbg!("handling ip packet...");
        debugdump(data);

        if SIZE_OF_IP_HEADER < data.len() {
            let hdr_bytes: [u8; SIZE_OF_IP_HEADER] = data[..SIZE_OF_IP_HEADER].try_into().unwrap();
            let hdr: IpHeader = unsafe { core::mem::transmute(hdr_bytes) };

            dbg!("{hdr:?}",);

            if hdr.version() != IP_VERSION_IPV4 {
                return Err(NetProtocolError::UnsurpportedIpVersion {
                    version: hdr.version(),
                });
            }

            // hlen is N * 4, so even number.
            if cksum16_from_bytes(&data[..hdr.hlen()], 0) != 0 {
                return Err(NetProtocolError::BrokenCheckSum);
            }

            if hdr.total() > data.len() {
                return Err(NetProtocolError::TooShortPacket { len: data.len() });
            }

            if hdr.more_fragments() || hdr.offset() > 0 {
                return Err(NetProtocolError::FragmentUnsurpported);
            }

            for i in dev.state().ifaces() {
                match i {
                    NetIface::Ip(ip_iface) => {
                        dbg!("ip iface processing starts...");
                        let payload = &data[hdr.hlen()..hdr.total()];
                        ip_iface.handle(hdr, payload)?;

                        return Ok(());
                    }
                    _ => {
                        continue;
                    }
                }
            }

            dbg!("ip iface not found and packet ignored.");
            println!("dev={dev:?}");

            Ok(())
        } else {
            Err(NetProtocolError::TooShortPacket { len: data.len() })
        }
    }
}

/// IP Header
///
/// WARN: Multi byte fields are network byte order (big endian).
#[repr(C)]
pub(crate) struct IpHeader {
    /// IP version (4 bits) and Header Length (4 bits).
    /// If a Header Length is N, N * 4 (bytes) is a real header length.
    vhl: u8,

    /// Type Of Service.
    tos: u8,

    /// Total Length of data gram.
    /// This also contains header, so payload length is `total` - vhl & 0x0f * 4 (bytes).
    total: u16,

    /// Identification of IP packet.
    id: u16,

    /// Flags (3 bits) and Fragment Offset (13 bits).
    /// If a Fragment Offset is N, N * 8 (bytes) is a real fragment offset.
    offset: u16,

    /// Time To Live.
    /// Every time a packet pass through network hosts (such as router),
    /// ttl is decremented, and when ttl becomes 0, the packet will be discarded.
    ttl: u8,

    /// Protocol number.
    protocol: u8,

    /// Header Checksum.
    sum: u16,

    /// Source Address.
    src: u32,

    /// Destination Address.
    dst: u32,
}

const SIZE_OF_IP_HEADER: usize = 20;
const _: () = assert!(SIZE_OF_IP_HEADER == core::mem::size_of::<IpHeader>());

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct IpAddr(u32);

pub(crate) const IP_ADDR_BROADCAST: IpAddr = IpAddr(0xffffffff);
pub(crate) const IP_ADDR_LOOPBACK: IpAddr = IpAddr(0x7f000001);
pub(crate) const IP_ADDR_LOOPBACK_NETMASK: IpAddr = IpAddr(0xff000000);

impl Debug for IpAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.0.to_be_bytes();
        write!(f, "{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3])
    }
}

const IP_HEADER_FLAG_MF: u16 = 0x2000; // more fragments flag.
const IP_HEADER_FLAG_DF: u16 = 0x4000; // don't fragment flag.
const IP_HEADER_FLAG_RF: u16 = 0x8000; // reserved.
const IP_HEADER_OFFSET_MASK: u16 = 0x1fff;

const IP_VERSION_IPV4: u8 = 4;

impl Debug for IpHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"IpHeader{{
    version: {},
    hlen: {} (bytes),
    tos: {},
    total: {} (bytes),
    id: {},
    flags: DF: {}, MF: {},
    offset: {} (bytes),
    ttl: {},
    protocol: {},
    sum: {:#x},
    src: {:?},
    dst: {:?},
}}"#,
            self.version(),
            self.hlen(),
            self.tos,
            self.total(),
            self.id(),
            self.dont_fragment(),
            self.more_fragments(),
            self.offset(),
            self.ttl,
            self.protocol,
            self.sum(),
            self.src(),
            self.dst()
        )
    }
}

impl IpHeader {
    #[inline]
    fn version(&self) -> u8 {
        self.vhl >> 4
    }

    /// Header length (bytes).
    #[inline]
    fn hlen(&self) -> usize {
        (self.vhl & 0x0f) as usize * 4
    }

    /// Togal length (bytes).
    #[inline]
    fn total(&self) -> usize {
        self.total.as_host() as usize
    }

    #[inline]
    fn id(&self) -> u16 {
        self.id.as_host()
    }

    #[inline]
    fn more_fragments(&self) -> bool {
        (self.offset & IP_HEADER_FLAG_MF) != 0
    }

    #[inline]
    fn dont_fragment(&self) -> bool {
        (self.offset & IP_HEADER_FLAG_DF) != 0
    }

    /// Fragment offset (bytes).
    #[inline]
    fn offset(&self) -> usize {
        (self.offset.as_host() & IP_HEADER_OFFSET_MASK) as usize * 8
    }

    #[inline]
    fn sum(&self) -> u16 {
        self.sum.as_host()
    }

    #[inline]
    pub(crate) fn src(&self) -> IpAddr {
        IpAddr(self.src.as_host())
    }

    #[inline]
    pub(crate) fn dst(&self) -> IpAddr {
        IpAddr(self.dst.as_host())
    }
}

fn cksum16_from_bytes(data: &[u8], init: u32) -> u16 {
    let mut sum: u32 = init;

    let word_len = data.len() / 2;
    for widx in 0..word_len {
        let idx = widx * 2;
        sum += (((data[idx] as u16) << 8) + data[idx + 1] as u16) as u32;
    }

    // 最後のbyteがあれば計算
    if data.len() % 2 == 1 {
        sum += data[data.len() - 1] as u32;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }

    !(sum as u16)
}

impl From<u32> for IpAddr {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl IpAddr {
    pub(crate) fn broadcast(unicast: Self, netmask: Self) -> Self {
        Self((unicast.0 & netmask.0) | !netmask.0)
    }
}
