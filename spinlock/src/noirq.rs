use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use kernel_guard::{BaseGuard, NoPreemptIrqSave};

pub struct SpinNoIrq<T> {
    data: UnsafeCell<T>,
}

pub struct SpinNoIrqGuard<T> {
    irq_state: usize,
    data: *mut T,
}

unsafe impl<T> Sync for SpinNoIrq<T> {}
unsafe impl<T> Send for SpinNoIrq<T> {}

impl<T> SpinNoIrq<T> {
    #[inline(always)]
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }

    #[inline(always)]
    pub fn into_inner(self) -> T {
        let SpinNoIrq { data, .. } = self;
        data.into_inner()
    }
}

impl<T> SpinNoIrq<T> {
    #[inline(always)]
    pub fn lock(&self) -> SpinNoIrqGuard<T> {
        let irq_state = NoPreemptIrqSave::acquire();
        SpinNoIrqGuard {
            irq_state,
            data: unsafe { &mut *self.data.get() },
        }
    }
}

impl<T> Deref for SpinNoIrqGuard<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { &*self.data }
    }
}

impl<T> DerefMut for SpinNoIrqGuard<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data }
    }
}

impl<T> Drop for SpinNoIrqGuard<T> {
    #[inline(always)]
    fn drop(&mut self) {
        NoPreemptIrqSave::release(self.irq_state);
    }
}
