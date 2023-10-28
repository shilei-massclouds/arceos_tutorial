use super::{PageTable, PAGE_KERNEL_RWX};
use axconfig::{SIZE_1G, SIZE_2M};

#[test]
fn test_early_pt() {
    let boot_pt: [u64; 512] = [0; 512];

    let mut pt: PageTable = PageTable::init(boot_pt.as_ptr() as usize, 0);
    let _ = pt.map(0x8000_0000, 0x8000_0000, SIZE_1G, SIZE_1G, PAGE_KERNEL_RWX);
    let _ = pt.map(0xffff_ffc0_8000_0000, 0x8000_0000, SIZE_1G, SIZE_1G, PAGE_KERNEL_RWX);
    assert_eq!(boot_pt[2], 0x200000ef, "pgd[2] = {:#x}", boot_pt[2]);
    assert_eq!(boot_pt[0x102], 0x200000ef, "pgd[2] = {:#x}", boot_pt[0x102]);
}

#[test]
fn test_final_pt() {
    let final_pgd: [u64; 512] = [0; 512];

    let mut pgd: PageTable = PageTable::init(final_pgd.as_ptr() as usize, 0);
    let _ = pgd.map(0xffff_ffc0_8020_a000, 0x8020_a000, 0x7df6000, SIZE_2M, PAGE_KERNEL_RWX);
    let pgd_index = pgd.entry_index(0xffff_ffc0_8020_a000);
    assert_eq!(pgd_index, 258);
    let pmd = pgd.next_table(pgd_index).unwrap();
    let pmd_index = pmd.entry_index(0xffff_ffc0_8020_a000);
    assert_eq!(pmd_index, 1);
    let pt = pmd.next_table(pmd_index).unwrap();
    let pt_index = pt.entry_index(0xffff_ffc0_8020_a000);
    assert_eq!(pt_index, 10);
    assert_eq!(pt.table[pt_index].paddr(), 0x8020_a000);
    assert_eq!(pt.table[pt_index].flags(), PAGE_KERNEL_RWX);
}
