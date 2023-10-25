OUTPUT_ARCH(riscv)

BASE_ADDRESS = 0xffffffc080200000;

ENTRY(_start)
SECTIONS
{
    . = BASE_ADDRESS;
    _skernel = .;

    .text : ALIGN(4K) {
        _stext = .;
        *(.text.boot)
        *(.text .text.*)
        . = ALIGN(4K);
        _etext = .;
    }

    .rodata : ALIGN(4K) {
        _srodata = .;
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
        *(.sdata2 .sdata2.*)
        . = ALIGN(4K);
        _erodata = .;
    }

    .data : ALIGN(4K) {
        _sdata = .;
        *(.data .data.*)
        *(.sdata .sdata.*)
        *(.got .got.*)
    }

    . = ALIGN(4K);
    _edata = .;

    .bss : ALIGN(4K) {
        boot_stack = .;
        . += 256K;
        boot_stack_top = .;

        _sbss = .;

        boot_page_table = .;
        . += 4K;
        boot_page_table_end = .;

        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        . = ALIGN(4K);
        _ebss = .;
    }

    _ekernel = .;

	/DISCARD/ : {
        *(.comment) *(.gnu*) *(.note*) *(.eh_frame*)
    }
}
