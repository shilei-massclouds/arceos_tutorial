use axconfig::SIZE_2M;
use page_table::{PageTable, PAGE_KERNEL_RWX};

#[test]
fn test_final() {
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
    assert_eq!(pt.entry_at(pt_index).paddr(), 0x8020_a000);
    assert_eq!(pt.entry_at(pt_index).flags(), PAGE_KERNEL_RWX);
}
