use std::{
    fmt::Debug,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{
    dbg,
    devices::{NetDevice, NetDeviceError},
    info,
    interfaces::{IfaceFamilyKind, IpIface, NetIface},
    print::debugdump,
    protocols::{
        IP_ADDR_LOOPBACK, IP_ADDR_LOOPBACK_NETMASK, IpProtocol, NetProtocol, NetProtocolType,
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

    let loopback_dev: Arc<dyn NetDevice> = Arc::new(crate::devices::LoopbackDevice::new());
    tcp_ip_app.register_net_device(Arc::clone(&loopback_dev));

    let ip_proto = IpProtocol::new();
    tcp_ip_app.register_net_protocol(ip_proto);

    tcp_ip_app.register_net_iface_on_device(
        NetIface::Ip(IpIface::new(IP_ADDR_LOOPBACK, IP_ADDR_LOOPBACK_NETMASK)),
        "net0",
    )?;

    println!("tcp_ip_app: {tcp_ip_app:#?}");

    TCP_IP_APP
        .set(Arc::new(tcp_ip_app))
        .map_err(|_| TcpIpError::FaildToInit)?;

    TCP_IP_APP.get().unwrap().run()?;

    Ok(())
}

#[derive(Debug)]
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
    devices: Vec<Arc<Mutex<NetDeviceContainer>>>,
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
            dev.lock().unwrap().open()?;
        }

        while !self.terminated.load(Ordering::Relaxed) {
            for dev in &self.devices {
                dev.lock()
                    .unwrap()
                    .output(NetProtocolType::Ip, TEST_DATA, ())?;
            }
        }

        // cleanup
        info!("Escape from TCP/IP processing.");

        for dev in &self.devices {
            dev.lock().unwrap().close()?;
        }

        info!("Cleaning up completed.");

        Ok(())
    }

    fn register_net_device(&mut self, dev: Arc<dyn NetDevice>) {
        let dev = Arc::new(Mutex::new(NetDeviceContainer {
            dev,
            state: NetDeviceState {
                name: format!("net{}", self.devices.len()),
                is_open: false,
                ifaces: Vec::new(),
            },
        }));

        self.devices.push(dev);
    }

    fn register_net_protocol<P: NetProtocol>(&mut self, proto: P) {
        self.protocols.push(Box::new(proto));
    }

    fn register_net_iface_on_device(&self, iface: NetIface, dev: &str) -> Result<(), TcpIpError> {
        dbg!("registering iface on dev={}", dev);

        for d in &self.devices {
            let mut d = d.try_lock().unwrap();
            if d.name() == dev {
                if d.ifaces()
                    .iter()
                    .any(|i| i.family_kind() == iface.family_kind())
                {
                    return Err(TcpIpError::DuplicatedIfaceFamily {
                        family: iface.family_kind(),
                        dev: dev.into(),
                    });
                } else {
                    d.state.ifaces.push(iface);
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
    dev: Arc<dyn NetDevice>,
    state: NetDeviceState,
}

#[derive(Debug)]
struct NetDeviceState {
    name: String,
    is_open: bool,
    ifaces: Vec<NetIface>,
}

impl NetDeviceContainer {
    #[inline]
    fn name(&self) -> &String {
        &self.state.name
    }

    #[inline]
    fn is_open(&self) -> bool {
        self.state.is_open
    }

    #[inline]
    pub(crate) fn ifaces(&self) -> &[NetIface] {
        &self.state.ifaces
    }

    fn open(&mut self) -> Result<(), TcpIpError> {
        dbg!("opening dev={}", &self.name());
        if self.is_open() {
            Err(TcpIpError::DeviceAlreadyOpened {
                name: self.name().clone(),
            })
        } else {
            self.dev.open()?;
            self.state.is_open = true;
            Ok(())
        }
    }

    fn close(&mut self) -> Result<(), TcpIpError> {
        dbg!("closing dev={}", self.name());
        if !self.is_open() {
            Err(TcpIpError::DeviceAlreadyClosed {
                name: self.name().clone(),
            })
        } else {
            self.dev.close()?;
            self.state.is_open = false;
            Ok(())
        }
    }

    fn output(&self, typ: NetProtocolType, data: &[u8], dst: ()) -> Result<(), TcpIpError> {
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
                self.dev.output(typ, data, dst, self)?;

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
    typ: NetProtocolType,
    data: &[u8],
    dev: &NetDeviceContainer,
) -> Result<(), NetDeviceError> {
    dbg!("net_input: type={typ:?}, len={}", data.len());

    debugdump(data);

    for proto in &TCP_IP_APP.get().unwrap().protocols {
        if proto.typ() == typ {
            proto.handle(data, dev)?;

            return Ok(());
        }
    }

    Ok(())
}
