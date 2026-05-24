use std::{
    fmt::Debug,
    sync::{
        Arc, OnceLock, RwLock, RwLockReadGuard,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{
    dbg,
    devices::{DeviceId, NetDevice, NetDeviceError},
    info,
    interfaces::{IfaceFamilyKind, IpIface, NetIface},
    print::debugdump,
    protocols::{
        IP_ADDR_LOOPBACK, IP_ADDR_LOOPBACK_NETMASK, IpAddr, IpProtocol, NetProtocol,
        NetProtocolType,
    },
};

const TEST_DATA: &[u8] = &[
    0x45, 0x00, 0x00, 0x30, 0x00, 0x80, 0x00, 0x00, 0xff, 0x01, 0xbd, 0x4a, 0x7f, 0x00, 0x00, 0x01,
    0x7f, 0x00, 0x00, 0x01, 0x08, 0x00, 0x35, 0x64, 0x00, 0x80, 0x00, 0x01, 0x31, 0x32, 0x33, 0x34,
    0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x21, 0x40, 0x23, 0x24, 0x25, 0x5e, 0x26, 0x2a, 0x28, 0x29,
];

static TCP_IP_APP: OnceLock<Arc<TcpIpApp>> = OnceLock::new();

pub fn tcp_ip_run() -> Result<(), TcpIpError> {
    let mut tcp_ip_app = TcpIpApp::new()?;

    tcp_ip_app.register_net_device(crate::devices::loopback::LoopbackDevice::new);
    tcp_ip_app.register_net_device(crate::platform::linux::driver::ether_tap::EtherTapDevice::new);

    let mut ip_proto = IpProtocol::new();
    ip_proto
        .register_protocol(crate::protocols::ip::IpUpperProtocol::Icmp(
            crate::protocols::ip::icmp::IcmpProtocol,
        ))
        .unwrap();
    tcp_ip_app.register_net_protocol(ip_proto);

    tcp_ip_app.register_net_iface_on_device(
        NetIface::Ip(IpIface::new(IP_ADDR_LOOPBACK, IP_ADDR_LOOPBACK_NETMASK)),
        "net0",
    )?;

    TCP_IP_APP
        .set(Arc::new(tcp_ip_app))
        .map_err(|_| TcpIpError::FaildToInit)?;

    TCP_IP_APP.get().unwrap().run()?;

    Ok(())
}

#[derive(Debug, Clone)]
pub enum TcpIpError {
    FaildToInit,
    DeviceAlreadyOpened {
        name: String,
    },
    DeviceAlreadyClosed {
        name: String,
    },
    DataLongerThanMTU {
        mtu: u16,
        len: usize,
    },
    DuplicatedIfaceFamily {
        family: IfaceFamilyKind,
        dev: String, // device name
    },
    DeviceError {
        error: NetDeviceError,
    },
    DeviceNotFound {
        dev: String,
    },
}

#[derive(Debug)]
pub(crate) struct TcpIpApp {
    terminated: Arc<AtomicBool>,
    devices: Vec<NetDeviceContainer>,
    pub(crate) protocols: Vec<Box<dyn NetProtocol>>,
}

impl TcpIpApp {
    fn new() -> Result<Self, TcpIpError> {
        Ok(Self {
            terminated: Arc::new(AtomicBool::new(false)),
            devices: Vec::new(),
            protocols: Vec::new(),
        })
    }

    fn run(&self) -> Result<(), TcpIpError> {
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&self.terminated))
            .unwrap();

        info!("Starting TCP/IP processing...");
        info!("Press <Ctrl> + C to terminate.");

        for dev in &self.devices {
            dev.open()?;
        }

        let src = IP_ADDR_LOOPBACK;
        let dst = src;

        let id = crate::platform::random16();
        let mut seq = 0u16;
        while !self.terminated.load(Ordering::Relaxed) {
            // // crate::protocols::ip::output(
            // //     crate::protocols::ip::IpUpperProtocolType::Icmp,
            // //     &TEST_DATA[20..],
            // //     src,
            // //     dst,
            // // )
            // // .unwrap();
            //
            // seq += 1;
            // let seq_bytes = seq.to_be_bytes();
            // let id_bytes = id.to_be_bytes();
            // let val = [id_bytes[0], id_bytes[1], seq_bytes[0], seq_bytes[1]];
            //
            // crate::protocols::ip::icmp::output(
            //     crate::protocols::ip::icmp::IcmpType::Echo,
            //     crate::protocols::ip::icmp::IcmpCode::NetUnreach,
            //     val,
            //     &[0x54, 0x45, 0x53, 0x54], // 'T', 'E', 'S', 'T'
            //     src,
            //     dst,
            // )
            // .unwrap();
        }

        // cleanup
        info!("Escape from TCP/IP processing.");

        for dev in &self.devices {
            dev.close()?;
        }

        info!("Cleaning up completed.");

        Ok(())
    }

    fn register_net_device<D: NetDevice, F: Fn(DeviceId) -> D>(&mut self, dev_init: F) {
        let dev_id = DeviceId::new(self.devices.len());
        let dev = dev_init(dev_id);

        let dev = NetDeviceContainer {
            dev: Box::new(dev),
            state: Arc::new(RwLock::new(NetDeviceState {
                name: format!("net{}", self.devices.len()),
                is_open: false,
                ifaces: Vec::new(),
            })),
        };

        self.devices.push(dev);
    }

    fn register_net_protocol<P: NetProtocol>(&mut self, proto: P) {
        self.protocols.push(Box::new(proto));
    }

    fn register_net_iface_on_device(&self, iface: NetIface, dev: &str) -> Result<(), TcpIpError> {
        dbg!("registering iface on dev={}", dev);

        for d in &self.devices {
            if d.name() == dev {
                if d.state()
                    .ifaces
                    .iter()
                    .any(|i| i.family_kind() == iface.family_kind())
                {
                    return Err(TcpIpError::DuplicatedIfaceFamily {
                        family: iface.family_kind(),
                        dev: dev.into(),
                    });
                } else {
                    d.state.write().unwrap().ifaces.push(iface);
                    return Ok(());
                }
            }
        }

        Err(TcpIpError::DeviceNotFound {
            dev: dev.to_string(),
        })
    }
}

