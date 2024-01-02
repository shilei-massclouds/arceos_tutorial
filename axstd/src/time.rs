use core::time::Duration;
use core::ops::Sub;
use core::fmt;

#[derive(Clone, Copy)]
pub struct Instant(axhal::time::TimeValue);

impl Instant {
    pub fn now() -> Instant {
        Instant(axhal::time::current_time())
    }
    pub fn elapsed(&self) -> Duration {
        Instant::now() - *self
    }
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        self.0.checked_sub(earlier.0).unwrap_or_default()
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;
    fn sub(self, other: Instant) -> Self::Output {
        self.duration_since(other)
    }
}

impl fmt::Display for Instant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{:06}", self.0.as_secs(), self.0.subsec_micros())
    }
}
