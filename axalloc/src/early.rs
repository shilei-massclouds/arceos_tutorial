use core::alloc::Layout;

extern crate alloc;

const fn align_up(val: usize, align: usize) -> usize {
    (val + align - 1) & !(align - 1)
}

pub struct EarlyAllocator {
    start:  usize,
    end:    usize,
    count:  usize,
    next:   usize,
}

impl EarlyAllocator {
    pub const fn new() -> Self {
        Self {
            start:  0,
            end:    0,
            count:  0,
            next:   0,
        }
    }

    pub fn init(&mut self, start: usize, len: usize) {
        self.start = start;
        self.end = start + len;
        self.next = start;
    }

    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let start = align_up(self.next, layout.align());
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
}
