/* ================================================================
   AykenOS Linker Script (x86_64, Higher-Half Kernel)
   ================================================================ */

ENTRY(kmain)

KERNEL_PHYS_BASE = 0x00100000;
KERNEL_VIRT_BASE = 0xFFFFFFFF80000000;
PHYS_TO_VIRT_OFFSET = KERNEL_VIRT_BASE - KERNEL_PHYS_BASE;

SECTIONS
{
    /* Fiziksel adres */
    . = KERNEL_PHYS_BASE;

    /* Sanal adres = fiziksel + offset */
    . = . + PHYS_TO_VIRT_OFFSET;

    /* ------------------------- TEXT -------------------------- */
    .text ALIGN(0x1000) :
        AT(ADDR(.text) - KERNEL_VIRT_BASE + KERNEL_PHYS_BASE)
    {
        _text_start = .;
        KEEP(*(.text.boot))
        *(.text*)
        _text_end = .;
    }

    /* ------------------------ RODATA ------------------------- */
    .rodata ALIGN(0x1000) :
        AT(ADDR(.rodata) - KERNEL_VIRT_BASE + KERNEL_PHYS_BASE)
    {
        _rodata_start = .;
        *(.rodata*)
        _rodata_end = .;
    }

    /* ------------------------- DATA -------------------------- */
    .data ALIGN(0x1000) :
        AT(ADDR(.data) - KERNEL_VIRT_BASE + KERNEL_PHYS_BASE)
    {
        _data_start = .;
        *(.data*)
        _data_end = .;
    }

    /* ------------------------- BSS --------------------------- */
    .bss ALIGN(0x1000) :
        AT(ADDR(.bss) - KERNEL_VIRT_BASE + KERNEL_PHYS_BASE)
    {
        _bss_start = .;
        *(COMMON)
        *(.bss*)
        _bss_end = .;
    }

    /* ------------------------- CPU tables -------------------- */
    .cpu ALIGN(0x1000) :
        AT(ADDR(.cpu) - KERNEL_VIRT_BASE + KERNEL_PHYS_BASE)
    {
        _cpu_start = .;
        *(.gdt*)
        *(.idt*)
        *(.cpu*)
        _cpu_end = .;
    }

    /* Kernel sonu */
    _kernel_end = .;
}

PROVIDE(kernel_phys_start = KERNEL_PHYS_BASE);
PROVIDE(kernel_virt_start = KERNEL_VIRT_BASE);
PROVIDE(kernel_end        = _kernel_end);
PROVIDE(_heap_start       = _kernel_end);
