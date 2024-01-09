use riscv::register::satp;
use axconfig::SIZE_1G;
use page_table::{PageTable, PAGE_KERNEL_RWX, phys_pfn};

extern "C" {
    fn boot_page_table();
}

pub unsafe fn init_boot_page_table() {
    let mut pt: PageTable = PageTable::init(boot_page_table as usize, 0);
    let _ = pt.map(0x8000_0000, 0x8000_0000, SIZE_1G, SIZE_1G, PAGE_KERNEL_RWX);
    let _ = pt.map(0xffff_ffc0_8000_0000, 0x8000_0000, SIZE_1G, SIZE_1G, PAGE_KERNEL_RWX);
}

pub unsafe fn init_mmu() {
    let page_table_root = boot_page_table as usize;
    satp::set(satp::Mode::Sv39, 0, phys_pfn(page_table_root));
    riscv::asm::sfence_vma_all();
}
