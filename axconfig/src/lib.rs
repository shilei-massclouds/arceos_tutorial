#![no_std]

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const PHYS_VIRT_OFFSET: usize = 0xffff_ffc0_0000_0000;
pub const ASPACE_BITS: usize = 39;

pub const SIZE_1G: usize = 0x4000_0000;

#[inline]
pub const fn align_up(val: usize, align: usize) -> usize {
    (val + align - 1) & !(align - 1)
}

#[inline]
pub const fn align_down(val: usize, align: usize) -> usize {
    (val) & !(align - 1)
}
