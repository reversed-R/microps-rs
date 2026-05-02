use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

mod platform;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum TcpIpError {}

#[derive(Debug)]
pub struct TcpIpApp {
    terminated: Arc<AtomicBool>,
}

impl TcpIpApp {
    pub fn new() -> Result<Self, TcpIpError> {
        Ok(Self {
            terminated: Arc::new(AtomicBool::new(false)),
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
}
