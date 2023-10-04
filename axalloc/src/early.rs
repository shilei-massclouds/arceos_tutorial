//! Earliest memory allocator

use core::alloc::Layout;

extern crate alloc;

#[macro_export]
macro_rules! ROUNDUP {
    ($a: expr, $b: expr) => {((($a) + (($b)-1)) & !(($b)-1))}
}

#[macro_export]
macro_rules! ALIGN {
    ($a: expr, $b: expr) => {ROUNDUP!($a, $b)}
}

pub struct EarlyByteAllocator {
    start:  usize,
    end:    usize,
    next:   usize,
    count:  usize,
}

impl EarlyByteAllocator {
    pub const fn new() -> Self {
        Self {
            start:  0,
            end:    0,
            next:   0,
            count:  0,
        }
    }

    pub fn init(&mut self, start: usize, len: usize) {
        self.start = start;
        self.end = start + len;
        self.next = start;
    }

    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let start = ALIGN!(self.next, layout.align());
        let end = start + layout.size();
        if end > self.end {
            alloc::alloc::handle_alloc_error(layout)
        } else {
            self.next = end;
            self.count += 1;
            start as *mut u8
        }
    }

    pub fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
        self.count -= 1;
        if self.count == 0 {
            self.next = self.start;
        }
    }

    pub fn available_bytes(&self) -> usize {
        self.end - self.next
    }
}
