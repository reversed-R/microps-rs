use std::{
    fmt::Debug,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::devices::{NetDevice, NetProtocolType};

mod devices;
mod platform;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum TcpIpError {
    DeviceAlreadyOpened { name: String },
    DeviceAlreadyClosed { name: String },
    DataLongerThanMTU { mtu: u16, len: usize },
}

#[derive(Debug)]
pub struct TcpIpApp {
    terminated: Arc<AtomicBool>,
    devices: Vec<NetDeviceContainer>,
}

impl TcpIpApp {
    pub fn new() -> Result<Self, TcpIpError> {
        Ok(Self {
            terminated: Arc::new(AtomicBool::new(false)),
            devices: Vec::new(),
        })
    }

    pub fn run(self) {
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

    fn register_net_device(&mut self, dev: Box<dyn NetDevice>) {
        let dev = NetDeviceContainer {
            name: format!("net{}", self.devices.len()),
            dev,
        };

        self.devices.push(dev);
    }
}

struct NetDeviceContainer {
    name: String,
    dev: Box<dyn NetDevice>,
}

impl Debug for NetDeviceContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NetDevice{{ name: \"{}\" }}", &self.name)
    }
}

impl NetDeviceContainer {
    fn open(&mut self) -> Result<(), TcpIpError> {
        println!("opening dev={}", &self.name);
        if self.dev.is_open() {
            Err(TcpIpError::DeviceAlreadyOpened {
                name: self.name.clone(),
            })
        } else {
            self.dev.open()?;
            self.dev.set_open_flag();
            Ok(())
        }
    }

    fn close(&mut self) -> Result<(), TcpIpError> {
        println!("closing dev={}", &self.name);
        if self.dev.is_close() {
            Err(TcpIpError::DeviceAlreadyClosed {
                name: self.name.clone(),
            })
        } else {
            self.dev.close()?;
            self.dev.set_close_flag();
            Ok(())
        }
    }

    fn output(&mut self, typ: NetProtocolType, data: &[u8], dst: ()) -> Result<(), TcpIpError> {
        println!("outputing dev={}", &self.name);
        if self.dev.is_close() {
            Err(TcpIpError::DeviceAlreadyClosed {
                name: self.name.clone(),
            })
        } else {
            if (self.dev.mtu() as usize) < data.len() {
                Err(TcpIpError::DataLongerThanMTU {
                    mtu: self.dev.mtu(),
                    len: data.len(),
                })
            } else {
                self.dev.output(typ, data, dst)?;

                Ok(())
            }
        }
    }
}
