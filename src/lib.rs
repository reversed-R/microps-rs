use std::{
    fmt::Debug,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::devices::{NetDevice, NetProtocolType};

mod devices;
mod platform;

#[cfg(test)]
mod tests;

static TCP_IP_APP: OnceLock<Arc<TcpIpApp>> = OnceLock::new();

pub fn tcp_ip_run() -> Result<(), TcpIpError> {
    let mut tcp_ip_app = TcpIpApp::new()?;

    let loopback_dev: Arc<dyn NetDevice> = Arc::new(devices::LoopbackDevice::new());
    tcp_ip_app.register_net_device(Arc::clone(&loopback_dev));

    TCP_IP_APP
        .set(Arc::new(tcp_ip_app))
        .map_err(|_| TcpIpError::FaildToInit)?;

    TCP_IP_APP.get().unwrap().run();

    Ok(())
}

#[derive(Debug)]
pub enum TcpIpError {
    FaildToInit,
    DeviceAlreadyOpened { name: String },
    DeviceAlreadyClosed { name: String },
    DataLongerThanMTU { mtu: u16, len: usize },
}

#[derive(Debug)]
struct TcpIpApp {
    terminated: Arc<AtomicBool>,
    devices: Vec<NetDeviceContainer>,
}

impl TcpIpApp {
    fn new() -> Result<Self, TcpIpError> {
        Ok(Self {
            terminated: Arc::new(AtomicBool::new(false)),
            devices: Vec::new(),
        })
    }

    fn run(&self) {
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&self.terminated))
            .unwrap();

        println!("Starting TCP/IP processing...");
        println!("Press <Ctrl> + C to terminate.");
        while !self.terminated.load(Ordering::Relaxed) {
            // todo
        }

        // cleanup
        println!("Escape from TCP/IP processing.");
        println!("Cleaning up completed.");
    }

    fn register_net_device(&mut self, dev: Arc<dyn NetDevice>) {
        let dev = NetDeviceContainer {
            name: format!("net{}", self.devices.len()),
            dev,
            is_open: false,
        };

        self.devices.push(dev);
    }
}

struct NetDeviceContainer {
    name: String,
    dev: Arc<dyn NetDevice>,
    is_open: bool,
}

impl Debug for NetDeviceContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NetDevice{{ name: \"{}\" }}", &self.name)
    }
}

impl NetDeviceContainer {
    fn open(&mut self) -> Result<(), TcpIpError> {
        println!("opening dev={}", &self.name);
        if self.is_open {
            Err(TcpIpError::DeviceAlreadyOpened {
                name: self.name.clone(),
            })
        } else {
            self.dev.open()?;
            self.is_open = true;
            Ok(())
        }
    }

    fn close(&mut self) -> Result<(), TcpIpError> {
        println!("closing dev={}", &self.name);
        if !self.is_open {
            Err(TcpIpError::DeviceAlreadyClosed {
                name: self.name.clone(),
            })
        } else {
            self.dev.close()?;
            self.is_open = false;
            Ok(())
        }
    }

    fn output(&mut self, typ: NetProtocolType, data: &[u8], dst: ()) -> Result<(), TcpIpError> {
        println!("outputing dev={}", &self.name);
        if !self.is_open {
            Err(TcpIpError::DeviceAlreadyClosed {
                name: self.name.clone(),
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
