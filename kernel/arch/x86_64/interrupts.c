#include <stdint.h>
#include "interrupts.h"
#include "gdt_idt.h"
#include "port_io.h"
#include "../../include/ayken.h"
#include "../../include/mm.h"
#include "../../sched/sched.h"

struct idt_entry idt_table[256];
struct idt_ptr idt_descriptor;
volatile uint32_t phase10_ring3_user_code_seen = 0;

// Early debugcon output (QEMU port 0xE9) — use macros to avoid calls in ISRs
#define OUTC(ch) do { \
    uint8_t __oc = (uint8_t)(ch); \
    __asm__ volatile("outb %0, $0xE9" : : "a"(__oc)); \
} while (0)

#define DUMP_HEX64(v) do { \
    static const char *_hex = "0123456789ABCDEF"; \
    uint64_t __v = (uint64_t)(v); \
    for (int _i = 15; _i >= 0; --_i) { \
        uint8_t __hc = (uint8_t)_hex[(__v >> (_i * 4)) & 0xF]; \
        OUTC(__hc); \
    } \
} while (0)

#define DUMP_HEX16(v) do { \
    static const char *_hex = "0123456789ABCDEF"; \
    uint16_t __v = (uint16_t)(v); \
    for (int _i = 3; _i >= 0; --_i) { \
        uint8_t __hc = (uint8_t)_hex[(__v >> (_i * 4)) & 0xF]; \
        OUTC(__hc); \
    } \
} while (0)

#define HALT_FOREVER() do { \
    for (;;) __asm__ volatile("cli; hlt"); \
} while (0)

#define ISR_SAFE_HELPER __attribute__((no_caller_saved_registers, target("general-regs-only")))

typedef struct pf_walk_snapshot {
    uint64_t root_phys;
    uint64_t va;
    uint64_t pml4_table_phys;
    uint64_t pml4e_phys;
    uint64_t pml4e;
    uint64_t pdpt_table_phys;
    uint64_t pdpte_phys;
    uint64_t pdpte;
    uint64_t pd_table_phys;
    uint64_t pde_phys;
    uint64_t pde;
    uint64_t pt_table_phys;
    uint64_t pte_phys;
    uint64_t pte;
    uint64_t final_phys;
    uint8_t valid;
} pf_walk_snapshot_t;

static void ISR_SAFE_HELPER pf_walk_emit_text(const char *text)
{
    while (text && *text) {
        OUTC(*text++);
    }
}

static void ISR_SAFE_HELPER pf_walk_emit_bool(uint8_t value)
{
    OUTC(value ? '1' : '0');
}

static uint8_t ISR_SAFE_HELPER pf_walk_reserved_suspect(uint64_t entry)
{
    const uint64_t allowed =
        AYKEN_PTE_ADDR_MASK |
        AYKEN_PTE_PRESENT |
        AYKEN_PTE_WRITABLE |
        AYKEN_PTE_USER |
        AYKEN_PTE_GLOBAL |
        AYKEN_PTE_NO_EXEC |
        (1ULL << 3) |
        (1ULL << 4) |
        (1ULL << 5) |
        (1ULL << 6) |
        (1ULL << 7);

    return (uint8_t)((entry & ~allowed) != 0);
}

static uint8_t ISR_SAFE_HELPER pf_walk_exec_ok(uint64_t entry)
{
    if ((entry & AYKEN_PTE_PRESENT) == 0) {
        return 0;
    }
    if ((entry & AYKEN_PTE_NO_EXEC) != 0) {
        return 0;
    }
    return (uint8_t)(pf_walk_reserved_suspect(entry) == 0);
}

