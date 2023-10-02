#![no_std]

use core::cell::{RefCell, RefMut};

// Temporary wrapper. Replace it with Mutex/SpinXXX later!
pub struct BootCell<T> {
    inner: RefCell<T>,
}

impl<T> BootCell<T> {
    pub const unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value)
        }
    }

    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}

unsafe impl<T> Sync for BootCell<T> {}