#[derive(Debug)]
pub(crate) struct NetDeviceContainer {
    dev: Box<dyn NetDevice>,
    state: Arc<RwLock<NetDeviceState>>,
}

#[derive(Debug)]
pub(crate) struct NetDeviceState {
    name: String,
    is_open: bool,
    ifaces: Vec<NetIface>,
}

impl NetDeviceContainer {
    #[inline]
    fn name(&self) -> String {
        self.state.read().unwrap().name.clone()
    }

    #[inline]
    fn is_open(&self) -> bool {
        self.state.read().unwrap().is_open
    }

    #[inline]
    pub(crate) fn dev(&self) -> &dyn NetDevice {
        &*self.dev
    }

    #[inline]
    pub(crate) fn state(&self) -> RwLockReadGuard<'_, NetDeviceState> {
        self.state.read().unwrap()
    }

    fn open(&self) -> Result<(), TcpIpError> {
        dbg!("opening dev={}", &self.name());
        if self.is_open() {
            Err(TcpIpError::DeviceAlreadyOpened {
                name: self.name().clone(),
            })
        } else {
            self.dev.open()?;
            self.state.write().unwrap().is_open = true;
            Ok(())
        }
    }

    fn close(&self) -> Result<(), TcpIpError> {
        dbg!("closing dev={}", self.name());
        if !self.is_open() {
            Err(TcpIpError::DeviceAlreadyClosed {
                name: self.name().clone(),
            })
        } else {
            self.dev.close()?;
            self.state.write().unwrap().is_open = false;
            Ok(())
        }
    }

    pub(crate) fn output(
        &self,
        typ: NetProtocolType,
        data: &[u8],
        dst: &crate::devices::HardwareAddr<'_>,
    ) -> Result<(), TcpIpError> {
        dbg!("outputing dev={}", self.name());
        if !self.is_open() {
            Err(TcpIpError::DeviceAlreadyClosed {
                name: self.name().clone(),
            })
        } else {
            if (self.dev.info().mtu() as usize) < data.len() {
                Err(TcpIpError::DataLongerThanMTU {
                    mtu: self.dev.info().mtu(),
                    len: data.len(),
                })
            } else {
                self.dev.output(typ, data, dst)?;

                Ok(())
            }
        }
    }
}

impl NetDeviceState {
    #[inline]
    pub(crate) fn name(&self) -> &String {
        &self.name
    }

    #[inline]
    pub(crate) fn is_open(&self) -> bool {
        self.is_open
    }

    #[inline]
    pub(crate) fn ifaces(&self) -> &[NetIface] {
        &self.ifaces
    }
}

pub(crate) fn input_to_app(
    dev_id: DeviceId,
    typ: NetProtocolType,
    data: &[u8],
) -> Result<(), NetDeviceError> {
    dbg!("net_input: type={typ:?}, len={}", data.len());

    debugdump(data);

    let app = TCP_IP_APP.get().unwrap();
    let dev = app.devices.get(dev_id.value()).unwrap();

    for proto in &app.protocols {
        if proto.typ() == typ {
            proto.handle(data, dev)?;

            return Ok(());
        }
    }

    Ok(())
}

pub(crate) fn select_ip_device(addr: &IpAddr) -> Option<(&NetDeviceContainer, IpAddr)> {
    for dev in &TCP_IP_APP.get().unwrap().devices {
        for iface in dev.state().ifaces() {
            match iface {
                NetIface::Ip(iface) => {
                    if iface.unicast() == addr {
                        return Some((dev, *iface.netmask()));
                    }
                }
                NetIface::IpV6 => {
                    todo!();
                }
            }
        }
    }

    None
}