static int ISR_SAFE_HELPER pf_capture_walk_snapshot(uint64_t root_phys,
                                                    uint64_t va,
                                                    pf_walk_snapshot_t *out)
{
    uint64_t active_cr3;
    uint64_t kernel_cr3;
    uint64_t *pml4;
    uint16_t pml4_i;
    uint16_t pdpt_i;
    uint16_t pd_i;
    uint16_t pt_i;

    if (!out || !root_phys) {
        return 0;
    }

    *out = (pf_walk_snapshot_t){0};
    out->root_phys = root_phys & AYKEN_PTE_ADDR_MASK;
    out->va = va;
    out->pml4_table_phys = out->root_phys;

    __asm__ volatile("mov %%cr3, %0" : "=r"(active_cr3));
    kernel_cr3 = paging_get_kernel_pml4_phys() & AYKEN_PTE_ADDR_MASK;
    if ((active_cr3 & AYKEN_PTE_ADDR_MASK) != kernel_cr3) {
        return 0;
    }

    pml4 = (uint64_t *)paging_phys_to_virt(out->root_phys);
    if (!pml4) {
        return 0;
    }

    pml4_i = (uint16_t)((va >> 39) & 0x1FF);
    pdpt_i = (uint16_t)((va >> 30) & 0x1FF);
    pd_i = (uint16_t)((va >> 21) & 0x1FF);
    pt_i = (uint16_t)((va >> 12) & 0x1FF);

    out->pml4e_phys = out->pml4_table_phys + ((uint64_t)pml4_i * sizeof(uint64_t));
    out->pml4e = pml4[pml4_i];
    if ((out->pml4e & AYKEN_PTE_PRESENT) == 0) {
        return 0;
    }

    out->pdpt_table_phys = out->pml4e & AYKEN_PTE_ADDR_MASK;
    {
        uint64_t *pdpt = (uint64_t *)paging_phys_to_virt(out->pdpt_table_phys);
        if (!pdpt) {
            return 0;
        }
        out->pdpte_phys = out->pdpt_table_phys + ((uint64_t)pdpt_i * sizeof(uint64_t));
        out->pdpte = pdpt[pdpt_i];
        if ((out->pdpte & AYKEN_PTE_PRESENT) == 0) {
            return 0;
        }
        if (out->pdpte & (1ULL << 7)) {
            out->final_phys = (out->pdpte & AYKEN_PTE_ADDR_MASK) | (va & ((1ULL << 30) - 1));
            out->valid = 1;
            return 1;
        }
    }

    out->pd_table_phys = out->pdpte & AYKEN_PTE_ADDR_MASK;
    {
        uint64_t *pd = (uint64_t *)paging_phys_to_virt(out->pd_table_phys);
        if (!pd) {
            return 0;
        }
        out->pde_phys = out->pd_table_phys + ((uint64_t)pd_i * sizeof(uint64_t));
        out->pde = pd[pd_i];
        if ((out->pde & AYKEN_PTE_PRESENT) == 0) {
            return 0;
        }
        if (out->pde & (1ULL << 7)) {
            out->final_phys = (out->pde & AYKEN_PTE_ADDR_MASK) | (va & ((1ULL << 21) - 1));
            out->valid = 1;
            return 1;
        }
    }

    out->pt_table_phys = out->pde & AYKEN_PTE_ADDR_MASK;
    {
        uint64_t *pt = (uint64_t *)paging_phys_to_virt(out->pt_table_phys);
        if (!pt) {
            return 0;
        }
        out->pte_phys = out->pt_table_phys + ((uint64_t)pt_i * sizeof(uint64_t));
        out->pte = pt[pt_i];
        if ((out->pte & AYKEN_PTE_PRESENT) == 0) {
            return 0;
        }
    }

    out->final_phys = (out->pte & AYKEN_PTE_ADDR_MASK) | (va & (AYKEN_FRAME_SIZE - 1));
    out->valid = 1;
    return 1;
}

static void ISR_SAFE_HELPER pf_emit_walk_snapshot_line(const char *tag, const pf_walk_snapshot_t *snap)
{
    if (!tag || !snap) {
        return;
    }

    OUTC('W');
    pf_walk_emit_text(tag);
    pf_walk_emit_text(" OK=");
    pf_walk_emit_bool(snap->valid);
    pf_walk_emit_text(" R=");
    DUMP_HEX64(snap->root_phys);
    pf_walk_emit_text(" V=");
    DUMP_HEX64(snap->va);
    pf_walk_emit_text(" 4T=");
    DUMP_HEX64(snap->pml4_table_phys);
    pf_walk_emit_text(" 4A=");
    DUMP_HEX64(snap->pml4e_phys);
    pf_walk_emit_text(" 4E=");
    DUMP_HEX64(snap->pml4e);
    pf_walk_emit_text(" 3T=");
    DUMP_HEX64(snap->pdpt_table_phys);
    pf_walk_emit_text(" 3A=");
    DUMP_HEX64(snap->pdpte_phys);
    pf_walk_emit_text(" 3E=");
    DUMP_HEX64(snap->pdpte);
    pf_walk_emit_text(" 2T=");
    DUMP_HEX64(snap->pd_table_phys);
    pf_walk_emit_text(" 2A=");
    DUMP_HEX64(snap->pde_phys);
    pf_walk_emit_text(" 2E=");
    DUMP_HEX64(snap->pde);
    pf_walk_emit_text(" 1T=");
    DUMP_HEX64(snap->pt_table_phys);
    pf_walk_emit_text(" 1A=");
    DUMP_HEX64(snap->pte_phys);
    pf_walk_emit_text(" 1E=");
    DUMP_HEX64(snap->pte);
    pf_walk_emit_text(" FPA=");
    DUMP_HEX64(snap->final_phys);
    OUTC('\n');
}

