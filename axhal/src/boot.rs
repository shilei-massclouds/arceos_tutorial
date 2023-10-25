#[no_mangle]
#[link_section = ".text.boot"]
unsafe extern "C" fn _start() -> ! {
    // a0 = hartid
    // a1 = dtb
    core::arch::asm!("
        mv      s0, a0                  // save hartid
        mv      s1, a1                  // save DTB pointer

        la      sp, boot_stack_top      // setup boot stack

        call    {init_boot_page_table}
        call    {init_mmu}              // setup boot page table and enabel MMU

        li      s2, {phys_virt_offset}  // fix up virtual high address
        add     sp, sp, s2              // readjust stack address

        mv      a0, s0                  // restore hartid
        mv      a1, s1                  // restore DTB pointer

        la      a2, {entry}
        add     a2, a2, s2              // readjust rust_entry address
        jalr    a2                      // call rust_entry(hartid, dtb)
        j       .",
        init_boot_page_table = sym crate::paging::init_boot_page_table,
        init_mmu = sym crate::paging::init_mmu,
        phys_virt_offset = const axconfig::PHYS_VIRT_OFFSET,
        entry = sym super::rust_entry,
        options(noreturn),
    )
}
