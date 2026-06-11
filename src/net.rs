pub(crate) mod context;

pub(crate) use context::ProtocolStackContext;

use std::{
    fmt::Debug,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{
    data_structure::rcu::RcuCell,
    dbg,
    devices::{DeviceId, NetDevice, NetDeviceError},
    info,
    interfaces::{IfaceFamilyKind, IpIface, NetIface},
    protocols::{
        IP_ADDR_LOOPBACK, IP_ADDR_LOOPBACK_NETMASK, IpAddr, IpProtocol, NetProtocol,
        NetProtocolType, arp::ArpProtocol,
    },
};

// const TEST_DATA: &[u8] = &[
//     0x45, 0x00, 0x00, 0x30, 0x00, 0x80, 0x00, 0x00, 0xff, 0x01, 0xbd, 0x4a, 0x7f, 0x00, 0x00, 0x01,
//     0x7f, 0x00, 0x00, 0x01, 0x08, 0x00, 0x35, 0x64, 0x00, 0x80, 0x00, 0x01, 0x31, 0x32, 0x33, 0x34,
//     0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x21, 0x40, 0x23, 0x24, 0x25, 0x5e, 0x26, 0x2a, 0x28, 0x29,
// ];
//
// static TCP_IP_APP: OnceLock<Arc<ProtocolStackApp>> = OnceLock::new();
//
// pub fn tcp_ip_run() -> Result<(), AppError> {
//     let mut tcp_ip_app = ProtocolStackApp::new()?;
//
//     tcp_ip_app.register_net_device(crate::devices::loopback::LoopbackDevice::new);
//     tcp_ip_app.register_net_device(crate::platform::linux::driver::ether_tap::EtherTapDevice::new);
//
//     let mut ip_proto = IpProtocol::new();
//     ip_proto
//         .register_protocol(crate::protocols::ip::IpUpperProtocol::Icmp(
//             crate::protocols::ip::icmp::IcmpProtocol,
//         ))
//         .unwrap();
//     tcp_ip_app.register_net_protocol(ip_proto);
//     let arp_proto = ArpProtocol::new();
//     tcp_ip_app.register_net_protocol(arp_proto);
//
//     tcp_ip_app.register_net_iface_on_device(
//         NetIface::Ip(IpIface::new(IP_ADDR_LOOPBACK, IP_ADDR_LOOPBACK_NETMASK)),
//         "net0",
//     )?;
//     tcp_ip_app.register_net_iface_on_device(
//         NetIface::Ip(IpIface::new(
//             IpAddr::from([192, 0, 2, 2]),
//             IpAddr::from([255, 255, 255, 0]),
//         )),
//         "net1",
//     )?;
//
//     TCP_IP_APP
//         .set(Arc::new(tcp_ip_app))
//         .map_err(|_| AppError::FaildToInit)?;
//
//     TCP_IP_APP.get().unwrap().run()?;
//
//     Ok(())
// }

#[derive(Debug, Clone)]
pub enum AppError {
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
pub struct ProtocolStackApp {
    terminated: Arc<AtomicBool>,
    devices: Vec<Arc<NetDeviceContainer>>,
    pub(crate) protocols: Vec<Box<dyn NetProtocol>>,
}

impl ProtocolStackApp {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            terminated: Arc::new(AtomicBool::new(false)),
            devices: Vec::new(),
            protocols: Vec::new(),
        })
    }

    pub fn run(self) -> Result<(), AppError> {
        let ctx = ProtocolStackContext::new(self);

        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&ctx.app.terminated))
            .unwrap();

        info!("Starting TCP/IP processing...");
        info!("Press <Ctrl> + C to terminate.");

        for dev in &ctx.app.devices {
            dev.open(ctx.clone())?;
        }

        while !ctx.app.terminated.load(Ordering::Relaxed) {}

        // cleanup
        info!("Escape from TCP/IP processing.");

        for dev in &ctx.app.devices {
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
            name: format!("net{}", self.devices.len()),
            state: NetDeviceState {
                is_open: RcuCell::new(false),
                ifaces: RcuCell::new(Vec::new()),
            },
        };

        self.devices.push(Arc::new(dev));
    }

    fn register_net_protocol<P: NetProtocol>(&mut self, proto: P) {
        self.protocols.push(Box::new(proto));
    }

    fn register_net_iface_on_device(
        &mut self,
        mut iface: NetIface,
        dev: &str,
    ) -> Result<(), AppError> {
        dbg!("registering iface on dev={}", dev);

        for d in &self.devices {
            if d.name() != dev {
                continue;
            }

            if d.state()
                .ifaces
                .load()
                .iter()
                .any(|i| i.family_kind() == iface.family_kind())
            {
                return Err(AppError::DuplicatedIfaceFamily {
                    family: iface.family_kind(),
                    dev: dev.into(),
                });
            }

            iface.set_dev(d);
            d.state.ifaces.update(|ifaces| {
                let mut ifaces = ifaces.clone();
                ifaces.push(iface);

                ifaces
            });
            return Ok(());
        }

        Err(AppError::DeviceNotFound {
            dev: dev.to_string(),
        })
    }

    pub fn setup_mock(mut self) -> Result<Self, AppError> {
        self.register_net_device(crate::devices::loopback::LoopbackDevice::new);
        self.register_net_device(crate::platform::linux::driver::ether_tap::EtherTapDevice::new);

        let mut ip_proto = IpProtocol::new();
        ip_proto
            .register_protocol(crate::protocols::ip::IpUpperProtocol::Icmp(
                crate::protocols::ip::icmp::IcmpProtocol,
            ))
            .unwrap();
        self.register_net_protocol(ip_proto);
        let arp_proto = ArpProtocol::new();
        self.register_net_protocol(arp_proto);

        self.register_net_iface_on_device(
            NetIface::Ip(IpIface::new(IP_ADDR_LOOPBACK, IP_ADDR_LOOPBACK_NETMASK)),
            "net0",
        )?;
        self.register_net_iface_on_device(
            NetIface::Ip(IpIface::new(
                IpAddr::from([192, 0, 2, 2]),
                IpAddr::from([255, 255, 255, 0]),
            )),
            "net1",
        )?;

        Ok(self)
    }
}