static void ISR_SAFE_HELPER pf_emit_walk_level_semantics(char level_tag, uint64_t entry, uint8_t leaf)
{
    OUTC(' ');
    OUTC(level_tag);
    pf_walk_emit_text("P=");
    pf_walk_emit_bool((uint8_t)((entry & AYKEN_PTE_PRESENT) != 0));
    OUTC(' ');
    OUTC(level_tag);
    pf_walk_emit_text("W=");
    pf_walk_emit_bool((uint8_t)((entry & AYKEN_PTE_WRITABLE) != 0));
    OUTC(' ');
    OUTC(level_tag);
    pf_walk_emit_text("U=");
    pf_walk_emit_bool((uint8_t)((entry & AYKEN_PTE_USER) != 0));
    OUTC(' ');
    OUTC(level_tag);
    pf_walk_emit_text("N=");
    pf_walk_emit_bool((uint8_t)((entry & AYKEN_PTE_NO_EXEC) != 0));
    OUTC(' ');
    OUTC(level_tag);
    pf_walk_emit_text("G=");
    pf_walk_emit_bool((uint8_t)((entry & AYKEN_PTE_GLOBAL) != 0));
    OUTC(' ');
    OUTC(level_tag);
    pf_walk_emit_text("H=");
    pf_walk_emit_bool((uint8_t)((entry & (1ULL << 7)) != 0));
    OUTC(' ');
    OUTC(level_tag);
    pf_walk_emit_text("A=");
    pf_walk_emit_bool((uint8_t)((entry & (1ULL << 5)) != 0));
    OUTC(' ');
    OUTC(level_tag);
    pf_walk_emit_text("D=");
    pf_walk_emit_bool((uint8_t)((entry & (1ULL << 6)) != 0));
    OUTC(' ');
    OUTC(level_tag);
    pf_walk_emit_text("R=");
    pf_walk_emit_bool(pf_walk_reserved_suspect(entry));
    OUTC(' ');
    OUTC(level_tag);
    pf_walk_emit_text("X=");
    pf_walk_emit_bool(pf_walk_exec_ok(entry));
    OUTC(' ');
    OUTC(level_tag);
    pf_walk_emit_text("L=");
    pf_walk_emit_bool(leaf);
}

static void ISR_SAFE_HELPER pf_emit_walk_semantics_line(const char *tag, const pf_walk_snapshot_t *snap)
{
    uint8_t leaf_3;
    uint8_t leaf_2;
    uint8_t leaf_1;

    if (!tag || !snap) {
        return;
    }

    leaf_3 = (uint8_t)((snap->pdpte & AYKEN_PTE_PRESENT) && (snap->pdpte & (1ULL << 7)));
    leaf_2 = (uint8_t)((snap->pde & AYKEN_PTE_PRESENT) && (snap->pde & (1ULL << 7)));
    leaf_1 = (uint8_t)((snap->pte & AYKEN_PTE_PRESENT) != 0);

    OUTC('W');
    pf_walk_emit_text(tag);
    pf_walk_emit_text(" OK=");
    pf_walk_emit_bool(snap->valid);
    pf_walk_emit_text(" V=");
    DUMP_HEX64(snap->va);
    pf_emit_walk_level_semantics('4', snap->pml4e, 0);
    pf_emit_walk_level_semantics('3', snap->pdpte, leaf_3);
    pf_emit_walk_level_semantics('2', snap->pde, leaf_2);
    pf_emit_walk_level_semantics('1', snap->pte, leaf_1);
    pf_walk_emit_text(" FPA=");
    DUMP_HEX64(snap->final_phys);
    OUTC('\n');
}

__attribute__((unused))
static void dump_exc_common(uint8_t vec, uint64_t err, const struct interrupt_frame *frame, int has_cr2)
{
    uint64_t cr2 = 0;
    if (has_cr2) {
        __asm__ volatile("mov %%cr2, %0" : "=r"(cr2));
    }
    OUTC('['); OUTC('K'); OUTC(']'); OUTC('['); OUTC('E'); OUTC('X'); OUTC('C'); OUTC(']'); OUTC(' ');
    OUTC('v'); OUTC('e'); OUTC('c'); OUTC('='); OUTC('0'); OUTC('x'); DUMP_HEX16(vec);
    OUTC(' '); OUTC('e'); OUTC('r'); OUTC('r'); OUTC('='); OUTC('0'); OUTC('x'); DUMP_HEX64(err);
    OUTC(' '); OUTC('r'); OUTC('i'); OUTC('p'); OUTC('='); OUTC('0'); OUTC('x'); DUMP_HEX64(frame->rip);
    OUTC(' '); OUTC('r'); OUTC('s'); OUTC('p'); OUTC('='); OUTC('0'); OUTC('x'); DUMP_HEX64(frame->rsp);
    OUTC(' '); OUTC('c'); OUTC('s'); OUTC('='); OUTC('0'); OUTC('x'); DUMP_HEX16((uint16_t)frame->cs);
    OUTC(' '); OUTC('s'); OUTC('s'); OUTC('='); OUTC('0'); OUTC('x'); DUMP_HEX16((uint16_t)frame->ss);
    if (has_cr2) {
        OUTC(' '); OUTC('c'); OUTC('r'); OUTC('2'); OUTC('='); OUTC('0'); OUTC('x'); DUMP_HEX64(cr2);
    }
    OUTC('\n');
}

