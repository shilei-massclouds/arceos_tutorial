use core::cell::OnceCell;

pub struct BootOnceCell<T> {
    inner: OnceCell<T>,
}

impl<T> BootOnceCell<T> {
    pub const fn new() -> Self {
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

    pub fn is_init(&self) -> bool {
        self.inner.get().is_some()
    }
}

unsafe impl<T> Sync for BootOnceCell<T> {}
