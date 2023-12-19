use spinlock::SpinRaw;

struct Inner {
    val: usize,
}

impl Inner {
    const fn new() -> Self {
        Self { val: 0 }
    }

    fn set(&mut self, v: usize) {
        self.val = v;
    }
    fn get(&self) -> usize {
        self.val
    }
}

static SPIN: SpinRaw<Inner> = SpinRaw::new(Inner::new());

#[test]
fn test_lock() {
    SPIN.lock().set(1);
    assert_eq!(SPIN.lock().get(), 1);
}