__attribute__((naked))
static void isr_pf_stub(void)
{
    __asm__ volatile(
        // [[PF_STUB_ENTER]] - EARLIEST possible marker, before ANY stack operations
        "movb $'[', %al\n"
        "outb %al, $0xE9\n"
        "movb $'[', %al\n"
        "outb %al, $0xE9\n"
        "movb $'P', %al\n"
        "outb %al, $0xE9\n"
        "movb $'F', %al\n"
        "outb %al, $0xE9\n"
        "movb $'_', %al\n"
        "outb %al, $0xE9\n"
        "movb $'S', %al\n"
        "outb %al, $0xE9\n"
        "movb $'T', %al\n"
        "outb %al, $0xE9\n"
        "movb $'U', %al\n"
        "outb %al, $0xE9\n"
        "movb $'B', %al\n"
        "outb %al, $0xE9\n"
        "movb $'_', %al\n"
        "outb %al, $0xE9\n"
        "movb $'E', %al\n"
        "outb %al, $0xE9\n"
        "movb $'N', %al\n"
        "outb %al, $0xE9\n"
        "movb $'T', %al\n"
        "outb %al, $0xE9\n"
        "movb $'E', %al\n"
        "outb %al, $0xE9\n"
        "movb $'R', %al\n"
        "outb %al, $0xE9\n"
        "movb $']', %al\n"
        "outb %al, $0xE9\n"
        "movb $']', %al\n"
        "outb %al, $0xE9\n"
        "movb $'\\n', %al\n"
        "outb %al, $0xE9\n"
        // Now witness RSP and TSS.RSP0
        "movb $'[', %al\n"
        "outb %al, $0xE9\n"
        "movb $'[', %al\n"
        "outb %al, $0xE9\n"
        "movb $'R', %al\n"
        "outb %al, $0xE9\n"
        "movb $'S', %al\n"
        "outb %al, $0xE9\n"
        "movb $'P', %al\n"
        "outb %al, $0xE9\n"
        "movb $'0', %al\n"
        "outb %al, $0xE9\n"
        "movb $'_', %al\n"
        "outb %al, $0xE9\n"
        "movb $'E', %al\n"
        "outb %al, $0xE9\n"
        "movb $'N', %al\n"
        "outb %al, $0xE9\n"
        "movb $'T', %al\n"
        "outb %al, $0xE9\n"
        "movb $'R', %al\n"
        "outb %al, $0xE9\n"
        "movb $'Y', %al\n"
        "outb %al, $0xE9\n"
        "movb $']', %al\n"
        "outb %al, $0xE9\n"
        "movb $']', %al\n"
        "outb %al, $0xE9\n"
        "movb $'\\n', %al\n"
        "outb %al, $0xE9\n"
        "cli\n"
        "1: hlt\n"
        "jmp 1b\n"
    );
}

__attribute__((naked))
static void isr_gp_stub(void)
{
    __asm__ volatile(
        // [[GP_STUB_ENTER]] - EARLIEST possible marker, before ANY stack operations
        "movb $'[', %al\n"
        "outb %al, $0xE9\n"
        "movb $'[', %al\n"
        "outb %al, $0xE9\n"
        "movb $'G', %al\n"
        "outb %al, $0xE9\n"
        "movb $'P', %al\n"
        "outb %al, $0xE9\n"
        "movb $'_', %al\n"
        "outb %al, $0xE9\n"
        "movb $'S', %al\n"
        "outb %al, $0xE9\n"
        "movb $'T', %al\n"
        "outb %al, $0xE9\n"
        "movb $'U', %al\n"
        "outb %al, $0xE9\n"
        "movb $'B', %al\n"
        "outb %al, $0xE9\n"
        "movb $'_', %al\n"
        "outb %al, $0xE9\n"
        "movb $'E', %al\n"
        "outb %al, $0xE9\n"
        "movb $'N', %al\n"
        "outb %al, $0xE9\n"
        "movb $'T', %al\n"
        "outb %al, $0xE9\n"
        "movb $'E', %al\n"
        "outb %al, $0xE9\n"
        "movb $'R', %al\n"
        "outb %al, $0xE9\n"
        "movb $']', %al\n"
        "outb %al, $0xE9\n"
        "movb $']', %al\n"
        "outb %al, $0xE9\n"
        "movb $'\\n', %al\n"
        "outb %al, $0xE9\n"
        // Now witness RSP and TSS.RSP0
        "movb $'[', %al\n"
        "outb %al, $0xE9\n"
        "movb $'[', %al\n"
        "outb %al, $0xE9\n"
        "movb $'R', %al\n"
        "outb %al, $0xE9\n"
        "movb $'S', %al\n"
        "outb %al, $0xE9\n"
        "movb $'P', %al\n"
        "outb %al, $0xE9\n"
        "movb $'0', %al\n"
        "outb %al, $0xE9\n"
        "movb $'_', %al\n"
        "outb %al, $0xE9\n"
        "movb $'E', %al\n"
        "outb %al, $0xE9\n"
        "movb $'N', %al\n"
        "outb %al, $0xE9\n"
        "movb $'T', %al\n"
        "outb %al, $0xE9\n"
        "movb $'R', %al\n"
        "outb %al, $0xE9\n"
        "movb $'Y', %al\n"
        "outb %al, $0xE9\n"
        "movb $']', %al\n"
        "outb %al, $0xE9\n"
        "movb $']', %al\n"
        "outb %al, $0xE9\n"
        "movb $'\\n', %al\n"
        "outb %al, $0xE9\n"
        "cli\n"
        "1: hlt\n"
        "jmp 1b\n"
    );
}

