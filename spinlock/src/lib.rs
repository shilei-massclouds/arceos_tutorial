//! `no_std` spin lock implementation that can disable kernel local IRQs or
//! preemption while locking.

#![no_std]

mod base;

use kernel_guard::NoPreemptIrqSave;

pub use self::base::BaseSpinLock;

/// A spin lock that disables kernel preemption and local IRQs while trying to
/// lock, and re-enables it after unlocking.
///
/// It can be used in the IRQ-enabled context.
pub type SpinNoIrq<T> = BaseSpinLock<NoPreemptIrqSave, T>;
