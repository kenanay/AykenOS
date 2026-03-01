/* Phase 10-A minimal userspace linker script.
 * Entry point: 0x400000
 * Exactly one RX PT_LOAD segment.
 */

OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

PHDRS
{
    text PT_LOAD FLAGS(5); /* PF_R | PF_X */
}

SECTIONS
{
    . = 0x400000;

    .text : ALIGN(16)
    {
        *(.text .text.*)
        *(.rodata .rodata.*)
    } :text

    /DISCARD/ :
    {
        *(.data .data.*)
        *(.bss .bss.*)
        *(COMMON)
        *(.comment)
        *(.eh_frame*)
        *(.note*)
        *(.llvm_addrsig)
    }
}
