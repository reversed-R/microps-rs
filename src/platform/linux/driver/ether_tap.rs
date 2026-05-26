use std::{fs::OpenOptions, io::Read, os::fd::AsRawFd};

use crate::{
    dbg,
    devices::{
        self, DeviceId, NetDevice, NetDeviceAddr, NetDeviceInner, NetDeviceType,
        ethernet::{
            ETHER_ADDR_ANY, ETHER_ADDR_BROADCAST, ETHER_ADDR_SIZE, ETHER_FRAME_SIZE_MAX,
            ETHER_HEADER_SIZE, ETHER_PAYLOAD_SIZE_MAX, ETHER_PAYLOAD_SIZE_MIN, EthernetAddr,
            EthernetHeader,
        },
    },
    error,
    print::debugdump,
};

#[derive(Debug)]
pub struct EtherTapDevice {
    inner: NetDeviceInner,
    tap_file: libc::c_int,
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
                NetDeviceAddr::Ethernet(ETHER_ADDR_ANY),
                NetDeviceAddr::Ethernet(ETHER_ADDR_BROADCAST),
            ),
            tap_file: -1,
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

        self.tap_file = file.as_raw_fd();

        // get hardware address
        if let NetDeviceAddr::Ethernet(ETHER_ADDR_ANY) = self.inner.addr() {
            let raw_addr = unsafe { ifreq.ifr_ifru.ifru_hwaddr.sa_data };
            let mut addr = [0u8; ETHER_ADDR_SIZE];
            for (i, b) in raw_addr[..ETHER_ADDR_SIZE].iter().enumerate() {
                addr[i] = *b as u8;
            }
            let addr = EthernetAddr::new(addr);
            dbg!("ether tap addr={:?}", addr);
            self.inner.set_addr(NetDeviceAddr::Ethernet(addr));
        }
        let addr = match self.inner.addr() {
            NetDeviceAddr::Ethernet(addr) => *addr,
            _ => panic!(),
        };

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
        dst: crate::devices::EthernetAddr,
    ) -> Result<(), crate::devices::NetDeviceError> {
        dbg!("outputing from ether tap");
        let addr = match self.inner.addr() {
            NetDeviceAddr::Ethernet(addr) => *addr,
            _ => panic!(),
        };

        let hdr = EthernetHeader::new(dst, addr, typ.into());
        let hdr_bytes: [u8; ETHER_HEADER_SIZE] = unsafe { core::mem::transmute(hdr) };

        let mut buf = [0u8; ETHER_FRAME_SIZE_MAX];
        buf[..ETHER_HEADER_SIZE].copy_from_slice(&hdr_bytes);

        let data_len = data.len();
        if data_len > ETHER_PAYLOAD_SIZE_MAX {
            return Err(crate::devices::NetDeviceError::OutOfPayloadSize { size: data_len });
        }
        buf[ETHER_HEADER_SIZE..ETHER_HEADER_SIZE + data_len].copy_from_slice(data);
        let payload_len = if data_len < ETHER_PAYLOAD_SIZE_MIN {
            ETHER_PAYLOAD_SIZE_MIN
        } else {
            data_len
        };

        if unsafe {
            libc::write(
                self.tap_file,
                buf.as_mut_ptr() as *mut libc::c_void,
                ETHER_HEADER_SIZE + payload_len,
            )
        } == -1
        {
            error!("failed to write to tap device");
        }

        Ok(())
    }

    fn close(&mut self) -> Result<(), crate::devices::NetDeviceError> {
        // nothing to do
        Ok(())
    }
}
