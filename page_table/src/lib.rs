#![no_std]

use core::cmp::min;
use core::alloc::Layout;
use axconfig::{PAGE_SHIFT, PAGE_SIZE, ASPACE_BITS};
use axconfig::{is_aligned, align_offset, align_down};
use axconfig::{phys_to_virt, virt_to_phys, pfn_phys};

extern crate alloc;

/*
 * RiscV64 PTE format:
 * | XLEN-1  10 | 9             8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0
 *       PFN      reserved for SW   D   A   G   U   X   W   R   V
 */
const _PAGE_V : usize = 1 << 0;     /* Valid */
const _PAGE_R : usize = 1 << 1;     /* Readable */
const _PAGE_W : usize = 1 << 2;     /* Writable */
const _PAGE_E : usize = 1 << 3;     /* Executable */
const _PAGE_U : usize = 1 << 4;     /* User */
const _PAGE_G : usize = 1 << 5;     /* Global */
const _PAGE_A : usize = 1 << 6;     /* Accessed (set by hardware) */
const _PAGE_D : usize = 1 << 7;     /* Dirty (set by hardware)*/

const PAGE_TABLE: usize = _PAGE_V;

pub const PAGE_KERNEL_RO: usize =
    _PAGE_V | _PAGE_R | _PAGE_G | _PAGE_A | _PAGE_D;

pub const PAGE_KERNEL_RW: usize = PAGE_KERNEL_RO | _PAGE_W;
pub const PAGE_KERNEL_RX: usize = PAGE_KERNEL_RO | _PAGE_E;
pub const PAGE_KERNEL_RWX: usize = PAGE_KERNEL_RW | _PAGE_E;

const PAGE_PFN_SHIFT: usize = 10;
const ENTRIES_COUNT: usize = 1 << (PAGE_SHIFT - 3);

pub const fn phys_pfn(pa: usize) -> usize {
    pa >> PAGE_SHIFT
}

#[derive(Debug)]
pub enum PagingError {}
pub type PagingResult<T = ()> = Result<T, PagingError>;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PTEntry(u64);

impl PTEntry {
    pub fn set(&mut self, pa: usize, flags: usize) {
        self.0 = Self::make(phys_pfn(pa), flags);
    }
    fn make(pfn: usize, prot: usize) -> u64 {
        ((pfn << PAGE_PFN_SHIFT) | prot) as u64
    }
    fn is_present(&self) -> bool {
        (self.0 as usize & _PAGE_V) == _PAGE_V
    }
    fn is_unused(&self) -> bool {
        self.0 == 0
    }
    pub fn paddr(&self) -> usize {
        pfn_phys(self.0 as usize >> PAGE_PFN_SHIFT)
    }
    pub fn flags(&self) -> usize {
        self.0 as usize & ((1 << PAGE_PFN_SHIFT) - 1)
    }
}

pub struct PageTable<'a> {
    level: usize,
    table: &'a mut [PTEntry],
}

impl PageTable<'_> {
    pub fn init(root_pa: usize, level: usize) -> Self {
        let table = unsafe {
            core::slice::from_raw_parts_mut(root_pa as *mut PTEntry, ENTRIES_COUNT)
        };
        Self { level, table }
    }

    pub fn entry_at(&self, index: usize) -> PTEntry {
        self.table[index]
    }
}

impl PageTable<'_> {
    const fn entry_shift(&self) -> usize {
        ASPACE_BITS - (self.level + 1) * (PAGE_SHIFT - 3)
    }
    const fn entry_size(&self) -> usize {
        1 << self.entry_shift()
    }
    pub const fn entry_index(&self, va: usize) -> usize {
        (va >> self.entry_shift()) & (ENTRIES_COUNT - 1)
    }

    pub fn map(&mut self, mut va: usize, mut pa: usize,
        mut total_size: usize, best_size: usize, flags: usize
    ) -> PagingResult {
        let mut map_size = best_size;
        if total_size < best_size {
            map_size = PAGE_SIZE;
        }

        let offset = align_offset(va, map_size);
        if offset != 0 {
            assert!(map_size != PAGE_SIZE);
            let offset = map_size - offset;
            self.map_aligned(va, pa, offset, PAGE_SIZE, flags)?;
            va += offset;
            pa += offset;
            total_size -= offset;
        }

        let aligned_total_size = align_down(total_size, map_size);
        total_size -= aligned_total_size;

        let ret = self.map_aligned(va, pa, aligned_total_size, map_size, flags)?;

        if total_size != 0 {
            va += aligned_total_size;
            pa += aligned_total_size;
            self.map_aligned(va, pa, total_size, PAGE_SIZE, flags)
        } else {
            Ok(ret)
        }
    }

	fn map_aligned(&mut self, mut va: usize, mut pa: usize,
        mut total_size: usize, best_size: usize, flags: usize
    ) -> PagingResult {
        assert!(is_aligned(va, best_size));
        assert!(is_aligned(pa, best_size));
        assert!(is_aligned(total_size, best_size));
        let entry_size = self.entry_size();
        let next_size = min(entry_size, total_size);
        while total_size >= next_size {
            let index = self.entry_index(va);
            if entry_size == best_size {
                self.table[index].set(pa, flags);
            } else {
                let mut pt = self.next_table_mut(index)?;
                pt.map(va, pa, next_size, best_size, flags)?;
            }
            total_size -= next_size;
            va += next_size;
            pa += next_size;
        }
        Ok(())
    }

    pub fn next_table(&self, index: usize) -> PagingResult<PageTable> {
        assert!(self.table[index].is_present());
        let pa = self.table[index].paddr();
        let va = phys_to_virt(pa);
        Ok(Self::init(va, self.level + 1))
    }

    fn next_table_mut(&mut self, index: usize) -> PagingResult<PageTable> {
        if self.table[index].is_unused() {
            let table = Self::alloc_table(self.level + 1);
            self.table[index].set(table.root_paddr(), PAGE_TABLE);
            Ok(table)
        } else {
            self.next_table(index)
        }
    }
    pub fn alloc_table(level: usize) -> Self {
        let layout = Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).unwrap();
        let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) };
        Self::init(ptr as usize, level)
    }
    pub fn root_paddr(&self) -> usize {
        virt_to_phys(self.table.as_ptr() as usize)
    }
}
