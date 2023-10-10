//! Earliest memory allocator

use core::alloc::Layout;
use core::ptr::NonNull;
use crate::{AllocResult, AllocError, BaseAllocator};
use crate::{ByteAllocator, PageAllocator};

extern crate alloc;

const PAGE_SIZE: usize = 4096;

#[macro_export]
macro_rules! ROUNDUP {
    ($a: expr, $b: expr) => {((($a) + (($b)-1)) & !(($b)-1))}
}

#[macro_export]
macro_rules! ROUNDDOWN {
    ($a: expr, $b: expr) => {(($a) & !(($b)-1))}
}

#[macro_export]
macro_rules! ALIGN_UP {
    ($a: expr, $b: expr) => {ROUNDUP!($a, $b)}
}

#[macro_export]
macro_rules! ALIGN_DOWN {
    ($a: expr, $b: expr) => {ROUNDDOWN!($a, $b)}
}

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start    bytes_pos    pages_pos      end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator {
    start:  usize,
    end:    usize,
    count:  usize,
    bytes_pos:  usize,
    pages_pos:  usize,
}

impl EarlyAllocator {
    pub const fn new() -> Self {
        Self {
            start:  0,
            end:    0,
            count:  0,
            bytes_pos:  0,
            pages_pos:  0,
        }
    }

    fn alloc_bytes(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        let start = ALIGN_UP!(self.bytes_pos, layout.align());
        let next = start + layout.size();
        if next > self.pages_pos {
            alloc::alloc::handle_alloc_error(layout)
        } else {
            self.bytes_pos = next;
            self.count += 1;
            NonNull::new(start as *mut u8).ok_or(AllocError::NoMemory)
        }
    }

    fn alloc_pages(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        assert_eq!(layout.size() % PAGE_SIZE, 0);
        let next = ALIGN_DOWN!(self.pages_pos - layout.size(), layout.align());
        if next <= self.bytes_pos {
            alloc::alloc::handle_alloc_error(layout)
        } else {
            self.pages_pos = next;
            NonNull::new(next as *mut u8).ok_or(AllocError::NoMemory)
        }
    }
}

impl BaseAllocator for EarlyAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.bytes_pos = start;
        self.pages_pos = self.end;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        todo!();
    }
}

impl ByteAllocator for EarlyAllocator {
    fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
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
            self.bytes_pos = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.bytes_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.pages_pos - self.bytes_pos
    }
}

impl PageAllocator for EarlyAllocator {
    fn alloc_pages(&mut self, _num_pages: usize, _align_pow2: usize) -> AllocResult<usize> {
        todo!();
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        todo!();
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.pages_pos) / PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        (self.pages_pos - self.bytes_pos) / PAGE_SIZE
    }
}
