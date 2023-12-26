#![no_std]

use axconfig::{PAGE_SHIFT, ASPACE_BITS};

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

pub const PAGE_KERNEL: usize =
    _PAGE_V | _PAGE_R | _PAGE_W | _PAGE_G | _PAGE_A | _PAGE_D;
pub const PAGE_KERNEL_EXEC : usize = PAGE_KERNEL | _PAGE_E;

const PAGE_PFN_SHIFT: usize = 10;
const ENTRIES_COUNT: usize = 1 << (PAGE_SHIFT - 3);

pub const fn phys_pfn(pa: usize) -> usize {
    pa >> PAGE_SHIFT
}

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
}

pub struct PageTable<'a> {
    level: usize,
    table: &'a mut [PTEntry],
}

impl PageTable<'_> {
    pub fn init(root_pa: usize) -> Self {
        let table = unsafe {
            core::slice::from_raw_parts_mut(root_pa as *mut PTEntry, ENTRIES_COUNT)
        };
        Self { level: 0, table }
    }
}

impl PageTable<'_> {
    const fn entry_shift(&self) -> usize {
        ASPACE_BITS - (self.level + 1) * (PAGE_SHIFT - 3)
    }
    const fn entry_size(&self) -> usize {
        1 << self.entry_shift()
    }
    const fn entry_index(&self, va: usize) -> usize {
        (va >> self.entry_shift()) & (ENTRIES_COUNT - 1)
    }

	pub fn map(&mut self, mut va: usize, mut pa: usize,
        mut total_size: usize, leaf_size: usize, flags: usize
    ) -> PagingResult {
        let entry_size = self.entry_size();
        while total_size >= entry_size {
            let index = self.entry_index(va);
            if entry_size == leaf_size {
                self.table[index].set(pa, flags);
            } else {
                let pt = self.next_table_mut(index)?;
                pt.map(va, pa, entry_size, leaf_size, flags)?;
            }
            total_size -= entry_size;
            va += entry_size;
            pa += entry_size;
        }
        Ok(())
    }

    fn next_table_mut(&mut self, _index: usize) -> PagingResult<&mut PageTable> {
        unimplemented!();
    }
}
