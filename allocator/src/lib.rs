//! Various allocator algorithms in a unified interface.

#![no_std]

mod early;

use core::ptr::NonNull;
use core::alloc::Layout;

pub use early::EarlyAllocator;

/// The error type used for allocation.
#[derive(Debug)]
pub enum AllocError {
    /// Invalid `size` or `align_pow2`. (e.g. unaligned)
    InvalidParam,
    /// Memory added by `add_memory` overlapped with existed memory.
    MemoryOverlap,
    /// No enough memory to allocate.
    NoMemory,
    /// Deallocate an unallocated memory region.
    NotAllocated,
}

/// A [`Result`] type with [`AllocError`] as the error type.
pub type AllocResult<T = ()> = Result<T, AllocError>;

/// The base allocator inherited by other allocators.
pub trait BaseAllocator {
    /// Initialize the allocator with a free memory region.
    fn init(&mut self, start: usize, size: usize);

    /// Add a free memory region to the allocator.
    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult;
}

/// Byte-granularity allocator.
pub trait ByteAllocator: BaseAllocator {
    /// Allocate memory with the given size (in bytes) and alignment.
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>>;

    /// Deallocate memory at the given position, size, and alignment.
    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout);

    /// Returns total memory size in bytes.
    fn total_bytes(&self) -> usize;

    /// Returns allocated memory size in bytes.
    fn used_bytes(&self) -> usize;

    /// Returns available memory size in bytes.
    fn available_bytes(&self) -> usize;
}

/// Page-granularity allocator.
pub trait PageAllocator: BaseAllocator {
    /// The size of a memory page.
    const PAGE_SIZE: usize;

    /// Allocate contiguous memory pages with given count and alignment.
    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize>;

    /// Deallocate contiguous memory pages with given position and count.
    fn dealloc_pages(&mut self, pos: usize, num_pages: usize);

    /// Returns the total number of memory pages.
    fn total_pages(&self) -> usize;

    /// Returns the number of allocated memory pages.
    fn used_pages(&self) -> usize;

    /// Returns the number of available memory pages.
    fn available_pages(&self) -> usize;
}
