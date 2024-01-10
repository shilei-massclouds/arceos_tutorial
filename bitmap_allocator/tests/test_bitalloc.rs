use bitmap_allocator::*;

#[test]
fn bitalloc16() {
    let mut ba = BitAlloc16::default();
    assert_eq!(BitAlloc16::CAP, 16);
    ba.insert(0..16);
    for i in 0..16 {
        assert!(ba.test(i));
    }
    ba.remove(2..8);
    assert_eq!(ba.alloc(), Some(0));
    assert_eq!(ba.alloc(), Some(1));
    assert_eq!(ba.alloc(), Some(8));
    ba.dealloc(0);
    ba.dealloc(1);
    ba.dealloc(8);

    assert!(!ba.is_empty());
    for _ in 0..10 {
        assert!(ba.alloc().is_some());
    }
    assert!(ba.is_empty());
    assert!(ba.alloc().is_none());
}

#[test]
fn bitalloc4k() {
    let mut ba = BitAlloc4K::default();
    assert_eq!(BitAlloc4K::CAP, 4096);
    ba.insert(0..4096);
    for i in 0..4096 {
        assert!(ba.test(i));
    }
    ba.remove(2..4094);
    for i in 0..4096 {
        assert_eq!(ba.test(i), !(2..4094).contains(&i));
    }
    assert_eq!(ba.alloc(), Some(0));
    assert_eq!(ba.alloc(), Some(1));
    assert_eq!(ba.alloc(), Some(4094));
    ba.dealloc(0);
    ba.dealloc(1);
    ba.dealloc(4094);

    assert!(!ba.is_empty());
    for _ in 0..4 {
        assert!(ba.alloc().is_some());
    }
    assert!(ba.is_empty());
    assert!(ba.alloc().is_none());
}

#[test]
fn bitalloc_contiguous() {
    let mut ba0 = BitAlloc16::default();
    ba0.insert(0..BitAlloc16::CAP);
    ba0.remove(3..6);
    assert_eq!(ba0.next(0), Some(0));
    assert_eq!(ba0.alloc_contiguous(1, 1), Some(0));
    assert_eq!(find_contiguous(&ba0, BitAlloc4K::CAP, 2, 0), Some(1));

    let mut ba = BitAlloc4K::default();
    assert_eq!(BitAlloc4K::CAP, 4096);
    ba.insert(0..BitAlloc4K::CAP);
    ba.remove(3..6);
    assert_eq!(ba.next(0), Some(0));
    assert_eq!(ba.alloc_contiguous(1, 1), Some(0));
    assert_eq!(ba.next(0), Some(1));
    assert_eq!(ba.next(1), Some(1));
    assert_eq!(ba.next(2), Some(2));
    assert_eq!(find_contiguous(&ba, BitAlloc4K::CAP, 2, 0), Some(1));
    assert_eq!(ba.alloc_contiguous(2, 0), Some(1));
    assert_eq!(ba.alloc_contiguous(2, 3), Some(8));
    ba.remove(0..4096 - 64);
    assert_eq!(ba.alloc_contiguous(128, 7), None);
    assert_eq!(ba.alloc_contiguous(7, 3), Some(4096 - 64));
    ba.insert(321..323);
    assert_eq!(ba.alloc_contiguous(2, 1), Some(4096 - 64 + 8));
    assert_eq!(ba.alloc_contiguous(2, 0), Some(321));
    assert_eq!(ba.alloc_contiguous(64, 6), None);
    assert_eq!(ba.alloc_contiguous(32, 4), Some(4096 - 48));
    for i in 0..4096 - 64 + 7 {
        ba.dealloc(i);
    }
    for i in 4096 - 64 + 8..4096 - 64 + 10 {
        ba.dealloc(i);
    }
    for i in 4096 - 48..4096 - 16 {
        ba.dealloc(i);
    }
}
