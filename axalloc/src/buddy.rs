use core::ptr::NonNull;
use crate::{Layout, AllocResult, AllocError};
use buddy_allocator::Heap;

pub struct BuddyByteAllocator {
    inner: Heap<32>,
}

impl BuddyByteAllocator {
    pub const fn new() -> Self {
        Self {
            inner: Heap::<32>::new(),
        }
    }
}

impl BuddyByteAllocator {
    pub fn init(&mut self, start: usize, size: usize) {
        unsafe { self.inner.init(start, size) };
    }
}

impl BuddyByteAllocator {
    pub fn alloc_bytes(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        self.inner.alloc(layout).map_err(|_| AllocError::NoMemory)
    }

    pub fn dealloc_bytes(&mut self, pos: NonNull<u8>, layout: Layout) {
        self.inner.dealloc(pos, layout)
    }
}
