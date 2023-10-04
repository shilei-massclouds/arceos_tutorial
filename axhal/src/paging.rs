use core::alloc::Layout;
use riscv::register::satp;
use memory_addr::{VirtAddr, PhysAddr, PAGE_SIZE_4K};
use crate::mem::{virt_to_phys, phys_to_virt, MemRegionFlags};
use page_table::PagingIf;

#[doc(no_inline)]
pub use page_table::{MappingFlags, PageSize, PagingError, PagingResult};

extern crate alloc;

pub type PageTable = page_table::riscv::Sv39PageTable<PagingIfImpl>;

const PAGE_SHIFT : usize = 12;
const PT_ENTRIES: usize = 1 << (PAGE_SHIFT - 3);

/*
 * PTE format:
 * | XLEN-1  10 | 9             8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0
 *       PFN      reserved for SW   D   A   G   U   X   W   R   V
 */

const PAGE_PFN_SHIFT: usize = 10;

const _PAGE_PRESENT : usize = 1 << 0;     /* Valid */
const _PAGE_READ    : usize = 1 << 1;     /* Readable */
const _PAGE_WRITE   : usize = 1 << 2;     /* Writable */
const _PAGE_EXEC    : usize = 1 << 3;     /* Executable */
const _PAGE_USER    : usize = 1 << 4;     /* User */
const _PAGE_GLOBAL  : usize = 1 << 5;     /* Global */
const _PAGE_ACCESSED: usize = 1 << 6;     /* Accessed (set by hardware) */
const _PAGE_DIRTY   : usize = 1 << 7;     /* Dirty (set by hardware)*/

const PAGE_KERNEL: usize =
    _PAGE_PRESENT | _PAGE_READ | _PAGE_WRITE |
    _PAGE_GLOBAL | _PAGE_ACCESSED | _PAGE_DIRTY;

const PAGE_KERNEL_EXEC : usize = PAGE_KERNEL | _PAGE_EXEC;

pub const PHYS_VIRT_OFFSET: usize = 0xffff_ffc0_0000_0000;

const SV39_BITS: usize = 39;
const ASPACE_BITS: usize = SV39_BITS;

const PGD_SHIFT: usize = ASPACE_BITS - (PAGE_SHIFT - 3);

macro_rules! phys_pfn {
    ($pa: expr) => {
        $pa >> PAGE_SHIFT
    }
}

macro_rules! pgd_index {
    ($va: expr) => {
        ($va >> PGD_SHIFT) & (PT_ENTRIES - 1)
    }
}

macro_rules! pgd_entry {
    ($pfn: expr, $prot: expr) => {
        (($pfn << PAGE_PFN_SHIFT) | $prot) as u64
    }
}

#[link_section = ".data.boot_page_table"]
static mut BOOT_PT_SV39: [u64; PT_ENTRIES] = [0; PT_ENTRIES];

pub unsafe fn init_boot_page_table() {
    let entry = pgd_entry!(phys_pfn!(0x8000_0000), PAGE_KERNEL_EXEC);

    // 0x8000_0000..0xc000_0000, VRWX_GAD, 1G block
    BOOT_PT_SV39[pgd_index!(0x8000_0000)] = entry;
    // 0xffff_ffc0_8000_0000..0xffff_ffc0_c000_0000, VRWX_GAD, 1G block
    BOOT_PT_SV39[pgd_index!(0xffff_ffc0_8000_0000)] = entry;
}

pub unsafe fn init_mmu() {
    let page_table_root = BOOT_PT_SV39.as_ptr() as usize;
    satp::set(satp::Mode::Sv39, 0, phys_pfn!(page_table_root));
    riscv::asm::sfence_vma_all();
}

/// Implementation of [`PagingIf`], to provide physical memory manipulation to
/// the [page_table] crate.
pub struct PagingIfImpl;

impl PagingIf for PagingIfImpl {
    fn alloc_frame() -> Option<PhysAddr> {
        unsafe {
            let layout =
                Layout::from_size_align_unchecked(PAGE_SIZE_4K, PAGE_SIZE_4K);
            let va = alloc::alloc::alloc_zeroed(layout) as usize;
            Some(virt_to_phys(va.into()))
        }
    }

    fn dealloc_frame(paddr: PhysAddr) {
        unsafe {
            let layout = Layout::from_size_align_unchecked(
                PAGE_SIZE_4K, PAGE_SIZE_4K
            );
            alloc::alloc::dealloc(
                phys_to_virt(paddr).as_usize() as *mut u8, layout
            )
        }
    }

    #[inline]
    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
        phys_to_virt(paddr)
    }
}

impl From<MemRegionFlags> for MappingFlags {
    fn from(f: MemRegionFlags) -> Self {
        let mut ret = Self::empty();
        if f.contains(MemRegionFlags::READ) {
            ret |= Self::READ;
        }
        if f.contains(MemRegionFlags::WRITE) {
            ret |= Self::WRITE;
        }
        if f.contains(MemRegionFlags::EXECUTE) {
            ret |= Self::EXECUTE;
        }
        if f.contains(MemRegionFlags::DEVICE) {
            ret |= Self::DEVICE;
        }
        if f.contains(MemRegionFlags::UNCACHED) {
            ret |= Self::UNCACHED;
        }
        ret
    }
}

/// Writes the register to update the current page table root.
///
/// # Safety
///
/// This function is unsafe as it changes the virtual memory address space.
pub unsafe fn write_page_table_root(root_paddr: PhysAddr) {
    let old_root = read_page_table_root();
    if old_root != root_paddr {
        satp::set(satp::Mode::Sv39, 0, root_paddr.as_usize() >> 12);
        riscv::asm::sfence_vma_all();
    }
}

/// Reads the register that stores the current page table root.
///
/// Returns the physical address of the page table root.
#[inline]
pub fn read_page_table_root() -> PhysAddr {
    PhysAddr::from(satp::read().ppn() << 12)
}
