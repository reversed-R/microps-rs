use std::ops::{Add, Sub};

#[cfg(feature = "linux-userland")]
pub(crate) mod linux;

pub(crate) fn random16() -> u16 {
    rand::random::<u16>()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TimeSec(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DurationSec(u64);

impl DurationSec {
    pub(crate) const fn new(sec: u64) -> Self {
        Self(sec)
    }
}

impl Add<DurationSec> for TimeSec {
    type Output = TimeSec;

    fn add(self, rhs: DurationSec) -> Self::Output {
        TimeSec(self.0 + rhs.0)
    }
}

impl Sub<DurationSec> for TimeSec {
    type Output = TimeSec;

    fn sub(self, rhs: DurationSec) -> Self::Output {
        TimeSec(self.0 - rhs.0)
    }
}
