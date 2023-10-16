use page_table::{PageTable, PAGE_KERNEL_EXEC};
use axconfig::SIZE_1G;

#[test]
fn test_early_pt() {
    let boot_pt: [u64; 512] = [0; 512];

    let mut pt: PageTable = PageTable::init(boot_pt.as_ptr() as usize, 0);
    let _ = pt.map(0x8000_0000, 0x8000_0000, SIZE_1G, SIZE_1G, PAGE_KERNEL_EXEC);
    let _ = pt.map(0xffff_ffc0_8000_0000, 0x8000_0000, SIZE_1G, SIZE_1G, PAGE_KERNEL_EXEC);
    assert_eq!(boot_pt[2], 0x200000ef, "pgd[2] = {:#x}", boot_pt[2]);
    assert_eq!(boot_pt[0x102], 0x200000ef, "pgd[2] = {:#x}", boot_pt[0x102]);
}