__attribute__((naked))
static void isr_df_stub(void)
{
    __asm__ volatile(
        // [[DF_STUB_ENTER]] - EARLIEST possible marker for Double Fault
        "movb $'[', %al\n"
        "outb %al, $0xE9\n"
        "movb $'[', %al\n"
        "outb %al, $0xE9\n"
        "movb $'D', %al\n"
        "outb %al, $0xE9\n"
        "movb $'F', %al\n"
        "outb %al, $0xE9\n"
        "movb $'_', %al\n"
        "outb %al, $0xE9\n"
        "movb $'S', %al\n"
        "outb %al, $0xE9\n"
        "movb $'T', %al\n"
        "outb %al, $0xE9\n"
        "movb $'U', %al\n"
        "outb %al, $0xE9\n"
        "movb $'B', %al\n"
        "outb %al, $0xE9\n"
        "movb $'_', %al\n"
        "outb %al, $0xE9\n"
        "movb $'E', %al\n"
        "outb %al, $0xE9\n"
        "movb $'N', %al\n"
        "outb %al, $0xE9\n"
        "movb $'T', %al\n"
        "outb %al, $0xE9\n"
        "movb $'E', %al\n"
        "outb %al, $0xE9\n"
        "movb $'R', %al\n"
        "outb %al, $0xE9\n"
        "movb $']', %al\n"
        "outb %al, $0xE9\n"
        "movb $']', %al\n"
        "outb %al, $0xE9\n"
        "movb $'!', %al\n"
        "outb %al, $0xE9\n"
        "movb $'\\n', %al\n"
        "outb %al, $0xE9\n"
        "cli\n"
        "1: hlt\n"
        "jmp 1b\n"
    );
}

__attribute__((naked))
static void isr_ts_stub(void)
{
    __asm__ volatile(
        // [[TS_STUB_ENTER]] - Invalid TSS exception
        "movb $'[', %al\n"
        "outb %al, $0xE9\n"
        "movb $'[', %al\n"
        "outb %al, $0xE9\n"
        "movb $'T', %al\n"
        "outb %al, $0xE9\n"
        "movb $'S', %al\n"
        "outb %al, $0xE9\n"
        "movb $'_', %al\n"
        "outb %al, $0xE9\n"
        "movb $'S', %al\n"
        "outb %al, $0xE9\n"
        "movb $'T', %al\n"
        "outb %al, $0xE9\n"
        "movb $'U', %al\n"
        "outb %al, $0xE9\n"
        "movb $'B', %al\n"
        "outb %al, $0xE9\n"
        "movb $']', %al\n"
        "outb %al, $0xE9\n"
        "movb $']', %al\n"
        "outb %al, $0xE9\n"
        "movb $'\\n', %al\n"
        "outb %al, $0xE9\n"
        "cli\n"
        "1: hlt\n"
        "jmp 1b\n"
    );
}

__attribute__((naked))
static void isr_np_stub(void)
{
    __asm__ volatile(
        // [[NP_STUB_ENTER]] - Segment Not Present exception
        "movb $'[', %al\n"
        "outb %al, $0xE9\n"
        "movb $'[', %al\n"
        "outb %al, $0xE9\n"
        "movb $'N', %al\n"
        "outb %al, $0xE9\n"
        "movb $'P', %al\n"
        "outb %al, $0xE9\n"
        "movb $'_', %al\n"
        "outb %al, $0xE9\n"
        "movb $'S', %al\n"
        "outb %al, $0xE9\n"
        "movb $'T', %al\n"
        "outb %al, $0xE9\n"
        "movb $'U', %al\n"
        "outb %al, $0xE9\n"
        "movb $'B', %al\n"
        "outb %al, $0xE9\n"
        "movb $']', %al\n"
        "outb %al, $0xE9\n"
        "movb $']', %al\n"
        "outb %al, $0xE9\n"
        "movb $'\\n', %al\n"
        "outb %al, $0xE9\n"
        "cli\n"
        "1: hlt\n"
        "jmp 1b\n"
    );
}

