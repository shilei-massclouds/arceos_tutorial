#![no_std]

use core::cell::{RefCell, RefMut, OnceCell};

pub struct BootCell<T> {
    inner: RefCell<T>,
}

impl<T> BootCell<T> {
    pub const unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value)
        }
    }

    pub fn access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}

unsafe impl<T> Sync for BootCell<T> {}

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
