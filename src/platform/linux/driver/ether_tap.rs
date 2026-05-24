use std::{fs::OpenOptions, io::Read, os::fd::AsRawFd};

use crate::{
    dbg,
    devices::{self, DeviceId, NetDevice, NetDeviceInner, NetDeviceType},
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
        }
    }
}

impl NetDevice for EtherTapDevice {
    fn info(&self) -> &NetDeviceInner {
        &self.inner
    }

    fn open(&self) -> Result<(), crate::devices::NetDeviceError> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/net/tun")
            .unwrap();

        let mut ifreq = IfReq {
            name: [0; _],
            flags: (libc::IFF_TAP | libc::IFF_NO_PI) as libc::c_short,
            __pading: [0; _],
        };

        let name = b"tap0";
        for (i, c) in name.iter().enumerate() {
            ifreq.name[i] = *c as libc::c_char;
        }

        if unsafe { libc::ioctl(file.as_raw_fd(), libc::TUNSETIFF, &ifreq) } == -1 {
            return Err(crate::devices::NetDeviceError::EtherTapOpenFailed);
        }

        let dev_id = self.inner.dev_id();
        std::thread::spawn(move || {
            let mut buf = [0u8; devices::ethernet::ETHER_FRAME_SIZE_MAX];
            loop {
                match file.read(&mut buf) {
                    Ok(n) => {
                        dbg!("read from ether tap device: n={}", n);
                        debugdump(&buf[..n]);

                        // crate::net::input_to_app(
                        //     dev_id,
                        //     crate::protocols::NetProtocolType::Ip,
                        //     &buf[..n],
                        // )
                        // .unwrap();
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
        dst: &crate::devices::HardwareAddr<'_>,
    ) -> Result<(), crate::devices::NetDeviceError> {
        todo!()
    }

    fn close(&self) -> Result<(), crate::devices::NetDeviceError> {
        // nothing to do
        Ok(())
    }
}
