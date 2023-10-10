//! A naïve spinning mutex.
//!
//! Waiting threads hammer an atomic variable until it becomes available. Best-case latency is low, but worst-case
//! latency is theoretically infinite.
//!
//! Based on [`spin::Mutex`](https://docs.rs/spin/latest/src/spin/mutex/spin.rs.html).

use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

use kernel_guard::BaseGuard;

/// A [spin lock](https://en.m.wikipedia.org/wiki/Spinlock) providing mutually
/// exclusive access to data.
///
/// This is a base struct, the specific behavior depends on the generic
/// parameter `G` that implements [`BaseGuard`], such as whether to disable
/// local IRQs or kernel preemption before acquiring the lock.
///
/// For single-core environment (without the "smp" feature), we remove the lock
/// state, CPU can always get the lock if we follow the proper guard in use.
pub struct BaseSpinLock<G: BaseGuard, T: ?Sized> {
    _phantom: PhantomData<G>,
    data: UnsafeCell<T>,
}

/// A guard that provides mutable data access.
///
/// When the guard falls out of scope it will release the lock.
pub struct BaseSpinLockGuard<'a, G: BaseGuard, T: ?Sized + 'a> {
    _phantom: &'a PhantomData<G>,
    irq_state: G::State,
    data: *mut T,
}

// Same unsafe impls as `std::sync::Mutex`
unsafe impl<G: BaseGuard, T: ?Sized + Send> Sync for BaseSpinLock<G, T> {}
unsafe impl<G: BaseGuard, T: ?Sized + Send> Send for BaseSpinLock<G, T> {}

impl<G: BaseGuard, T> BaseSpinLock<G, T> {
    /// Creates a new [`BaseSpinLock`] wrapping the supplied data.
    #[inline(always)]
    pub const fn new(data: T) -> Self {
        Self {
            _phantom: PhantomData,
            data: UnsafeCell::new(data),
        }
    }

    /// Consumes this [`BaseSpinLock`] and unwraps the underlying data.
    #[inline(always)]
    pub fn into_inner(self) -> T {
        // We know statically that there are no outstanding references to
        // `self` so there's no need to lock.
        let BaseSpinLock { data, .. } = self;
        data.into_inner()
    }
}

impl<G: BaseGuard, T: ?Sized> BaseSpinLock<G, T> {
    /// Locks the [`BaseSpinLock`] and returns a guard that permits access to the inner data.
    ///
    /// The returned value may be dereferenced for data access
    /// and the lock will be dropped when the guard falls out of scope.
    #[inline(always)]
    pub fn lock(&self) -> BaseSpinLockGuard<G, T> {
        let irq_state = G::acquire();
        BaseSpinLockGuard {
            _phantom: &PhantomData,
            irq_state,
            data: unsafe { &mut *self.data.get() },
        }
    }

    /// Returns `true` if the lock is currently held.
    ///
    /// # Safety
    ///
    /// This function provides no synchronization guarantees and so its result should be considered 'out of date'
    /// the instant it is called. Do not use it for synchronization purposes. However, it may be useful as a heuristic.
    #[inline(always)]
    pub fn is_locked(&self) -> bool {
        false
    }

    /// Try to lock this [`BaseSpinLock`], returning a lock guard if successful.
    #[inline(always)]
    pub fn try_lock(&self) -> Option<BaseSpinLockGuard<G, T>> {
        let irq_state = G::acquire();

        Some(BaseSpinLockGuard {
            _phantom: &PhantomData,
            irq_state,
            data: unsafe { &mut *self.data.get() },
        })
    }

    /// Force unlock this [`BaseSpinLock`].
    ///
    /// # Safety
    ///
    /// This is *extremely* unsafe if the lock is not held by the current
    /// thread. However, this can be useful in some instances for exposing the
    /// lock to FFI that doesn't know how to deal with RAII.
    #[inline(always)]
    pub unsafe fn force_unlock(&self) {
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the [`BaseSpinLock`] mutably, and a mutable reference is guaranteed to be exclusive in
    /// Rust, no actual locking needs to take place -- the mutable borrow statically guarantees no locks exist. As
    /// such, this is a 'zero-cost' operation.
    #[inline(always)]
    pub fn get_mut(&mut self) -> &mut T {
        // We know statically that there are no other references to `self`, so
        // there's no need to lock the inner mutex.
        unsafe { &mut *self.data.get() }
    }
}

impl<'a, G: BaseGuard, T: ?Sized> Deref for BaseSpinLockGuard<'a, G, T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        // We know statically that only we are referencing data
        unsafe { &*self.data }
    }
}

impl<'a, G: BaseGuard, T: ?Sized> DerefMut for BaseSpinLockGuard<'a, G, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        // We know statically that only we are referencing data
        unsafe { &mut *self.data }
    }
}

impl<'a, G: BaseGuard, T: ?Sized> Drop for BaseSpinLockGuard<'a, G, T> {
    /// The dropping of the [`BaseSpinLockGuard`] will release the lock it was
    /// created from.
    #[inline(always)]
    fn drop(&mut self) {
        G::release(self.irq_state);
    }
}
