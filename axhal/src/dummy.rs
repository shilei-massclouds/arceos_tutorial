pub mod console {
    /// Write a slice of bytes to the console.
    pub fn write_bytes(_bytes: &[u8]) {
        unimplemented!()
    }
}

pub mod time {
    use core::time::Duration;
    pub type TimeValue = Duration;
    pub fn current_time() -> TimeValue {
        unimplemented!()
    }
}

pub mod misc {
    /// Shutdown the whole system, including all CPUs.
    pub fn terminate() -> ! {
        unimplemented!()
    }
}

pub mod mem {
    pub const fn phys_to_virt(_pa: usize) -> usize {
        0
    }
}
