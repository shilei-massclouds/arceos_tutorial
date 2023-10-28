use axconfig::{PAGE_SIZE, align_up, align_down};
use axconfig::virt_to_phys;
use page_table::{PAGE_KERNEL_RO, PAGE_KERNEL_RW, PAGE_KERNEL_RX};

/// A physical memory region.
#[derive(Debug)]
pub struct MemRegion {
    /// The start physical address of the region.
    pub paddr: usize,
    /// The size in bytes of the region.
    pub size: usize,
    /// The region flags.
    pub flags: usize,
    /// The region name, used for identification.
    pub name: &'static str,
}

/// Returns the memory regions of the kernel image (code and data sections).
pub fn kernel_image_regions() -> impl Iterator<Item = MemRegion> {
    [
        MemRegion {
            paddr: virt_to_phys((_stext as usize).into()),
            size: _etext as usize - _stext as usize,
            flags: PAGE_KERNEL_RX,
            name: ".text",
        },
        MemRegion {
            paddr: virt_to_phys((_srodata as usize).into()),
            size: _erodata as usize - _srodata as usize,
            flags: PAGE_KERNEL_RO,
            name: ".rodata",
        },
        MemRegion {
            paddr: virt_to_phys((_sdata as usize).into()),
            size: _edata as usize - _sdata as usize,
            flags: PAGE_KERNEL_RW,
            name: ".data .tdata .tbss .percpu",
        },
        MemRegion {
            paddr: virt_to_phys(_skernel as usize) - 0x100000,
            size: 0x100000,
            flags: PAGE_KERNEL_RW,
            name: "early heap",
        },
        MemRegion {
            paddr: virt_to_phys((boot_stack as usize).into()),
            size: boot_stack_top as usize - boot_stack as usize,
            flags: PAGE_KERNEL_RW,
            name: "boot stack",
        },
        MemRegion {
            paddr: virt_to_phys((_sbss as usize).into()),
            size: _ebss as usize - _sbss as usize,
            flags: PAGE_KERNEL_RW,
            name: ".bss",
        },
    ]
    .into_iter()
}

/// Returns the default free memory regions (kernel image end to physical memory end).
pub fn free_regions(phys_mem_size: usize) -> impl Iterator<Item = MemRegion> {
    let start = align_up(virt_to_phys(_ekernel as usize), PAGE_SIZE);
    let size = _skernel as usize + phys_mem_size - _ekernel as usize;
    core::iter::once(MemRegion {
        paddr: start,
        size: align_down(size, PAGE_SIZE),
        flags: PAGE_KERNEL_RW,
        name: "free memory",
    })
}

extern "C" {
    fn _skernel();
    fn _stext();
    fn _etext();
    fn _srodata();
    fn _erodata();
    fn _sdata();
    fn _edata();
    fn _sbss();
    fn _ebss();
    fn _ekernel();
    fn boot_stack();
    fn boot_stack_top();
}
