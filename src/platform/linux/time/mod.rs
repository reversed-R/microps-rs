use std::time::{SystemTime, UNIX_EPOCH};

use crate::platform::TimeSec;

impl TimeSec {
    pub(crate) fn now() -> Self {
        let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        Self(duration.as_secs())
    }
}
