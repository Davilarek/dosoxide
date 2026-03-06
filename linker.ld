ENTRY(_start)
SECTIONS {
    . = 0x0;

    to_real_mode_masked = to_real_mode & 0xfff0;
    .text : {
        *(.entry16)
        *(.text32)
        *(.text .text.*)
    }

    .data : {
        *(.data .data.*)
        *(.rodata .rodata.*)
    }

    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    }

    /DISCARD/ : { *(.comment) *(.note*) }
}
