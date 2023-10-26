//! Temporal quantification.

use core::time::Duration;
use core::ops::Sub;

/// A measurement of a monotonically nondecreasing clock.
/// Opaque and useful only with [`Duration`].
#[derive(Clone, Copy)]
pub struct Instant(axhal::time::TimeValue);

impl Instant {
    /// Returns an instant corresponding to "now".
    pub fn now() -> Instant {
        Instant(axhal::time::current_time())
    }

    ///
    /// # Panics
    ///
    /// Previous rust versions panicked when the current time was earlier than self. Currently this
    /// method returns a Duration of zero in that case. Future versions may reintroduce the panic.
    pub fn elapsed(&self) -> Duration {
        Instant::now() - *self
    }

    /// Returns the amount of time elapsed from another instant to this one,
    /// or zero duration if that instant is later than this one.
    ///
    /// # Panics
    ///
    /// Previous rust versions panicked when `earlier` was later than `self`. Currently this
    /// method saturates. Future versions may reintroduce the panic in some circumstances.
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        self.0.checked_sub(earlier.0).unwrap_or_default()
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;

    /// Returns the amount of time elapsed from another instant to this one,
    /// or zero duration if that instant is later than this one.
    fn sub(self, other: Instant) -> Self::Output {
        self.duration_since(other)
    }
}
