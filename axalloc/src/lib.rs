#![no_std]

extern crate alloc;

mod early;

#[derive(Debug)]
pub enum AllocError {
    InvalidParam,
    MemoryOverlap,
    NoMemory,
    NotAllocated,
}

pub type AllocResult<T = ()> = Result<T, AllocError>;