__attribute__((naked))
static void isr_ss_stub(void)
{
    __asm__ volatile(
        // [[SS_STUB_ENTER]] - Stack Segment Fault exception
        "movb $'[', %al\n"
        "outb %al, $0xE9\n"
        "movb $'[', %al\n"
        "outb %al, $0xE9\n"
        "movb $'S', %al\n"
        "outb %al, $0xE9\n"
        "movb $'S', %al\n"
        "outb %al, $0xE9\n"
        "movb $'_', %al\n"
        "outb %al, $0xE9\n"
        "movb $'S', %al\n"
        "outb %al, $0xE9\n"
        "movb $'T', %al\n"
        "outb %al, $0xE9\n"
        "movb $'U', %al\n"
        "outb %al, $0xE9\n"
        "movb $'B', %al\n"
        "outb %al, $0xE9\n"
        "movb $']', %al\n"
        "outb %al, $0xE9\n"
        "movb $']', %al\n"
        "outb %al, $0xE9\n"
        "movb $'\\n', %al\n"
        "outb %al, $0xE9\n"
        "cli\n"
        "1: hlt\n"
        "jmp 1b\n"
    );
}

__attribute__((interrupt))
static void isr_bp(struct interrupt_frame *frame)
{
    const uint16_t cs = (uint16_t)frame->cs;
    const uint16_t ss = (uint16_t)frame->ss;
    const uint64_t rip = frame->rip;
    const uint64_t upper = rip >> 48;
    const uint64_t sign = (rip >> 47) & 1ULL;
    const int rip_canonical = sign ? (upper == 0xFFFFULL) : (upper == 0x0000ULL);
    const int user_cpl = ((cs & 0x3u) == 0x3u);
    const int user_rip = (rip >= USER_TEXT_BASE) && (rip < USER_STACK_TOP);
    const int is_ring3_bp =
        user_cpl &&
        user_rip &&
        rip_canonical;

    if (is_ring3_bp) {
        phase10_ring3_user_code_seen = 1u;
        // Source anchor token for runtime-marker-contract: P10_RING3_USER_CODE
        // ISR-safe marker emission: no helper calls in interrupt context.
        OUTC('P'); OUTC('1'); OUTC('0'); OUTC('_');
        OUTC('R'); OUTC('I'); OUTC('N'); OUTC('G'); OUTC('3'); OUTC('_');
        OUTC('U'); OUTC('S'); OUTC('E'); OUTC('R'); OUTC('_');
        OUTC('C'); OUTC('O'); OUTC('D'); OUTC('E'); OUTC('\n');
        HALT_FOREVER();
    }

    // Ring0 breakpoint: keep debug behavior and return.
    OUTC('B'); OUTC('P'); OUTC('!'); OUTC('\n');
}

__attribute__((interrupt))
static void isr_ud(struct interrupt_frame *frame)
{
    OUTC('['); OUTC('E'); OUTC('X'); OUTC(']'); OUTC('['); OUTC('#'); OUTC('U'); OUTC('D'); OUTC(']'); OUTC(' ');
    OUTC('r'); OUTC('i'); OUTC('p'); OUTC('='); OUTC('0'); OUTC('x');
    DUMP_HEX64(frame->rip);
    OUTC(' '); OUTC('c'); OUTC('s'); OUTC('='); OUTC('0'); OUTC('x');
    DUMP_HEX16((uint16_t)frame->cs);
    OUTC(' '); OUTC('r'); OUTC('s'); OUTC('p'); OUTC('='); OUTC('0'); OUTC('x');
    DUMP_HEX64(frame->rsp);
    OUTC(' '); OUTC('r'); OUTC('f'); OUTC('l'); OUTC('a'); OUTC('g'); OUTC('s'); OUTC('='); OUTC('0'); OUTC('x');
    DUMP_HEX64(frame->rflags);
    OUTC('\n');
    HALT_FOREVER();
}

__attribute__((interrupt))
static void isr_gp(struct interrupt_frame *frame, uint64_t error_code)
{
    (void)error_code;
    // CRITICAL: GP fault marker - ASM safe, no C calls
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'G'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'P'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'!'), "Nd"(0xE9));
    
    // Show RIP where GP occurred (simple hex dump)
    uint64_t rip = frame->rip;
    for (int i = 60; i >= 0; i -= 4) {
        uint8_t nibble = (rip >> i) & 0xF;
        uint8_t ch = (nibble < 10) ? ('0' + nibble) : ('A' + nibble - 10);
        __asm__ volatile("outb %0, %1" : : "a"(ch), "Nd"(0xE9));
    }
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'\n'), "Nd"(0xE9));
    
    // Halt forever - no C calls
    __asm__ volatile("cli; 1: hlt; jmp 1b");
}

