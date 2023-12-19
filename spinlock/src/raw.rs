use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

pub struct SpinRaw<T> {
    data: UnsafeCell<T>,
}

pub struct SpinRawGuard<T> {
    data: *mut T,
}

unsafe impl<T> Sync for SpinRaw<T> {}
unsafe impl<T> Send for SpinRaw<T> {}

impl<T> SpinRaw<T> {
    #[inline(always)]
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }

    #[inline(always)]
    pub fn lock(&self) -> SpinRawGuard<T> {
        SpinRawGuard {
            data: unsafe { &mut *self.data.get() },
        }
    }
}

impl<T> Deref for SpinRawGuard<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { &*self.data }
    }
}

impl<T> DerefMut for SpinRawGuard<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data }
    }
}
