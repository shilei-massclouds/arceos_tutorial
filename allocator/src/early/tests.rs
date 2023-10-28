use core::alloc::Layout;
use super::EarlyAllocator;
use super::{BaseAllocator, ByteAllocator};
use super::alloc;
use axconfig::PAGE_SIZE;

#[test]
fn test_alloc_bytes() {
    let space: [u8; PAGE_SIZE] = [0; PAGE_SIZE];
    let base = space.as_ptr() as usize;

    let mut early = EarlyAllocator::new();
    early.init(base, PAGE_SIZE);
    assert_eq!(early.total_bytes(), PAGE_SIZE);
    assert_eq!(early.available_bytes(), PAGE_SIZE);
    assert_eq!(early.used_bytes(), 0);

    let layout = Layout::from_size_align(2, 2).unwrap();
    let p0 = early.alloc_bytes(layout).unwrap();
    assert_eq!(p0.as_ptr() as usize - base, 0);
    assert_eq!(early.used_bytes(), 2);

    let layout = Layout::from_size_align(4, 4).unwrap();
    let p1 = early.alloc_bytes(layout).unwrap();
    assert_eq!(p1.as_ptr() as usize - base, 4);
    assert_eq!(early.used_bytes(), 8);

    early.dealloc(p0, Layout::new::<usize>());
    early.dealloc(p1, Layout::new::<usize>());
    assert_eq!(early.total_bytes(), PAGE_SIZE);
    assert_eq!(early.available_bytes(), PAGE_SIZE);
    assert_eq!(early.used_bytes(), 0);
}

#[test]
fn test_alloc_pages() {
    let num_pages = 16;
    let total_size = PAGE_SIZE * num_pages;
    let layout = Layout::from_size_align(total_size, PAGE_SIZE).unwrap();
    let space = unsafe { alloc::alloc::alloc(layout) };
    let start = space as usize;
    let end = start + total_size;

    let mut early = EarlyAllocator::new();
    early.init(start, total_size);
    assert_eq!(early.total_pages(), num_pages);
    assert_eq!(early.available_pages(), num_pages);
    assert_eq!(early.used_pages(), 0);

    let layout = Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).unwrap();
    let p0 = early.alloc_pages(layout).unwrap();
    assert_eq!(p0.as_ptr() as usize, end - PAGE_SIZE);
    assert_eq!(early.used_pages(), 1);

    let layout = Layout::from_size_align(PAGE_SIZE*2, PAGE_SIZE).unwrap();
    let p1 = early.alloc_pages(layout).unwrap();
    assert_eq!(p1.as_ptr() as usize, end - PAGE_SIZE*3);
    assert_eq!(early.used_pages(), 3);
}
