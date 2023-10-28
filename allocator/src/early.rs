use core::alloc::Layout;
use core::ptr::NonNull;
use crate::{AllocResult, AllocError, BaseAllocator, ByteAllocator};
use axconfig::{PAGE_SIZE, align_up, align_down};

extern crate alloc;

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
    disabled: bool,
}

impl EarlyAllocator {
    pub const fn new() -> Self {
        Self { start: 0, end: 0, count: 0, b_pos: 0, p_pos: 0, disabled: false }
    }

    pub fn alloc_bytes(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let next = align_up(self.b_pos, layout.align());
        let end = next + layout.size();
        if end > self.end {
            alloc::alloc::handle_alloc_error(layout)
        } else {
            self.b_pos = end;
            self.count += 1;
            NonNull::new(next as *mut u8).ok_or(AllocError::NoMemory)
        }
    }

    pub fn alloc_pages(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        assert_eq!(layout.size() % PAGE_SIZE, 0);
        let next = align_down(self.p_pos - layout.size(), layout.align());
        if next <= self.b_pos {
            alloc::alloc::handle_alloc_error(layout)
        } else {
            self.p_pos = next;
            NonNull::new(next as *mut u8).ok_or(AllocError::NoMemory)
        }
    }

    #[cfg(test)]
    fn total_pages(&self) -> usize {
        (self.end - self.start) / PAGE_SIZE
    }
    pub fn used_pages(&self) -> usize {
        (self.end - self.p_pos) / PAGE_SIZE
    }
    #[cfg(test)]
    fn available_pages(&self) -> usize {
        (self.p_pos - self.b_pos) / PAGE_SIZE
    }

    pub fn disable(&mut self) {
        self.disabled = true;
    }

    pub fn disabled(&self) -> bool {
        self.disabled
    }
}

impl BaseAllocator for EarlyAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = self.end;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        unimplemented!();
    }
}

impl ByteAllocator for EarlyAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        if layout.size() % PAGE_SIZE == 0 {
            assert_eq!(layout.align(), PAGE_SIZE);
            self.alloc_pages(layout)
        } else {
            self.alloc_bytes(layout)
        }
    }

    fn dealloc(&mut self, _ptr: NonNull<u8>, _layout: Layout) {
        self.count -= 1;
        if self.count == 0 {
            self.b_pos = self.start;
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
}

#[cfg(test)]
mod tests;
