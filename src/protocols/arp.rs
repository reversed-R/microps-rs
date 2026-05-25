use crate::{
    devices::{
        EthernetAddr,
        ethernet::{ETHER_ADDR_SIZE, ETHER_TYPE_IP},
    },
    interfaces::NetIface,
    protocols::{IpAddr, NetProtocol, ip::IP_ADDR_SIZE},
};

pub(crate) const ARP_OP_REQUEST: u16 = 0x0001;
pub(crate) const ARP_OP_REPLY: u16 = 0x0002;

pub(crate) const ARP_HRD_ETHER: u16 = 0x0001;

pub(crate) const ARP_PRO_IP: u16 = ETHER_TYPE_IP;

#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct ArpHeader {
    /// Hardware address space
    hrd: u16,

    /// Protocol address space
    pro: u16,

    /// Hardware address length
    hln: u8,

    /// Protocol address length
    pln: u8,

    /// Operation code
    op: u16,
}
const _: () = assert!(ARP_HEADER_SIZE == core::mem::size_of::<ArpHeader>());
const ARP_HEADER_SIZE: usize = 8;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArpOp {
    Request = ARP_OP_REQUEST,
    Reply = ARP_OP_REPLY,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArpHardwareAddrSpace {
    Ethernet = ARP_HRD_ETHER,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArpProtocolAddrSpace {
    Ip = ARP_PRO_IP,
}

// NOTE: Ethernet Address (MAC Address)'s length is 6 byte (48 bit).
// So, prevent from being inserted padding between `sha` and `spa` (also `tha` and `tpa`)
// for alignment rule, I treat IP Address (v4, 4 byte = 32 bit) as [u8; 4] instead of u32.
#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct ArpEtherIpBody {
    /// Source hardware address
    sha: [u8; ETHER_ADDR_SIZE],

    /// Source Protocol address
    spa: [u8; IP_ADDR_SIZE],

    /// Target hardware address
    tha: [u8; ETHER_ADDR_SIZE],

    /// Target Protocol address
    tpa: [u8; IP_ADDR_SIZE],
}
const _: () = assert!(ARP_ETHER_IP_BODY_SIZE == core::mem::size_of::<ArpEtherIpBody>());
const ARP_ETHER_IP_BODY_SIZE: usize = 20;

#[repr(C)]
#[derive(Debug, Clone)]
struct ArpEtherIp {
    header: ArpHeader,
    body: ArpEtherIpBody,
}
const _: () = assert!(ARP_ETHER_IP_SIZE == core::mem::size_of::<ArpEtherIp>());
const ARP_ETHER_IP_SIZE: usize = ARP_HEADER_SIZE + ARP_ETHER_IP_BODY_SIZE;

impl ArpHeader {
    fn hrd(&self) -> Result<ArpHardwareAddrSpace, ArpProtocolError> {
        match u16::from_be(self.hrd) {
            ARP_HRD_ETHER => Ok(ArpHardwareAddrSpace::Ethernet),
            hrd => Err(ArpProtocolError::UnsurpportedHardwareAddress { hrd }),
        }
    }
    fn pro(&self) -> Result<ArpProtocolAddrSpace, ArpProtocolError> {
        match u16::from_be(self.pro) {
            ARP_PRO_IP => Ok(ArpProtocolAddrSpace::Ip),
            pro => Err(ArpProtocolError::UnsurpportedProtocolAddress { pro }),
        }
    }
    fn hln(&self) -> u8 {
        self.hln
    }
    fn pln(&self) -> u8 {
        self.pln
    }
    fn op(&self) -> Result<ArpOp, ArpProtocolError> {
        match u16::from_be(self.op) {
            ARP_OP_REQUEST => Ok(ArpOp::Request),
            ARP_OP_REPLY => Ok(ArpOp::Reply),
            op => Err(ArpProtocolError::UnsurpportedOperation { op }),
        }
    }
}

impl ArpEtherIpBody {
    fn sha(&self) -> EthernetAddr {
        EthernetAddr::new(self.sha)
    }
    fn spa(&self) -> IpAddr {
        IpAddr::new(u32::from_be_bytes(self.spa))
    }
    fn tha(&self) -> EthernetAddr {
        EthernetAddr::new(self.tha)
    }
    fn tpa(&self) -> IpAddr {
        IpAddr::new(u32::from_be_bytes(self.tpa))
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ArpProtocolError {
    UnsurpportedHardwareAddress { hrd: u16 },
    UnsurpportedProtocolAddress { pro: u16 },
    UnsurpportedOperation { op: u16 },
    TooShortPacket { len: usize },
}

impl From<ArpProtocolError> for super::NetProtocolError {
    fn from(value: ArpProtocolError) -> Self {
        Self::ArpProtocolError { error: value }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ArpProtocol;

impl NetProtocol for ArpProtocol {
    fn typ(&self) -> super::NetProtocolType {
        super::NetProtocolType::Arp
    }

    // NOTE: only surpports Ethernet Address (MAC Address) and IP Address (v4) resolution.
    fn handle(
        &self,
        data: &[u8],
        dev: &crate::net::NetDeviceContainer,
    ) -> Result<(), super::NetProtocolError> {
        if data.len() < ARP_ETHER_IP_SIZE {
            return Err(ArpProtocolError::TooShortPacket { len: data.len() }.into());
        }

        let msg_bytes: [u8; ARP_ETHER_IP_SIZE] = data[..ARP_ETHER_IP_SIZE].try_into().unwrap();
        let msg: ArpEtherIp = unsafe { core::mem::transmute(msg_bytes) };

        if msg.header.hrd()? != ArpHardwareAddrSpace::Ethernet
            || msg.header.hln() as usize != ETHER_ADDR_SIZE
        {
            return Err(ArpProtocolError::UnsurpportedHardwareAddress {
                hrd: u16::from_be(msg.header.hrd),
            }
            .into());
        }

        if msg.header.pro()? != ArpProtocolAddrSpace::Ip
            || msg.header.pln() as usize != IP_ADDR_SIZE
        {
            return Err(ArpProtocolError::UnsurpportedProtocolAddress {
                pro: u16::from_be(msg.header.pro),
            }
            .into());
        }

        for i in dev.state().ifaces() {
            match i {
                NetIface::Ip(ip_iface) => {
                    if ip_iface.unicast() == &msg.body.tpa() {
                        match msg.header.op()? {
                            ArpOp::Request => {
                                todo!()
                            }
                            ArpOp::Reply => {
                                todo!()
                            }
                        }
                    }
                }
                _ => {
                    continue;
                }
            }
        }

        dbg!("ip iface not found and packet ignored.");
        println!("dev={dev:?}");

        Ok(())
    }
}

pub(crate) fn output() {
    // TODO:
}