#[derive(Debug)]
pub(crate) struct NetDeviceContainer {
    dev: Box<dyn NetDevice>,
    name: String,
    state: NetDeviceState,
}

#[derive(Debug)]
pub(crate) struct NetDeviceState {
    is_open: RcuCell<bool>,
    ifaces: RcuCell<Vec<NetIface>>,
}

impl NetDeviceContainer {
    #[inline]
    fn name(&self) -> String {
        self.name.clone()
    }

    #[inline]
    fn is_open(&self) -> bool {
        self.state.is_open()
    }

    #[inline]
    pub(crate) fn dev(&self) -> &dyn NetDevice {
        &*self.dev
    }

    #[inline]
    pub(crate) fn state(&self) -> &NetDeviceState {
        &self.state
    }

    fn open(&self, ctx: ProtocolStackContext) -> Result<(), AppError> {
        dbg!("opening dev={}", &self.name());
        if self.is_open() {
            Err(AppError::DeviceAlreadyOpened {
                name: self.name().clone(),
            })
        } else {
            self.dev.open(ctx)?;
            self.state.is_open.store(true);
            Ok(())
        }
    }

    fn close(&self) -> Result<(), AppError> {
        dbg!("closing dev={}", self.name());
        if !self.is_open() {
            Err(AppError::DeviceAlreadyClosed {
                name: self.name().clone(),
            })
        } else {
            self.dev.close()?;
            self.state.is_open.store(false);
            Ok(())
        }
    }

    pub(crate) fn output(
        &self,
        ctx: ProtocolStackContext,
        typ: NetProtocolType,
        data: &[u8],
        dst: crate::devices::EthernetAddr,
    ) -> Result<(), AppError> {
        dbg!(
            "outputing dev={}, typ={:?}, dst={:?}",
            self.name(),
            typ,
            dst
        );
        if !self.is_open() {
            Err(AppError::DeviceAlreadyClosed {
                name: self.name().clone(),
            })
        } else {
            let mtu = self.dev().info().mtu();
            if (mtu as usize) < data.len() {
                Err(AppError::DataLongerThanMTU {
                    mtu,
                    len: data.len(),
                })
            } else {
                self.dev().output(ctx, typ, data, dst)?;

                Ok(())
            }
        }
    }
}

impl NetDeviceState {
    #[inline]
    pub(crate) fn is_open(&self) -> bool {
        *self.is_open.load()
    }

    #[inline]
    pub(crate) fn ifaces(&self) -> Arc<Vec<NetIface>> {
        self.ifaces.load()
    }
}
