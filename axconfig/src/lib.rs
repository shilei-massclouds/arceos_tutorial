#![no_std]

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const PHYS_VIRT_OFFSET: usize = 0xffff_ffc0_0000_0000;
pub const ASPACE_BITS: usize = 39;

pub const SIZE_1G: usize = 0x4000_0000;
pub const SIZE_2M: usize = 0x20_0000;

#[inline]
pub const fn align_up(val: usize, align: usize) -> usize {
    (val + align - 1) & !(align - 1)
}

#[inline]
pub const fn align_down(val: usize, align: usize) -> usize {
    (val) & !(align - 1)
}

#[inline]
pub const fn align_offset(addr: usize, align: usize) -> usize {
    addr & (align - 1)
}

#[inline]
pub const fn is_aligned(addr: usize, align: usize) -> bool {
    align_offset(addr, align) == 0
}

#[inline]
pub const fn phys_to_virt(pa: usize) -> usize {
    pa.wrapping_add(PHYS_VIRT_OFFSET)
}

#[inline]
pub const fn virt_to_phys(va: usize) -> usize {
    va.wrapping_sub(PHYS_VIRT_OFFSET)
}

#[inline]
pub const fn pfn_phys(pfn: usize) -> usize {
    pfn << PAGE_SHIFT
}
