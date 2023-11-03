#![no_std]

use core::cell::OnceCell;

pub struct BootOnceCell<T> {
    inner: OnceCell<T>,
}

impl<T> BootOnceCell<T> {
    pub const unsafe fn new() -> Self {
        Self {
            inner: OnceCell::new()
        }
    }

    pub fn init(&self, val: T) {
        let _ = self.inner.set(val);
    }

    pub fn get(&self) -> &T {
        self.inner.get().unwrap()
    }
}

unsafe impl<T> Sync for BootOnceCell<T> {}
