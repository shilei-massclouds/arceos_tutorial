#![no_std]

use bit_field::BitField;
use core::ops::Range;

pub type BitAlloc1M = BitAllocCascade16<BitAlloc64K>;
pub type BitAlloc64K = BitAllocCascade16<BitAlloc4K>;
pub type BitAlloc4K = BitAllocCascade16<BitAlloc256>;
pub type BitAlloc256 = BitAllocCascade16<BitAlloc16>;

#[derive(Default)]
pub struct BitAllocCascade16<T: BitAlloc> {
    bitset: u16, // for each bit, 1 indicates available, 0 indicates inavailable
    sub: [T; 16],
}

#[derive(Default)]
pub struct BitAlloc16(u16);

pub trait BitAlloc: Default {
    /// The bitmap has a total of CAP bits, numbered from 0 to CAP-1 inclusively.
    const CAP: usize;
    const DEFAULT: Self;
    fn alloc(&mut self) -> Option<usize>;
    fn alloc_contiguous(&mut self, size: usize, align_log2: usize) -> Option<usize>;
    fn next(&self, key: usize) -> Option<usize>;
    fn dealloc(&mut self, key: usize);
    fn insert(&mut self, range: Range<usize>);
    fn remove(&mut self, range: Range<usize>);
    fn is_empty(&self) -> bool;
    fn test(&self, key: usize) -> bool;
}

impl<T: BitAlloc> BitAlloc for BitAllocCascade16<T> {
    const CAP: usize = T::CAP * 16;
    const DEFAULT: Self = BitAllocCascade16 {
        bitset: 0,
        sub: [T::DEFAULT; 16],
    };
    fn alloc(&mut self) -> Option<usize> {
        if !self.is_empty() {
            let i = self.bitset.trailing_zeros() as usize;
            let res = self.sub[i].alloc().unwrap() + i * T::CAP;
            self.bitset.set_bit(i, !self.sub[i].is_empty());
            Some(res)
        } else {
            None
        }
    }
    fn alloc_contiguous(&mut self, size: usize, align_log2: usize) -> Option<usize> {
        if let Some(base) = find_contiguous(self, Self::CAP, size, align_log2) {
            self.remove(base..base + size);
            Some(base)
        } else {
            None
        }
    }
    fn dealloc(&mut self, key: usize) {
        let i = key / T::CAP;
        self.sub[i].dealloc(key % T::CAP);
        self.bitset.set_bit(i, true);
    }
    fn insert(&mut self, range: Range<usize>) {
        self.for_range(range, |sub: &mut T, range| sub.insert(range));
    }
    fn remove(&mut self, range: Range<usize>) {
        self.for_range(range, |sub: &mut T, range| sub.remove(range));
    }
    fn is_empty(&self) -> bool {
        self.bitset == 0
    }
    fn test(&self, key: usize) -> bool {
        self.sub[key / T::CAP].test(key % T::CAP)
    }
    fn next(&self, key: usize) -> Option<usize> {
        let idx = key / T::CAP;
        (idx..16).find_map(|i| {
            if self.bitset.get_bit(i) {
                let key = if i == idx { key - T::CAP * idx } else { 0 };
                self.sub[i].next(key).map(|x| x + T::CAP * i)
            } else {
                None
            }
        })
    }
}

impl<T: BitAlloc> BitAllocCascade16<T> {
    fn for_range(&mut self, range: Range<usize>, f: impl Fn(&mut T, Range<usize>)) {
        let Range { start, end } = range;
        assert!(start <= end);
        assert!(end <= Self::CAP);
        for i in start / T::CAP..=(end - 1) / T::CAP {
            let begin = if start / T::CAP == i { start % T::CAP } else { 0 };
            let end = if end / T::CAP == i { end % T::CAP } else { T::CAP };
            f(&mut self.sub[i], begin..end);
            self.bitset.set_bit(i, !self.sub[i].is_empty());
        }
    }
}

impl BitAlloc for BitAlloc16 {
    const CAP: usize = u16::BITS as usize;
    const DEFAULT: Self = Self(0);
    fn alloc(&mut self) -> Option<usize> {
        let i = self.0.trailing_zeros() as usize;
        if i < Self::CAP {
            self.0.set_bit(i, false);
            Some(i)
        } else {
            None
        }
    }
    fn alloc_contiguous(&mut self, size: usize, align_log2: usize) -> Option<usize> {
        if let Some(base) = find_contiguous(self, Self::CAP, size, align_log2) {
            self.remove(base..base + size);
            Some(base)
        } else {
            None
        }
    }
    fn dealloc(&mut self, key: usize) {
        assert!(!self.test(key));
        self.0.set_bit(key, true);
    }
    fn insert(&mut self, range: Range<usize>) {
        self.0.set_bits(range.clone(), 0xffff.get_bits(range));
    }
    fn remove(&mut self, range: Range<usize>) {
        self.0.set_bits(range, 0);
    }
    fn is_empty(&self) -> bool {
        self.0 == 0
    }
    fn test(&self, key: usize) -> bool {
        self.0.get_bit(key)
    }
    fn next(&self, key: usize) -> Option<usize> {
        (key..Self::CAP).find(|&i| self.0.get_bit(i))
    }
}

pub fn find_contiguous(
    ba: &impl BitAlloc,
    capacity: usize,
    size: usize,
    align_log2: usize,
) -> Option<usize> {
    if capacity < (1 << align_log2) || ba.is_empty() {
        return None;
    }
    let mut base = 0;
    let mut offset = base;
    while offset < capacity {
        if let Some(next) = ba.next(offset) {
            if next != offset {
                // it can be guarenteed that no bit in (offset..next) is free
                // move to next aligned position after next-1
                assert!(next > offset);
                base = (((next - 1) >> align_log2) + 1) << align_log2;
                assert_ne!(offset, next);
                offset = base;
                continue;
            }
        } else {
            return None;
        }
        offset += 1;
        if offset - base == size {
            return Some(base);
        }
    }
    None
}
