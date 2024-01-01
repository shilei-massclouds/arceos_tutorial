#![allow(dead_code)]
use core::alloc::Layout;
use core::ptr::NonNull;
use axconfig::{align_up, align_down, PAGE_SIZE};
use crate::{AllocResult, AllocError};

#[derive(Default)]
pub struct EarlyAllocator {
    start:  usize,
    end:    usize,
    count:  usize,
    byte_pos:  usize,
    page_pos:  usize,
}

impl EarlyAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.byte_pos = start;
        self.page_pos = self.end;
    }
}

impl EarlyAllocator {
	fn alloc_bytes(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let start = align_up(self.byte_pos, layout.align());
        let next = start + layout.size();
        if next > self.page_pos {
            alloc::alloc::handle_alloc_error(layout)
        } else {
            self.byte_pos = next;
            self.count += 1;
            NonNull::new(start as *mut u8).ok_or(AllocError::NoMemory)
        }
    }

    fn dealloc_bytes(&mut self, _ptr: NonNull<u8>, _layout: Layout) {
        self.count -= 1;
        if self.count == 0 {
            self.byte_pos = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }
    fn used_bytes(&self) -> usize {
        self.byte_pos - self.start
    }
    fn available_bytes(&self) -> usize {
        self.page_pos - self.byte_pos
    }
}

impl EarlyAllocator {
	fn alloc_pages(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        assert_eq!(layout.size() % PAGE_SIZE, 0);
        let next = align_down(self.page_pos - layout.size(), layout.align());
        if next <= self.byte_pos {
            alloc::alloc::handle_alloc_error(layout)
        } else {
            self.page_pos = next;
            NonNull::new(next as *mut u8).ok_or(AllocError::NoMemory)
        }
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / PAGE_SIZE
    }
    pub fn used_pages(&self) -> usize {
        (self.end - self.page_pos) / PAGE_SIZE
    }
    fn available_pages(&self) -> usize {
        (self.page_pos - self.byte_pos) / PAGE_SIZE
    }
}

#[cfg(test)]
mod tests;
