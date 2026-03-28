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

    .ring3_transition ALIGN(0x1000) :
        AT(ADDR(.ring3_transition) - KERNEL_VIRT_BASE + KERNEL_PHYS_BASE)
    {
        _ring3_transition_start = .;
        KEEP(*(.ring3_transition))
        . = ALIGN(0x1000);
        _ring3_transition_end = .;
    }

    .ring3_trampoline ALIGN(0x1000) :
        AT(ADDR(.ring3_trampoline) - KERNEL_VIRT_BASE + KERNEL_PHYS_BASE)
    {
        _ring3_trampoline_start = .;
        KEEP(*(.ring3_trampoline))
        . = ALIGN(0x1000);
        _ring3_trampoline_end = .;
    }

    .ring3_iret_trampoline ALIGN(0x1000) :
        AT(ADDR(.ring3_iret_trampoline) - KERNEL_VIRT_BASE + KERNEL_PHYS_BASE)
    {
        _ring3_iret_trampoline_start = .;
        KEEP(*(.ring3_iret_trampoline))
        . = ALIGN(0x1000);
        _ring3_iret_trampoline_end = .;
    }

    .ring3_alt_bridge_trampoline ALIGN(0x1000) :
        AT(ADDR(.ring3_alt_bridge_trampoline) - KERNEL_VIRT_BASE + KERNEL_PHYS_BASE)
    {
        _ring3_alt_bridge_trampoline_start = .;
        KEEP(*(.ring3_alt_bridge_trampoline))
        . = ALIGN(0x1000);
        _ring3_alt_bridge_trampoline_end = .;
    }

    .ring3_final_iret_trampoline ALIGN(0x1000) :
        AT(ADDR(.ring3_final_iret_trampoline) - KERNEL_VIRT_BASE + KERNEL_PHYS_BASE)
    {
        _ring3_final_iret_trampoline_start = .;
        KEEP(*(.ring3_final_iret_trampoline))
        . = ALIGN(0x1000);
        _ring3_final_iret_trampoline_end = .;
    }

    .ring3_transition_probe ALIGN(0x1000) :
        AT(ADDR(.ring3_transition_probe) - KERNEL_VIRT_BASE + KERNEL_PHYS_BASE)
    {
        _ring3_transition_probe_start = .;
        KEEP(*(.ring3_transition_probe))
        . = ALIGN(0x1000);
        _ring3_transition_probe_end = .;
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
        . = ALIGN(16);
        _tss_rsp0_stack_bottom = .;
        . += 0x4000;
        _tss_rsp0_stack_top = .;
        . = ALIGN(16);
        _tss_ist1_stack_bottom = .;
        . += 0x4000;
        _tss_ist1_stack_top = .;
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

    ASSERT(SIZEOF(.ring3_transition) <= 0x1000,
           "ring3_transition exceeds one page")
    ASSERT(SIZEOF(.ring3_trampoline) <= 0x1000,
           "ring3_trampoline exceeds one page")
    ASSERT(SIZEOF(.ring3_iret_trampoline) <= 0x1000,
           "ring3_iret_trampoline exceeds one page")
    ASSERT(SIZEOF(.ring3_alt_bridge_trampoline) <= 0x1000,
           "ring3_alt_bridge_trampoline exceeds one page")
    ASSERT(SIZEOF(.ring3_final_iret_trampoline) <= 0x1000,
           "ring3_final_iret_trampoline exceeds one page")
    ASSERT(SIZEOF(.ring3_transition_probe) <= 0x1000,
           "ring3_transition_probe exceeds one page")
}

PROVIDE(kernel_phys_start = KERNEL_PHYS_BASE);
PROVIDE(kernel_virt_start = KERNEL_VIRT_BASE);
PROVIDE(kernel_end        = _kernel_end);
PROVIDE(_heap_start       = _kernel_end);
