#[cfg(feature = "linux-userland")]
pub(crate) mod linux;

pub(crate) fn random16() -> u16 {
    rand::random::<u16>()
}
