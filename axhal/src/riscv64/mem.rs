#[inline]
pub const fn phys_to_virt(paddr: usize) -> usize {
    paddr + axconfig::PHYS_VIRT_OFFSET
}
