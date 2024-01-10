use core::ptr::NonNull;
use crate::{AllocError, AllocResult};
use axconfig::PAGE_SIZE;
use bitmap_allocator::{BitAlloc1M, BitAlloc};
use alloc::alloc::Layout;

pub struct BitmapPageAllocator {
    base: usize,
    inner: BitAlloc1M,
}

impl BitmapPageAllocator {
    pub const fn new() -> Self {
        Self { base: 0, inner: BitAlloc1M::DEFAULT }
    }
    pub fn init(&mut self, start: usize, size: usize) {
        let end = axconfig::align_down(start + size, PAGE_SIZE);
        let start = axconfig::align_up(start, PAGE_SIZE);
        self.base = start;
        let total_pages = (end - start) / PAGE_SIZE;
        self.inner.insert(0..total_pages);
    }
    pub fn alloc_pages(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        if layout.align() % PAGE_SIZE != 0 {
            return Err(AllocError::InvalidParam);
        }
        let align_pow2 = layout.align() / PAGE_SIZE;
        if !align_pow2.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }
        let num_pages = layout.size() / PAGE_SIZE;
        let align_log2 = align_pow2.trailing_zeros() as usize;
        match num_pages.cmp(&1) {
            core::cmp::Ordering::Equal => self.inner.alloc().map(|idx| idx * PAGE_SIZE + self.base),
            core::cmp::Ordering::Greater => self
                .inner
                .alloc_contiguous(num_pages, align_log2)
                .map(|idx| idx * PAGE_SIZE + self.base),
            _ => return Err(AllocError::InvalidParam),
        }
        .map(|pos| NonNull::new(pos as *mut u8).unwrap())
        .ok_or(AllocError::NoMemory)
    }
    pub fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        let idx = (pos - self.base) / PAGE_SIZE;
        for i in 0..num_pages {
            self.inner.dealloc(idx+i)
        }
    }
}
