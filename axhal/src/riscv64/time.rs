//! Time-related operations.

use core::time::Duration;
use riscv::register::time;

/// Timer interrupt frequency in Hz.
const TIMER_FREQUENCY: u64 = 10_000_000;    // 10MHz

/// Number of nanoseconds in a second.
pub const NANOS_PER_SEC: u64 = 1_000_000_000;

const NANOS_PER_TICK: u64 = NANOS_PER_SEC / TIMER_FREQUENCY;

/// A measurement of the system clock.
///
/// Currently, it reuses the [`core::time::Duration`] type. But it does not
/// represent a duration, but a clock time.
pub type TimeValue = Duration;

/// Returns the current clock time in [`TimeValue`].
pub fn current_time() -> TimeValue {
    TimeValue::from_nanos(current_time_nanos())
}

/// Returns the current clock time in nanoseconds.
pub fn current_time_nanos() -> u64 {
    ticks_to_nanos(current_ticks())
}

/// Returns the current clock time in hardware ticks.
#[inline]
pub fn current_ticks() -> u64 {
    time::read() as u64
}

/// Converts hardware ticks to nanoseconds.
#[inline]
pub const fn ticks_to_nanos(ticks: u64) -> u64 {
    ticks * NANOS_PER_TICK
}

/// Converts nanoseconds to hardware ticks.
#[inline]
pub const fn nanos_to_ticks(nanos: u64) -> u64 {
    nanos / NANOS_PER_TICK
}

/// Set a one-shot timer.
///
/// A timer interrupt will be triggered at the given deadline (in nanoseconds).
pub fn set_oneshot_timer(deadline_ns: u64) {
    sbi_rt::set_timer(nanos_to_ticks(deadline_ns));
}

pub(super) fn init_percpu() {
    sbi_rt::set_timer(0);
}
