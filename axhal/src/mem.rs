#[doc(no_inline)]
pub use memory_addr::{PhysAddr, VirtAddr, PAGE_SIZE_4K};

/// Fills the `.bss` section with zeros.
pub(crate) fn clear_bss() {
    unsafe {
        core::slice::from_raw_parts_mut(_sbss as usize as *mut u8, _ebss as usize - _sbss as usize)
            .fill(0);
    }
}

/// Converts a physical address to a virtual address.
///
/// It assumes that there is a linear mapping with the offset
/// [`PHYS_VIRT_OFFSET`], that maps all the physical memory to the virtual
/// space at the address plus the offset. So we have
/// `vaddr = paddr + PHYS_VIRT_OFFSET`.
///
/// [`PHYS_VIRT_OFFSET`]: axconfig::PHYS_VIRT_OFFSET
#[inline]
pub const fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    VirtAddr::from(paddr.as_usize() + crate::paging::PHYS_VIRT_OFFSET)
}

extern "C" {
    fn _sbss();
    fn _ebss();
}
