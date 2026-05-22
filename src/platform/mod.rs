#[cfg(feature = "linux-userland")]
mod linux;

pub(crate) fn random16() -> u16 {
    rand::random::<u16>()
}
