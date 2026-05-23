use crate::{dbg, protocols::ip::IpUpperProtocolHandler};

#[derive(Debug, Clone)]
pub(crate) struct IcmpProtocol;

impl IpUpperProtocolHandler for IcmpProtocol {
    fn protocol(&self) -> &super::IpUpperProtocolType {
        &super::IpUpperProtocolType::Icmp
    }

    fn handle(
        &self,
        hdr: super::IpHeader,
        payload: &[u8],
        iface: &crate::interfaces::IpIface,
    ) -> Result<(), super::IpProtocolError> {
        dbg!("ICMP handling...");

        Ok(())
    }
}