__attribute__((interrupt))
static void isr_pf(struct interrupt_frame *frame, uint64_t error_code)
{
    uint64_t cr2 = 0;
    uint64_t cr3 = 0;
    pf_walk_snapshot_t rip_walk = (pf_walk_snapshot_t){0};

    __asm__ volatile("mov %%cr2, %0" : "=r"(cr2));
    __asm__ volatile("mov %%cr3, %0" : "=r"(cr3));

    // CRITICAL: Page fault marker - keep ASM-safe emission.
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'P'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'F'), "Nd"(0xE9));
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'!'), "Nd"(0xE9));

    // Fault RIP (kept first for compatibility with existing logs/tools).
    uint64_t rip = frame->rip;
    for (int i = 60; i >= 0; i -= 4) {
        uint8_t nibble = (rip >> i) & 0xF;
        uint8_t ch = (nibble < 10) ? ('0' + nibble) : ('A' + nibble - 10);
        __asm__ volatile("outb %0, %1" : : "a"(ch), "Nd"(0xE9));
    }
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)' '), "Nd"(0xE9));
    OUTC('C'); OUTC('R'); OUTC('2'); OUTC('='); DUMP_HEX64(cr2);
    OUTC(' ');
    OUTC('C'); OUTC('R'); OUTC('3'); OUTC('='); DUMP_HEX64(cr3);
    OUTC(' ');
    OUTC('E'); OUTC('R'); OUTC('R'); OUTC('='); DUMP_HEX64(error_code);
    OUTC(' ');
    OUTC('P'); OUTC('=');
    OUTC((error_code & (1ULL << 0)) ? '1' : '0');
    OUTC(' ');
    OUTC('W'); OUTC('=');
    OUTC((error_code & (1ULL << 1)) ? '1' : '0');
    OUTC(' ');
    OUTC('U'); OUTC('=');
    OUTC((error_code & (1ULL << 2)) ? '1' : '0');
    OUTC(' ');
    OUTC('R'); OUTC('=');
    OUTC((error_code & (1ULL << 3)) ? '1' : '0');
    OUTC(' ');
    OUTC('I'); OUTC('=');
    OUTC((error_code & (1ULL << 4)) ? '1' : '0');
    OUTC(' ');
    OUTC('C'); OUTC('P'); OUTC('L'); OUTC('=');
    OUTC((uint8_t)('0' + (((uint16_t)frame->cs) & 0x3)));
    OUTC(' ');
    OUTC('C'); OUTC('S'); OUTC('='); DUMP_HEX16((uint16_t)frame->cs);
    OUTC(' ');
    OUTC('S'); OUTC('S'); OUTC('='); DUMP_HEX16((uint16_t)frame->ss);
    OUTC(' ');
    OUTC('R'); OUTC('S'); OUTC('P'); OUTC('='); DUMP_HEX64(frame->rsp);
    OUTC(' ');
    OUTC('P'); OUTC('I'); OUTC('D'); OUTC('=');
    if (current_proc) {
        DUMP_HEX64((uint64_t)(uint32_t)current_proc->pid);
        OUTC(' ');
        OUTC('P'); OUTC('C'); OUTC('S'); OUTC('='); DUMP_HEX16(current_proc->context.cs);
        OUTC(' ');
        OUTC('P'); OUTC('R'); OUTC('I'); OUTC('P'); OUTC('='); DUMP_HEX64(current_proc->context.rip);
        OUTC(' ');
        OUTC('P'); OUTC('R'); OUTC('S'); OUTC('P'); OUTC('='); DUMP_HEX64(current_proc->context.rsp);
        OUTC(' ');
        OUTC('P'); OUTC('C'); OUTC('R'); OUTC('3'); OUTC('='); DUMP_HEX64(current_proc->context.cr3);
    } else {
        OUTC('N'); OUTC('U'); OUTC('L'); OUTC('L');
    }
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)'\n'), "Nd"(0xE9));

    pf_capture_walk_snapshot(cr3, frame->rip, &rip_walk);
    pf_emit_walk_snapshot_line("PFH", &rip_walk);
    pf_emit_walk_semantics_line("PFS", &rip_walk);

    // Halt forever - no recovery from early PF in validation path.
    __asm__ volatile("cli; 1: hlt; jmp 1b");
}

void idt_set_gate(uint8_t num, interrupt_handler_t handler, uint8_t flags)
{
    uint64_t addr = (uint64_t)handler;
    idt_table[num].offset_low = addr & 0xFFFF;
    idt_table[num].selector = GDT_KERNEL_CODE; // kernel code segment
    idt_table[num].ist = 0;
    idt_table[num].type_attr = flags;
    idt_table[num].offset_mid = (addr >> 16) & 0xFFFF;
    idt_table[num].offset_high = (addr >> 32) & 0xFFFFFFFF;
    idt_table[num].zero = 0;
}

static void idt_set_gate_selector(uint8_t num, interrupt_handler_t handler, uint8_t flags, uint16_t selector)
{
    uint64_t addr = (uint64_t)handler;
    idt_table[num].offset_low = addr & 0xFFFF;
    idt_table[num].selector = selector;
    idt_table[num].ist = 0;
    idt_table[num].type_attr = flags;
    idt_table[num].offset_mid = (addr >> 16) & 0xFFFF;
    idt_table[num].offset_high = (addr >> 32) & 0xFFFFFFFF;
    idt_table[num].zero = 0;
}

