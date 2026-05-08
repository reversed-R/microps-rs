use crate::{
    dbg,
    print::debugdump,
    protocols::{NetProtocol, NetProtocolType},
};

#[derive(Debug, Clone)]
pub(crate) struct IpProtocol;

impl IpProtocol {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl NetProtocol for IpProtocol {
    fn typ(&self) -> super::NetProtocolType {
        NetProtocolType::Ip
    }

    fn handle(&self, data: &[u8], dev: &dyn crate::devices::NetDevice) {
        dbg!("handling ip packet...");
        debugdump(data);
    }
}
