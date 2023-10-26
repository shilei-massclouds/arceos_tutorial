#![allow(dead_code)]
use core::alloc::Layout;
use axconfig::PAGE_SIZE;

extern crate alloc;

const fn align_up(val: usize, align: usize) -> usize {
    (val + align - 1) & !(align - 1)
}

const fn align_down(val: usize, align: usize) -> usize {
    (val) & !(align - 1)
}

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator {
    start:  usize,
    end:    usize,
    count:  usize,
    b_pos:  usize,
    p_pos:  usize,
}

impl EarlyAllocator {
    pub const fn new() -> Self {
        Self { start: 0, end: 0, count: 0, b_pos: 0, p_pos: 0 }
    }

    pub fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = self.end;
    }

    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        if layout.size() % PAGE_SIZE == 0 {
            assert_eq!(layout.align(), PAGE_SIZE);
            self.alloc_pages(layout)
        } else {
            self.alloc_bytes(layout)
        }
    }

    pub fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
        self.count -= 1;
        if self.count == 0 {
            self.b_pos = self.start;
        }
    }

    pub fn alloc_bytes(&mut self, layout: Layout) -> *mut u8 {
        let next = align_up(self.b_pos, layout.align());
        let end = next + layout.size();
        if end > self.end {
            alloc::alloc::handle_alloc_error(layout)
        } else {
            self.b_pos = end;
            self.count += 1;
            next as *mut u8
        }
    }

    pub fn alloc_pages(&mut self, layout: Layout) -> *mut u8 {
        assert_eq!(layout.size() % PAGE_SIZE, 0);
        let next = align_down(self.p_pos - layout.size(), layout.align());
        if next <= self.b_pos {
            alloc::alloc::handle_alloc_error(layout)
        } else {
            self.p_pos = next;
            next as *mut u8
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }
    fn used_bytes(&self) -> usize {
        self.b_pos - self.start
    }
    fn available_bytes(&self) -> usize {
        self.p_pos - self.b_pos
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / PAGE_SIZE
    }
    fn used_pages(&self) -> usize {
        (self.end - self.p_pos) / PAGE_SIZE
    }
    fn available_pages(&self) -> usize {
        (self.p_pos - self.b_pos) / PAGE_SIZE
    }
}

#[cfg(test)]
mod tests;