void interrupts_install(void)
{
    // zero-out IDT
    for (int i = 0; i < 256; ++i) {
        idt_table[i].offset_low = 0;
        idt_table[i].selector = 0;
        idt_table[i].ist = 0;
        idt_table[i].type_attr = 0;
        idt_table[i].offset_mid = 0;
        idt_table[i].offset_high = 0;
        idt_table[i].zero = 0;
    }

    // Install core exception handlers for the late IDT (early IDT gets wiped above).
    // INT3 uses interrupt gate (DPL=3) to keep marker emission deterministic.
    idt_set_gate(3,  (interrupt_handler_t)isr_bp, 0xEE);
    idt_set_gate(6,  isr_ud, 0x8F);
    idt_set_gate(8,  (interrupt_handler_t)isr_df_stub, 0x8F);
    idt_set_gate(10, (interrupt_handler_t)isr_ts_stub, 0x8F);
    idt_set_gate(11, (interrupt_handler_t)isr_np_stub, 0x8F);
    idt_set_gate(12, (interrupt_handler_t)isr_ss_stub, 0x8F);
/* Validation and explicit IRQ-debug builds use verbose #GP/#PF handlers. */
#if (defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)) || \
    (defined(AYKEN_DEBUG_IRQ) && (AYKEN_DEBUG_IRQ == 1))
    idt_set_gate(13, (interrupt_handler_t)isr_gp, 0x8F);
    idt_set_gate(14, (interrupt_handler_t)isr_pf, 0x8F);
#else
    idt_set_gate(13, (interrupt_handler_t)isr_gp_stub, 0x8F);
    idt_set_gate(14, (interrupt_handler_t)isr_pf_stub, 0x8F);
#endif

    // Keep current-stack delivery for diagnostic consistency during bring-up.
    idt_table[3].ist  = 0;
    idt_table[8].ist  = 0;
    idt_table[10].ist = 0;
    idt_table[11].ist = 0;
    idt_table[12].ist = 0;
    idt_table[13].ist = 0;
    idt_table[14].ist = 0;

    idt_descriptor.limit = sizeof(idt_table) - 1;
    idt_descriptor.base = (uint64_t)&idt_table[0];

    idt_init();
}

void interrupts_install_early(void)
{
    uint16_t cs = 0;
    __asm__ volatile("mov %%cs, %0" : "=r"(cs));

    // zero-out IDT
    for (int i = 0; i < 256; ++i) {
        idt_table[i].offset_low = 0;
        idt_table[i].selector = 0;
        idt_table[i].ist = 0;
        idt_table[i].type_attr = 0;
        idt_table[i].offset_mid = 0;
        idt_table[i].offset_high = 0;
        idt_table[i].zero = 0;
    }

    // Exceptions we care about early.
    // CRITICAL: INT3 (#BP) is DPL=3 interrupt gate for deterministic marker path.
    idt_set_gate_selector(3,  (interrupt_handler_t)isr_bp, 0xEE, cs);
    idt_set_gate_selector(6,  isr_ud, 0x8F, cs);
    idt_set_gate_selector(8,  (interrupt_handler_t)isr_df_stub, 0x8F, cs);
    idt_set_gate_selector(10, (interrupt_handler_t)isr_ts_stub, 0x8F, cs);
    idt_set_gate_selector(11, (interrupt_handler_t)isr_np_stub, 0x8F, cs);
    idt_set_gate_selector(12, (interrupt_handler_t)isr_ss_stub, 0x8F, cs);
/* Validation and explicit IRQ-debug builds use verbose #GP/#PF handlers. */
#if (defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)) || \
    (defined(AYKEN_DEBUG_IRQ) && (AYKEN_DEBUG_IRQ == 1))
    idt_set_gate_selector(13, (interrupt_handler_t)isr_gp, 0x8F, cs);
    idt_set_gate_selector(14, (interrupt_handler_t)isr_pf, 0x8F, cs);
#else
    idt_set_gate_selector(13, (interrupt_handler_t)isr_gp_stub, 0x8F, cs);
    idt_set_gate_selector(14, (interrupt_handler_t)isr_pf_stub, 0x8F, cs);
#endif
    // Keep current-stack delivery for diagnostic consistency during bring-up.
    idt_table[3].ist  = 0;
    idt_table[8].ist  = 0;
    idt_table[10].ist = 0;
    idt_table[11].ist = 0;
    idt_table[12].ist = 0;
    idt_table[13].ist = 0;
    idt_table[14].ist = 0;

    idt_descriptor.limit = sizeof(idt_table) - 1;
    idt_descriptor.base = (uint64_t)&idt_table[0];

    idt_init();

}
