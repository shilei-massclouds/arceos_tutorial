#![no_std]

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const PHYS_VIRT_OFFSET: usize = 0xffff_ffc0_0000_0000;
pub const ASPACE_BITS: usize = 39;
pub const TASK_STACK_SIZE: usize = 0x40000; // 256 K

pub const SIZE_1G: usize = 0x4000_0000;
pub const SIZE_2M: usize = 0x20_0000;
pub const SIZE_4K: usize = 0x1000;

//
// common utilities
//

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
pub const fn phys_to_virt(paddr: usize) -> usize {
    paddr.wrapping_add(PHYS_VIRT_OFFSET)
}

#[inline]
pub const fn virt_to_phys(vaddr: usize) -> usize {
    vaddr.wrapping_sub(PHYS_VIRT_OFFSET)
}
