pub(crate) const ETHER_HEADER_SIZE: usize = 14;
pub(crate) const ETHER_FRAME_SIZE_MIN: usize = 60;
pub(crate) const ETHER_FRAME_SIZE_MAX: usize = 1514;
pub(crate) const ETHER_PAYLOAD_SIZE_MIN: usize = ETHER_FRAME_SIZE_MIN - ETHER_HEADER_SIZE;
pub(crate) const ETHER_PAYLOAD_SIZE_MAX: usize = ETHER_FRAME_SIZE_MAX - ETHER_HEADER_SIZE;
