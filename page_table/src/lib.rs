#![no_std]

extern crate alloc;
extern crate log;

use core::cmp::min;
use alloc::alloc::Layout;
use axconfig::{PAGE_SHIFT, ASPACE_BITS, PAGE_SIZE};
use axconfig::{virt_to_phys, phys_to_virt, is_aligned, align_offset};

/*
 * RiscV64 PTE format:
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

const PAGE_TABLE: usize = _PAGE_PRESENT;

pub const PAGE_KERNEL_RO: usize =
    _PAGE_PRESENT | _PAGE_READ |
    _PAGE_GLOBAL | _PAGE_ACCESSED | _PAGE_DIRTY;

pub const PAGE_KERNEL_RW: usize = PAGE_KERNEL_RO | _PAGE_WRITE;

pub const PAGE_KERNEL_RX: usize = PAGE_KERNEL_RO | _PAGE_EXEC;

pub const PAGE_KERNEL_RWX : usize = PAGE_KERNEL_RW | _PAGE_EXEC;

const ENTRIES_COUNT: usize = 1 << (PAGE_SHIFT - 3);

pub const fn phys_pfn(pa: usize) -> usize {
    pa >> PAGE_SHIFT
}

pub const fn pfn_phys(pfn: usize) -> usize {
    pfn << PAGE_SHIFT
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

    fn paddr(&self) -> usize {
        pfn_phys(self.0 as usize >> PAGE_PFN_SHIFT)
    }

    #[cfg(test)]
    fn flags(&self) -> usize {
        self.0 as usize & ((1 << PAGE_PFN_SHIFT) - 1)
    }

    fn is_unused(&self) -> bool {
        self.0 == 0
    }
    fn is_present(&self) -> bool {
        (self.0 as usize & _PAGE_PRESENT) == _PAGE_PRESENT
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

    pub fn alloc_table(level: usize) -> Self {
        let layout = Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).unwrap();
        let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) };
        Self::init(ptr as usize, level)
    }

    pub fn root_paddr(&self) -> usize {
        virt_to_phys(self.table.as_ptr() as usize)
    }

    pub fn map(&mut self, mut va: usize, mut pa: usize,
        mut total_size: usize, leaf_size: usize, flags: usize
    ) -> PagingResult {
        let mut map_size = leaf_size;
        if total_size < leaf_size {
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
        self.map_aligned(va, pa, total_size, map_size, flags)
    }

    fn map_aligned(&mut self, mut va: usize, mut pa: usize,
        mut total_size: usize, leaf_size: usize, flags: usize
    ) -> PagingResult {
        assert!(is_aligned(va, leaf_size));
        assert!(is_aligned(pa, leaf_size));
        assert!(is_aligned(total_size, leaf_size));

        let entry_size = self.entry_size();
        let next_size = min(entry_size, total_size);
        while total_size >= next_size {
            let index = self.entry_index(va);
            if entry_size == leaf_size {
                self.table[index].set(pa, flags);
            } else {
                let mut pt = self.next_table_mut(index)?;
                pt.map(va, pa, next_size, leaf_size, flags)?;
            }
            total_size -= next_size;
            va += next_size;
            pa += next_size;
        }
        Ok(())
    }

    const fn entry_shift(&self) -> usize {
        ASPACE_BITS - (self.level + 1) * (PAGE_SHIFT - 3)
    }
    const fn entry_size(&self) -> usize {
        1 << self.entry_shift()
    }
    const fn entry_index(&self, va: usize) -> usize {
        (va >> self.entry_shift()) & (ENTRIES_COUNT - 1)
    }

    fn next_table(&self, index: usize) -> PagingResult<PageTable> {
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
}

#[cfg(test)]
mod tests;
