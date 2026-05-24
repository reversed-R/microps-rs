use std::{fs::OpenOptions, io::Read, os::fd::AsRawFd};

use crate::{
    dbg,
    devices::{
        self, DeviceId, NetDevice, NetDeviceInner, NetDeviceType,
        ethernet::{
            ETHER_ADDR_ANY, ETHER_ADDR_BROADCAST, ETHER_ADDR_SIZE, ETHER_HEADER_SIZE, EthernetAddr,
            EthernetHeader,
        },
    },
    error,
    print::debugdump,
};

#[repr(C)]
struct IfReq {
    name: [libc::c_char; libc::IFNAMSIZ],
    flags: libc::c_short,

    __pading: [u8; 24],
}

#[derive(Debug)]
pub struct EtherTapDevice {
    inner: NetDeviceInner,
    tap_file: Option<std::fs::File>,
    hw_addr: EthernetAddr,
}

impl EtherTapDevice {
    pub(crate) fn new(dev_id: DeviceId) -> Self {
        Self {
            inner: NetDeviceInner::new(
                dev_id,
                NetDeviceType::Ethernet,
                devices::ethernet::ETHER_PAYLOAD_SIZE_MAX as u16,
                devices::NET_DEVICE_FLAG_BROADCAST | devices::NET_DEVICE_FLAG_NEED_ARP,
                devices::ethernet::ETHER_HEADER_SIZE as u16,
                Vec::new(),
                Vec::new(),
            ),
            tap_file: None,
            hw_addr: ETHER_ADDR_ANY,
        }
    }
}

impl NetDevice for EtherTapDevice {
    fn info(&self) -> &NetDeviceInner {
        &self.inner
    }

    fn open(&mut self) -> Result<(), crate::devices::NetDeviceError> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/net/tun")
            .unwrap();

        // let mut ifreq = IfReq {
        //     name: [0; _],
        //     flags: (libc::IFF_TAP | libc::IFF_NO_PI) as libc::c_short,
        //     __pading: [0; _],
        // };
        let mut ifreq = libc::ifreq {
            ifr_name: [0; _],
            ifr_ifru: libc::__c_anonymous_ifr_ifru {
                ifru_flags: (libc::IFF_TAP | libc::IFF_NO_PI) as libc::c_short,
            },
        };

        let name = b"tap0";
        for (i, c) in name.iter().enumerate() {
            ifreq.ifr_name[i] = *c as libc::c_char;
        }

        if unsafe { libc::ioctl(file.as_raw_fd(), libc::TUNSETIFF, &ifreq) } == -1 {
            error!("failed to open tap device");
            return Err(crate::devices::NetDeviceError::EtherTapOpenFailed);
        }

        let soc = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
        if soc == -1 {
            error!("failed to socket");
            return Err(crate::devices::NetDeviceError::EtherTapOpenFailed);
        }

        if unsafe { libc::ioctl(soc, libc::SIOCGIFHWADDR, &mut ifreq) } == -1 {
            error!("failed to get hardware address");
            return Err(crate::devices::NetDeviceError::EtherTapOpenFailed);
        }

        self.tap_file = Some(file.try_clone().unwrap());

        // get hardware address
        if self.hw_addr == ETHER_ADDR_ANY {
            let raw_addr = unsafe { ifreq.ifr_ifru.ifru_hwaddr.sa_data };
            let mut addr = [0u8; ETHER_ADDR_SIZE];
            for (i, b) in raw_addr[..ETHER_ADDR_SIZE].iter().enumerate() {
                addr[i] = *b as u8;
            }
            self.hw_addr = EthernetAddr::new(addr);
        }
        let addr = self.hw_addr.clone();

        let dev_id = self.inner.dev_id();
        std::thread::spawn(move || {
            let mut buf = [0u8; devices::ethernet::ETHER_FRAME_SIZE_MAX];
            loop {
                match file.read(&mut buf) {
                    Ok(n) => {
                        dbg!("read from ether tap device: n={}", n);
                        debugdump(&buf[..n]);

                        if n < ETHER_HEADER_SIZE {
                            dbg!("too short data");
                            continue;
                        }

                        let hdr_bytes: [u8; ETHER_HEADER_SIZE] =
                            buf[..ETHER_HEADER_SIZE].try_into().unwrap();
                        let hdr: EthernetHeader = unsafe { core::mem::transmute(hdr_bytes) };
                        let typ = hdr.typ().unwrap();
                        dbg!(
                            "ether header: src={:?}, dst={:?}, typ={:?}",
                            hdr.src(),
                            hdr.dst(),
                            typ
                        );

                        if hdr.dst() != addr && hdr.dst() != ETHER_ADDR_BROADCAST {
                            dbg!("for other host");
                            continue;
                        }

                        crate::net::input_to_app(dev_id, typ.into(), &buf[ETHER_HEADER_SIZE..n])
                            .unwrap();
                    }
                    Err(e) => {
                        error!("{}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    fn output(
        &self,
        typ: crate::protocols::NetProtocolType,
        data: &[u8],
        dst: &crate::devices::HardwareAddr,
    ) -> Result<(), crate::devices::NetDeviceError> {
        todo!()
    }

    fn close(&mut self) -> Result<(), crate::devices::NetDeviceError> {
        // nothing to do
        Ok(())
    }
}
