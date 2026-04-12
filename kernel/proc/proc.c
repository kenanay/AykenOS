// kernel/proc/proc.c
#include <stddef.h>
#include "../include/proc.h"
#include "../sched/sched.h"
#include "../include/mm.h"
#include "../include/mm/user_as.h"
#include "../include/kheap.h"
#include "../include/ayken.h"
#include "../include/gdt_idt.h"
#include "../include/capability.h"
#include "../drivers/console/fb_console.h"
#include "../arch/x86_64/port_io.h"
#include "../sched/sched_mailbox.h"
#include "../include/alias_registry.h"

#ifndef AYKEN_GATE45_PROOF
#define AYKEN_GATE45_PROOF 0
#endif

#ifndef AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST
#define AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST 0
#endif

#ifndef AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_SELFTEST
#define AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_SELFTEST 0
#endif

#ifndef AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT
#define AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT 2
#endif

#ifndef AYKEN_LOW_HALF_KHEAP_INTERLEAVING_PROOF_SELFTEST
#define AYKEN_LOW_HALF_KHEAP_INTERLEAVING_PROOF_SELFTEST 0
#endif

#ifndef AYKEN_LOW_HALF_KHEAP_INTERLEAVING_PROOF_COUNT
#define AYKEN_LOW_HALF_KHEAP_INTERLEAVING_PROOF_COUNT 2
#endif

#ifndef AYKEN_ALIAS_PROOF_SELFTEST
#define AYKEN_ALIAS_PROOF_SELFTEST 0
#endif

_Static_assert(offsetof(cpu_context_t, rip) == 48, "ctx.rip offset");
_Static_assert(offsetof(cpu_context_t, rsp) == 56, "ctx.rsp offset");
_Static_assert(offsetof(cpu_context_t, rflags) == 64, "ctx.rflags offset");
_Static_assert(offsetof(cpu_context_t, cr3) == 72, "ctx.cr3 offset");
_Static_assert(offsetof(cpu_context_t, cs) == 80, "ctx.cs offset");
_Static_assert(offsetof(cpu_context_t, ss) == 82, "ctx.ss offset");
_Static_assert(sizeof(cpu_context_t) == 96, "ctx size");

// Use compiler builtin functions for memory operations
#define memset __builtin_memset
#define memcpy __builtin_memcpy

// Ring3 INT80 return-path canary
#define RING3_CANARY_ADDR 0x0000000000405000ULL
#define RING3_CANARY_PRE  0x1111111122222222ULL
#define RING3_CANARY_POST 0x3333333344444444ULL

static proc_t* proc_table[MAX_PROCS];
static int next_pid = 1;
static proc_t* g_deferred_reap_queue[MAX_PROCS];
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
static uint32_t g_low_half_kheap_runtime_seq = 0;
#endif
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    (AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST == 1)
static uint8_t g_low_half_kheap_exit_selftest_armed = 0;
static uint8_t g_low_half_kheap_exit_selftest_completed = 0;
static uint32_t g_low_half_kheap_exit_selftest_exit_pid = 0;
static uint32_t g_low_half_kheap_exit_selftest_return_pid = 0;
#endif
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    (AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_SELFTEST == 1)
#define LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT \
    ((uint32_t)AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT)
_Static_assert(AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT > 0,
               "multi-exit proof count must be positive");
_Static_assert(AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT < MAX_PROCS,
               "multi-exit proof count must remain below MAX_PROCS");
static uint8_t g_low_half_kheap_multi_exit_selftest_armed = 0;
static uint8_t g_low_half_kheap_multi_exit_selftest_completed = 0;
static uint32_t g_low_half_kheap_multi_exit_current_slot = 0;
static uint32_t g_low_half_kheap_multi_exit_owner_pid = 0;
static uint32_t g_low_half_kheap_multi_exit_return_pid = 0;
static uint32_t g_low_half_kheap_multi_exit_exit_pids[AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT] = {0};
#endif
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    (AYKEN_LOW_HALF_KHEAP_INTERLEAVING_PROOF_SELFTEST == 1)
#define LOW_HALF_KHEAP_INTERLEAVING_PROOF_COUNT \
    ((uint32_t)AYKEN_LOW_HALF_KHEAP_INTERLEAVING_PROOF_COUNT)
_Static_assert(AYKEN_LOW_HALF_KHEAP_INTERLEAVING_PROOF_COUNT > 1,
               "interleaving proof count must be at least 2");
_Static_assert(AYKEN_LOW_HALF_KHEAP_INTERLEAVING_PROOF_COUNT < MAX_PROCS,
               "interleaving proof count must remain below MAX_PROCS");
static uint8_t g_low_half_kheap_interleaving_selftest_prepared = 0;
static uint8_t g_low_half_kheap_interleaving_selftest_armed = 0;
static uint8_t g_low_half_kheap_interleaving_selftest_completed = 0;
static uint32_t g_low_half_kheap_interleaving_current_slot = 0;
static uint32_t g_low_half_kheap_interleaving_owner_pid = 0;
static uint32_t g_low_half_kheap_interleaving_return_pid = 0;
static uint32_t g_low_half_kheap_interleaving_exit_pids[AYKEN_LOW_HALF_KHEAP_INTERLEAVING_PROOF_COUNT] = {0};
#endif

void init_process_main(void);
void kernel_first_entry(void);
void kernel_iret_entry(void);  // IRET-safe kernel entry
extern char ring3_enter_post_cr3[];

#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    ((AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST == 1) || \
     (AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_SELFTEST == 1) || \
     (AYKEN_LOW_HALF_KHEAP_INTERLEAVING_PROOF_SELFTEST == 1))
static const uint8_t low_half_kheap_exit_proof_code[] = {
    0xB9, 0x00, 0x00, 0x20, 0x00, /* mov ecx, 0x00200000 */
    0xF3, 0x90,                   /* pause */
    0xE2, 0xFC,                   /* loop .-4 */
    0xB8, 0xF0, 0x03, 0x00, 0x00, /* mov eax, 1008 */
    0xBF, 0x01, 0x00, 0x00, 0x00, /* mov edi, 1 */
    0x31, 0xF6,                   /* xor esi, esi */
    0x31, 0xD2,                   /* xor edx, edx */
    0x45, 0x31, 0xD2,             /* xor r10d, r10d */
    0xCD, 0x80,                   /* int 0x80 */
    0xB8, 0xF1, 0x03, 0x00, 0x00, /* mov eax, 1009 */
    0xBF, 0x17, 0x0E, 0x00, 0x00, /* mov edi, 0xE17 */
    0x31, 0xF6,                   /* xor esi, esi */
    0x31, 0xD2,                   /* xor edx, edx */
    0x45, 0x31, 0xD2,             /* xor r10d, r10d */
    0xCD, 0x80,                   /* int 0x80 */
    0xEB, 0xFE,                   /* jmp $ */
};
#endif

static void debugcon_write(const char *s)
{
    for (; *s; ++s) {
        outb(0xE9, (uint8_t)*s);
    }
}

static void debugcon_write_char(char c)
{
    outb(0xE9, (uint8_t)c);
}

static void debugcon_hex8(uint8_t v)
{
    static const char hex[] = "0123456789ABCDEF";
    outb(0xE9, (uint8_t)hex[(v >> 4) & 0xF]);
    outb(0xE9, (uint8_t)hex[v & 0xF]);
}

static void debugcon_hex64(uint64_t v)
{
    static const char hex[] = "0123456789ABCDEF";
    for (int i = 15; i >= 0; --i) {
        outb(0xE9, (uint8_t)hex[(v >> (i * 4)) & 0xF]);
    }
}

static void __attribute__((unused)) debugcon_u32(uint32_t v)
{
    char buf[10];
    int i = 0;
    if (v == 0) {
        outb(0xE9, (uint8_t)'0');
        return;
    }
    while (v > 0 && i < (int)sizeof(buf)) {
        buf[i++] = (char)('0' + (v % 10));
        v /= 10;
    }
    while (i > 0) {
        outb(0xE9, (uint8_t)buf[--i]);
    }
}

static uint64_t debugcon_read_u64_le(const uint8_t *ptr)
{
    uint64_t value = 0;

    if (!ptr) {
        return 0;
    }

    for (uint32_t i = 0; i < 8; ++i) {
        value |= ((uint64_t)ptr[i]) << (i * 8);
    }

    return value;
}

static uint64_t debugcon_hash_page_bytes(const uint8_t *page)
{
    uint64_t hash = 1469598103934665603ULL;

    if (!page) {
        return 0;
    }

    for (uint64_t i = 0; i < AYKEN_FRAME_SIZE; ++i) {
        hash ^= page[i];
        hash *= 1099511628211ULL;
    }

    return hash;
}

static void proc_emit_phys_frame_witness(const char *tag,
                                         const char *phase,
                                         uint64_t root_phys,
                                         uint64_t pte,
                                         uint64_t phys_page)
{
    const uint8_t *page;
    int used;

    if (!tag || !phase) {
        return;
    }

    phys_page &= AYKEN_PTE_ADDR_MASK;
    page = phys_page ? (const uint8_t *)paging_phys_to_virt(phys_page) : NULL;
    used = phys_page ? phys_frame_is_used(phys_page) : 0;

    debugcon_write(tag);
    debugcon_write(" phase=");
    debugcon_write(phase);
    debugcon_write(" root=");
    debugcon_hex64(root_phys & AYKEN_PTE_ADDR_MASK);
    debugcon_write(" pte=");
    debugcon_hex64(pte);
    debugcon_write(" phys=");
    debugcon_hex64(phys_page);
    debugcon_write(" used=");
    debugcon_write_char((phys_page != 0 && used == 1) ? '1' : '0');
    debugcon_write(" lo=");
    debugcon_hex64(debugcon_read_u64_le(page));
    debugcon_write(" hi=");
    debugcon_hex64(debugcon_read_u64_le(page ? page + 8 : NULL));
    debugcon_write(" hash=");
    debugcon_hex64(debugcon_hash_page_bytes(page));
    debugcon_write("\n");
}

static void proc_emit_user_text_root_witness(uint64_t root_phys, const char *phase)
{
    uint64_t text_pte;
    uint64_t text_phys;

    if (!root_phys || !phase) {
        return;
    }

    text_pte = paging_get_pte_in_pml4(root_phys, USER_TEXT_BASE);
    text_phys = text_pte & AYKEN_PTE_ADDR_MASK;

    proc_emit_phys_frame_witness(
        "P10_ROOT_FRAME_WITNESS",
        phase,
        root_phys,
        0,
        root_phys);
    proc_emit_phys_frame_witness(
        "P10_TEXT_FRAME_WITNESS",
        phase,
        root_phys,
        text_pte,
        text_phys);
}

static uint64_t proc_alloc_user_image_frame(void)
{
    /*
     * Keep authored user image leaves out of the low-phys frame class. The
     * page-table child-table fix already proved MMU-visible mismatch there.
     */
    return phys_alloc_frame_high();
}

#if defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)
static void gate4_emit_pid_marker(uint32_t pid)
{
    debugcon_write("[[AYKEN_GATE4_PID]] pid=");
    debugcon_u32(pid);
    debugcon_write("\n");
}
#endif

static void debug_dump_pte(uint64_t pml4_phys, uint64_t va, const char *tag)
{
    const uint64_t NX_MASK = (1ULL << 63);
    if (!pml4_phys) {
        debugcon_write("[PTE] ");
        debugcon_write(tag);
        debugcon_write(" PML4=0\n");
        return;
    }

    uint64_t *pml4 = (uint64_t *)paging_phys_to_virt(pml4_phys);
    uint16_t pml4_i = (va >> 39) & 0x1FF;
    uint16_t pdpt_i = (va >> 30) & 0x1FF;
    uint16_t pd_i   = (va >> 21) & 0x1FF;
    uint16_t pt_i   = (va >> 12) & 0x1FF;

    uint64_t pml4e = pml4[pml4_i];
    if (!(pml4e & AYKEN_PTE_PRESENT)) {
        debugcon_write("[PTE] ");
        debugcon_write(tag);
        debugcon_write(" VA=");
        debugcon_hex64(va);
        debugcon_write(" PML4E !P\n");
        return;
    }
    uint64_t *pdpt = (uint64_t *)paging_phys_to_virt(pml4e & AYKEN_PTE_ADDR_MASK);

    uint64_t pdpte = pdpt[pdpt_i];
    if (!(pdpte & AYKEN_PTE_PRESENT)) {
        debugcon_write("[PTE] ");
        debugcon_write(tag);
        debugcon_write(" VA=");
        debugcon_hex64(va);
        debugcon_write(" PDPTE !P\n");
        return;
    }
    uint64_t *pd = (uint64_t *)paging_phys_to_virt(pdpte & AYKEN_PTE_ADDR_MASK);

    uint64_t pde = pd[pd_i];
    if (!(pde & AYKEN_PTE_PRESENT)) {
        debugcon_write("[PTE] ");
        debugcon_write(tag);
        debugcon_write(" VA=");
        debugcon_hex64(va);
        debugcon_write(" PDE !P\n");
        return;
    }
    uint64_t *pt = (uint64_t *)paging_phys_to_virt(pde & AYKEN_PTE_ADDR_MASK);

    uint64_t pte = pt[pt_i];
    debugcon_write("[PTE] ");
    debugcon_write(tag);
    debugcon_write(" VA=");
    debugcon_hex64(va);
    debugcon_write(" PTE=");
    debugcon_hex64(pte);
    debugcon_write(" P=");
    debugcon_write_char((pte & AYKEN_PTE_PRESENT) ? '1' : '0');
    debugcon_write(" U=");
    debugcon_write_char((pte & AYKEN_PTE_USER) ? '1' : '0');
    debugcon_write(" W=");
    debugcon_write_char((pte & AYKEN_PTE_WRITABLE) ? '1' : '0');
    debugcon_write(" NX=");
    debugcon_write_char((pte & NX_MASK) ? '1' : '0');
    debugcon_write(" PA=");
    debugcon_hex64(pte & AYKEN_PTE_ADDR_MASK);
    debugcon_write("\n");
}

typedef struct proc_walk_snapshot {
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
} proc_walk_snapshot_t;

static uint8_t proc_walk_reserved_suspect(uint64_t entry)
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

static uint8_t proc_walk_exec_ok(uint64_t entry)
{
    if ((entry & AYKEN_PTE_PRESENT) == 0) {
        return 0;
    }
    if ((entry & AYKEN_PTE_NO_EXEC) != 0) {
        return 0;
    }
    return (uint8_t)(proc_walk_reserved_suspect(entry) == 0);
}

static int proc_capture_walk_snapshot(uint64_t root_phys,
                                      uint64_t va,
                                      proc_walk_snapshot_t *out)
{
    uint64_t *pml4;
    uint16_t pml4_i;
    uint16_t pdpt_i;
    uint16_t pd_i;
    uint16_t pt_i;

    if (!out || !root_phys) {
        return 0;
    }

    memset(out, 0, sizeof(*out));
    out->root_phys = root_phys & AYKEN_PTE_ADDR_MASK;
    out->va = va;
    out->pml4_table_phys = out->root_phys;

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

static void proc_emit_walk_snapshot_hex_line(const char *tag, const proc_walk_snapshot_t *snap)
{
    if (!tag || !snap) {
        return;
    }

    debugcon_write(tag);
    debugcon_write(" OK=");
    debugcon_write_char(snap->valid ? '1' : '0');
    debugcon_write(" R=");
    debugcon_hex64(snap->root_phys);
    debugcon_write(" V=");
    debugcon_hex64(snap->va);
    debugcon_write(" 4T=");
    debugcon_hex64(snap->pml4_table_phys);
    debugcon_write(" 4A=");
    debugcon_hex64(snap->pml4e_phys);
    debugcon_write(" 4E=");
    debugcon_hex64(snap->pml4e);
    debugcon_write(" 3T=");
    debugcon_hex64(snap->pdpt_table_phys);
    debugcon_write(" 3A=");
    debugcon_hex64(snap->pdpte_phys);
    debugcon_write(" 3E=");
    debugcon_hex64(snap->pdpte);
    debugcon_write(" 2T=");
    debugcon_hex64(snap->pd_table_phys);
    debugcon_write(" 2A=");
    debugcon_hex64(snap->pde_phys);
    debugcon_write(" 2E=");
    debugcon_hex64(snap->pde);
    debugcon_write(" 1T=");
    debugcon_hex64(snap->pt_table_phys);
    debugcon_write(" 1A=");
    debugcon_hex64(snap->pte_phys);
    debugcon_write(" 1E=");
    debugcon_hex64(snap->pte);
    debugcon_write(" FPA=");
    debugcon_hex64(snap->final_phys);
    debugcon_write("\n");
}

static void proc_emit_walk_level_semantics(char level_tag, uint64_t entry, uint8_t leaf)
{
    debugcon_write_char(' ');
    debugcon_write_char(level_tag);
    debugcon_write("P=");
    debugcon_write_char((entry & AYKEN_PTE_PRESENT) ? '1' : '0');
    debugcon_write_char(' ');
    debugcon_write_char(level_tag);
    debugcon_write("W=");
    debugcon_write_char((entry & AYKEN_PTE_WRITABLE) ? '1' : '0');
    debugcon_write_char(' ');
    debugcon_write_char(level_tag);
    debugcon_write("U=");
    debugcon_write_char((entry & AYKEN_PTE_USER) ? '1' : '0');
    debugcon_write_char(' ');
    debugcon_write_char(level_tag);
    debugcon_write("N=");
    debugcon_write_char((entry & AYKEN_PTE_NO_EXEC) ? '1' : '0');
    debugcon_write_char(' ');
    debugcon_write_char(level_tag);
    debugcon_write("G=");
    debugcon_write_char((entry & AYKEN_PTE_GLOBAL) ? '1' : '0');
    debugcon_write_char(' ');
    debugcon_write_char(level_tag);
    debugcon_write("H=");
    debugcon_write_char((entry & (1ULL << 7)) ? '1' : '0');
    debugcon_write_char(' ');
    debugcon_write_char(level_tag);
    debugcon_write("A=");
    debugcon_write_char((entry & (1ULL << 5)) ? '1' : '0');
    debugcon_write_char(' ');
    debugcon_write_char(level_tag);
    debugcon_write("D=");
    debugcon_write_char((entry & (1ULL << 6)) ? '1' : '0');
    debugcon_write_char(' ');
    debugcon_write_char(level_tag);
    debugcon_write("R=");
    debugcon_write_char(proc_walk_reserved_suspect(entry) ? '1' : '0');
    debugcon_write_char(' ');
    debugcon_write_char(level_tag);
    debugcon_write("X=");
    debugcon_write_char(proc_walk_exec_ok(entry) ? '1' : '0');
    debugcon_write_char(' ');
    debugcon_write_char(level_tag);
    debugcon_write("L=");
    debugcon_write_char(leaf ? '1' : '0');
}

static void proc_emit_walk_snapshot_semantics_line(const char *tag, const proc_walk_snapshot_t *snap)
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

    debugcon_write(tag);
    debugcon_write(" OK=");
    debugcon_write_char(snap->valid ? '1' : '0');
    debugcon_write(" V=");
    debugcon_hex64(snap->va);
    proc_emit_walk_level_semantics('4', snap->pml4e, 0);
    proc_emit_walk_level_semantics('3', snap->pdpte, leaf_3);
    proc_emit_walk_level_semantics('2', snap->pde, leaf_2);
    proc_emit_walk_level_semantics('1', snap->pte, leaf_1);
    debugcon_write(" FPA=");
    debugcon_hex64(snap->final_phys);
    debugcon_write("\n");
}

static void proc_debug_emit_ring3_creation_snapshot(uint64_t root_phys)
{
    proc_walk_snapshot_t snap;
    uint64_t canonical_fetch_va = ((uint64_t)(uintptr_t)ring3_enter_post_cr3) + 3;

    if (!proc_capture_walk_snapshot(root_phys, canonical_fetch_va, &snap)) {
        memset(&snap, 0, sizeof(snap));
        snap.root_phys = root_phys & AYKEN_PTE_ADDR_MASK;
        snap.va = canonical_fetch_va;
    }

    proc_emit_walk_snapshot_hex_line("CCH", &snap);
    proc_emit_walk_snapshot_semantics_line("CCS", &snap);
}

#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
typedef struct low_half_mapping_stats {
    uint32_t root_entries_present;
    uint32_t leaf_entries_present;
    uint32_t user_leaf_entries_present;
} low_half_mapping_stats_t;

static void collect_low_half_mapping_stats_recursive(uint64_t table_phys,
                                                     uint32_t level,
                                                     low_half_mapping_stats_t *stats)
{
    const uint64_t HUGE_MASK = (1ULL << 7);
    uint64_t *table;
    uint32_t i;

    if (table_phys == 0 || level == 0 || !stats) {
        return;
    }

    table = (uint64_t *)paging_phys_to_virt(table_phys);
    if (!table) {
        return;
    }

    for (i = 0; i < 512; ++i) {
        uint64_t entry = table[i];
        uint64_t child_phys;

        if ((entry & AYKEN_PTE_PRESENT) == 0) {
            continue;
        }

        if ((entry & HUGE_MASK) != 0 || level == 1) {
            stats->leaf_entries_present += 1;
            if (entry & AYKEN_PTE_USER) {
                stats->user_leaf_entries_present += 1;
            }
            continue;
        }

        child_phys = entry & AYKEN_PTE_ADDR_MASK;
        if (child_phys == 0) {
            continue;
        }

        collect_low_half_mapping_stats_recursive(child_phys, level - 1, stats);
    }
}

static void collect_low_half_mapping_stats(uint64_t pml4_phys, low_half_mapping_stats_t *stats)
{
    const uint64_t HUGE_MASK = (1ULL << 7);
    uint64_t *pml4;
    uint32_t i;

    if (pml4_phys == 0 || !stats) {
        return;
    }

    stats->root_entries_present = 0;
    stats->leaf_entries_present = 0;
    stats->user_leaf_entries_present = 0;

    pml4 = (uint64_t *)paging_phys_to_virt(pml4_phys);
    if (!pml4) {
        return;
    }

    for (i = 0; i < 256; ++i) {
        uint64_t entry = pml4[i];
        uint64_t child_phys;

        if ((entry & AYKEN_PTE_PRESENT) == 0) {
            continue;
        }

        stats->root_entries_present += 1;
        child_phys = entry & AYKEN_PTE_ADDR_MASK;
        if (child_phys == 0) {
            continue;
        }

        if (entry & HUGE_MASK) {
            stats->leaf_entries_present += 1;
            if (entry & AYKEN_PTE_USER) {
                stats->user_leaf_entries_present += 1;
            }
            continue;
        }

        collect_low_half_mapping_stats_recursive(child_phys, 3, stats);
    }
}

static void emit_low_half_kheap_runtime_proof_raw(uint64_t user_pml4_phys,
                                                  uint32_t pid,
                                                  const char *phase)
{
    const uint64_t NX_MASK = (1ULL << 63);
    const uint64_t HIGHER_HALF_MIN = 0xFFFF800000000000ULL;
    low_half_mapping_stats_t lower_half_stats = {0};
    uint64_t pte;
    uint64_t active_cr3 = 0;
    uint64_t kernel_cr3 = paging_get_kernel_pml4_phys();
    uint64_t saved_rflags = 0;
    int switched_to_kernel_cr3 = 0;
    uint32_t seq = ++g_low_half_kheap_runtime_seq;

    __asm__ volatile("mov %%cr3, %0" : "=r"(active_cr3));
    if (kernel_cr3 &&
        ((active_cr3 & AYKEN_PTE_ADDR_MASK) != (kernel_cr3 & AYKEN_PTE_ADDR_MASK))) {
        __asm__ volatile("pushfq; popq %0" : "=r"(saved_rflags));
        __asm__ volatile("cli");
        __asm__ volatile("mov %0, %%cr3" :: "r"(kernel_cr3) : "memory");
        switched_to_kernel_cr3 = 1;
    }

    pte = paging_get_pte_in_pml4(user_pml4_phys, AYKEN_KHEAP_START);
    collect_low_half_mapping_stats(user_pml4_phys, &lower_half_stats);

    if (switched_to_kernel_cr3) {
        __asm__ volatile("mov %0, %%cr3" :: "r"(active_cr3) : "memory");
        if (saved_rflags & (1ULL << 9)) {
            __asm__ volatile("sti");
        }
    }

    debugcon_write("[[AYKEN_LOW_HALF_KHEAP_RUNTIME]]");
    debugcon_write(" phase=");
    debugcon_write(phase ? phase : "unknown");
    debugcon_write(" seq=");
    debugcon_u32(seq);
    debugcon_write(" pid=");
    debugcon_u32(pid);
    debugcon_write(" pml4=0x");
    debugcon_hex64(user_pml4_phys);
    debugcon_write(" kheap_start=0x");
    debugcon_hex64(AYKEN_KHEAP_START);
    debugcon_write(" kernel_virt_base=0x");
    debugcon_hex64(KERNEL_VIRT_BASE);
    debugcon_write(" pte=0x");
    debugcon_hex64(pte);
    debugcon_write(" present=");
    debugcon_write_char((pte & AYKEN_PTE_PRESENT) ? '1' : '0');
    debugcon_write(" user=");
    debugcon_write_char((pte & AYKEN_PTE_USER) ? '1' : '0');
    debugcon_write(" writable=");
    debugcon_write_char((pte & AYKEN_PTE_WRITABLE) ? '1' : '0');
    debugcon_write(" nx=");
    debugcon_write_char((pte & NX_MASK) ? '1' : '0');
    debugcon_write(" kheap_low_half=");
    debugcon_write_char((AYKEN_KHEAP_START < HIGHER_HALF_MIN) ? '1' : '0');
    debugcon_write(" kernel_higher_half=");
    debugcon_write_char((KERNEL_VIRT_BASE >= HIGHER_HALF_MIN) ? '1' : '0');
    debugcon_write(" scaffold=");
#if AYKEN_LOW_HALF_KHEAP_SCAFFOLD_ACTIVE
    debugcon_write_char('1');
#else
    debugcon_write_char('0');
#endif
    debugcon_write(" lower_half_roots=");
    debugcon_u32(lower_half_stats.root_entries_present);
    debugcon_write(" lower_half_leaves=");
    debugcon_u32(lower_half_stats.leaf_entries_present);
    debugcon_write(" lower_half_user_leaves=");
    debugcon_u32(lower_half_stats.user_leaf_entries_present);
    debugcon_write("\n");
}
#endif

static int proc_switch_to_kernel_cr3(uint64_t *saved_active_cr3, uint64_t *saved_rflags)
{
    uint64_t active_cr3 = 0;
    uint64_t kernel_cr3 = paging_get_kernel_pml4_phys();

    if (!saved_active_cr3 || !saved_rflags || kernel_cr3 == 0) {
        return 0;
    }

    __asm__ volatile("mov %%cr3, %0" : "=r"(active_cr3));
    *saved_active_cr3 = active_cr3;
    *saved_rflags = 0;

    if ((active_cr3 & AYKEN_PTE_ADDR_MASK) == (kernel_cr3 & AYKEN_PTE_ADDR_MASK)) {
        return 0;
    }

    __asm__ volatile("pushfq; popq %0" : "=r"(*saved_rflags));
    __asm__ volatile("cli");
    __asm__ volatile("mov %0, %%cr3" :: "r"(kernel_cr3) : "memory");
    return 1;
}

static void proc_restore_cr3(uint64_t active_cr3, uint64_t saved_rflags, int switched)
{
    if (!switched) {
        return;
    }

    __asm__ volatile("mov %0, %%cr3" :: "r"(active_cr3) : "memory");
    if (saved_rflags & (1ULL << 9)) {
        __asm__ volatile("sti");
    }
}

void proc_emit_low_half_kheap_runtime_proof(proc_t *p, const char *phase)
{
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
    uint64_t user_pml4_phys;

    if (!p || p->type != PROC_TYPE_USER || p->pid <= 0 || !phase) {
        return;
    }

    user_pml4_phys = p->pml4_phys != 0 ? p->pml4_phys : p->context.cr3;
    if (user_pml4_phys == 0) {
        return;
    }

    emit_low_half_kheap_runtime_proof_raw(user_pml4_phys, (uint32_t)p->pid, phase);
#else
    (void)p;
    (void)phase;
#endif
}

#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    ((AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST == 1) || \
     (AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_SELFTEST == 1) || \
     (AYKEN_LOW_HALF_KHEAP_INTERLEAVING_PROOF_SELFTEST == 1))
static int proc_write_mailbox_candidate(proc_t *publisher,
                                        uint64_t epoch,
                                        uint32_t candidate_pid)
{
    ayken_sched_mailbox_t *mb;

    if (!publisher || publisher->mailbox_pa == 0 || publisher->pid <= 0) {
        return 0;
    }

    mb = (ayken_sched_mailbox_t *)paging_phys_to_virt(publisher->mailbox_pa);
    if (!mb) {
        return 0;
    }

    mb->magic = AYKEN_SCHED_MB_MAGIC;
    mb->version = AYKEN_SCHED_MB_VERSION;
    mb->kind = AYKEN_SCHED_HINT_CANDIDATE;
    mb->epoch = epoch;
    mb->proposer_pid = (uint32_t)publisher->pid;
    mb->candidate_pid = candidate_pid;
    mb->flags = 0;
    mb->status = AYKEN_SCHED_STATUS_EMPTY;
    mb->reject_reason = AYKEN_SCHED_REJECT_NONE;
    mb->reserved = 0;
    return 1;
}

static int proc_seed_mailbox_candidate(proc_t *publisher, uint32_t candidate_pid)
{
    uint64_t next_epoch;

    if (!publisher) {
        return 0;
    }

    next_epoch = publisher->mailbox_last_epoch + 1;
    if (next_epoch == 0) {
        next_epoch = 1;
    }

    return proc_write_mailbox_candidate(publisher, next_epoch, candidate_pid);
}

static int proc_reset_mailbox_to_self(proc_t *publisher)
{
    if (!publisher || publisher->pid <= 0) {
        return 0;
    }

    return proc_write_mailbox_candidate(publisher, 1, (uint32_t)publisher->pid);
}

static int proc_validate_low_half_kheap_exit_round(proc_t *owner_proc,
                                                   proc_t *controller_proc,
                                                   uint32_t exit_pid,
                                                   uint32_t return_pid,
                                                   int *switch_from_pid_out,
                                                   int *switch_to_pid_out)
{
    proc_t *exit_proc = NULL;
    int switch_seen = 0;
    int switch_from_pid = 0;
    int switch_to_pid = 0;

    if (!owner_proc || !controller_proc || exit_pid == 0 || return_pid == 0) {
        return 0;
    }

    switch_seen = sched_validation_take_exit_switch_event(&switch_from_pid, &switch_to_pid);
    sched_validation_disarm_exit_successor();

    if (switch_from_pid_out) {
        *switch_from_pid_out = switch_from_pid;
    }
    if (switch_to_pid_out) {
        *switch_to_pid_out = switch_to_pid;
    }

    if (!proc_reset_mailbox_to_self(owner_proc)) {
        return 0;
    }

    exit_proc = proc_find_by_pid((int)exit_pid);
    if (!switch_seen ||
        controller_proc->pid != (int)return_pid ||
        switch_from_pid != (int)exit_pid ||
        switch_to_pid != (int)return_pid ||
        current_proc != controller_proc ||
        !exit_proc ||
        exit_proc->state != PROC_ZOMBIE) {
        return 0;
    }

    return 1;
}

#if AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST == 1
static int proc_finish_low_half_kheap_exit_proof_selftest(proc_t *owner_proc)
{
    proc_t *controller_proc = current_proc;

    if (!g_low_half_kheap_exit_selftest_armed) {
        return 1;
    }

    if (!controller_proc ||
        !proc_validate_low_half_kheap_exit_round(owner_proc,
                                                 controller_proc,
                                                 g_low_half_kheap_exit_selftest_exit_pid,
                                                 g_low_half_kheap_exit_selftest_return_pid,
                                                 NULL,
                                                 NULL)) {
        debugcon_write("[[AYKEN_LOW_HALF_KHEAP_EXIT_SELFTEST_FAIL]] switch_contract\n");
        return 0;
    }

    g_low_half_kheap_exit_selftest_armed = 0;
    g_low_half_kheap_exit_selftest_completed = 1;

    debugcon_write("[[AYKEN_LOW_HALF_KHEAP_EXIT_SELFTEST_OK]] exit_pid=");
    debugcon_u32(g_low_half_kheap_exit_selftest_exit_pid);
    debugcon_write(" return_pid=");
    debugcon_u32(g_low_half_kheap_exit_selftest_return_pid);
    debugcon_write("\n");
    return 1;
}

static int proc_run_low_half_kheap_exit_proof_selftest(proc_t *owner_proc)
{
    proc_t *controller_proc = current_proc;
    proc_t *exit_proc = NULL;

    if (!controller_proc || controller_proc->pid <= 0 || !owner_proc ||
        owner_proc->type != PROC_TYPE_USER || owner_proc->pid <= 0) {
        debugcon_write("[[AYKEN_LOW_HALF_KHEAP_EXIT_SELFTEST_FAIL]] bad_context\n");
        return 0;
    }

    exit_proc = proc_create_user_process("phase10-low-half-exit-proof",
                                         low_half_kheap_exit_proof_code,
                                         sizeof(low_half_kheap_exit_proof_code),
                                         PROC_IMAGE_FLAT);
    if (!exit_proc || exit_proc->type != PROC_TYPE_USER) {
        debugcon_write("[[AYKEN_LOW_HALF_KHEAP_EXIT_SELFTEST_FAIL]] create_exit_proc\n");
        return 0;
    }

    debugcon_write("[[AYKEN_LOW_HALF_KHEAP_EXIT_SELFTEST_ARMED]] owner_pid=");
    debugcon_u32((uint32_t)owner_proc->pid);
    debugcon_write(" exit_pid=");
    debugcon_u32((uint32_t)exit_proc->pid);
    debugcon_write("\n");

    if (!proc_seed_mailbox_candidate(owner_proc, (uint32_t)exit_proc->pid)) {
        debugcon_write("[[AYKEN_LOW_HALF_KHEAP_EXIT_SELFTEST_FAIL]] seed_owner_mailbox\n");
        return 0;
    }

    g_low_half_kheap_exit_selftest_armed = 1;
    g_low_half_kheap_exit_selftest_exit_pid = (uint32_t)exit_proc->pid;
    g_low_half_kheap_exit_selftest_return_pid = (uint32_t)controller_proc->pid;
    sched_validation_arm_exit_successor(controller_proc);
    sched_yield();
    return proc_finish_low_half_kheap_exit_proof_selftest(owner_proc);
}
#endif

#if AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_SELFTEST == 1
static void proc_emit_low_half_kheap_multi_exit_lineage(uint32_t slot,
                                                        uint32_t total,
                                                        uint32_t owner_pid,
                                                        uint32_t exit_pid,
                                                        uint32_t return_pid,
                                                        int switch_from_pid,
                                                        int switch_to_pid)
{
    debugcon_write("[[AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_LINEAGE]] slot=");
    debugcon_u32(slot);
    debugcon_write(" total=");
    debugcon_u32(total);
    debugcon_write(" owner_pid=");
    debugcon_u32(owner_pid);
    debugcon_write(" exit_pid=");
    debugcon_u32(exit_pid);
    debugcon_write(" return_pid=");
    debugcon_u32(return_pid);
    debugcon_write(" switch_from_pid=");
    debugcon_u32((uint32_t)switch_from_pid);
    debugcon_write(" switch_to_pid=");
    debugcon_u32((uint32_t)switch_to_pid);
    debugcon_write("\n");
}

static void proc_emit_low_half_kheap_multi_exit_ok(void)
{
    uint32_t i;

    debugcon_write("[[AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_SELFTEST_OK]] owner_pid=");
    debugcon_u32(g_low_half_kheap_multi_exit_owner_pid);
    debugcon_write(" total=");
    debugcon_u32(LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT);
    debugcon_write(" return_pid=");
    debugcon_u32(g_low_half_kheap_multi_exit_return_pid);
    debugcon_write(" exit_pids=");
    for (i = 0; i < LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT; ++i) {
        if (i != 0) {
            debugcon_write_char(',');
        }
        debugcon_u32(g_low_half_kheap_multi_exit_exit_pids[i]);
    }
    debugcon_write("\n");
}

static int proc_finish_low_half_kheap_multi_exit_proof_selftest(proc_t *owner_proc)
{
    proc_t *controller_proc = current_proc;
    uint32_t slot = g_low_half_kheap_multi_exit_current_slot;
    int switch_from_pid = 0;
    int switch_to_pid = 0;

    if (!g_low_half_kheap_multi_exit_selftest_armed) {
        if (!g_low_half_kheap_multi_exit_selftest_completed &&
            slot >= LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT) {
            g_low_half_kheap_multi_exit_selftest_completed = 1;
            proc_emit_low_half_kheap_multi_exit_ok();
        }
        return 1;
    }

    if (!controller_proc ||
        slot >= LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT ||
        !proc_validate_low_half_kheap_exit_round(
            owner_proc,
            controller_proc,
            g_low_half_kheap_multi_exit_exit_pids[slot],
            g_low_half_kheap_multi_exit_return_pid,
            &switch_from_pid,
            &switch_to_pid)) {
        debugcon_write("[[AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_SELFTEST_FAIL]] slot=");
        debugcon_u32(slot + 1u);
        debugcon_write(" reason=switch_contract\n");
        return 0;
    }

    proc_emit_low_half_kheap_multi_exit_lineage(slot + 1u,
                                                LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT,
                                                g_low_half_kheap_multi_exit_owner_pid,
                                                g_low_half_kheap_multi_exit_exit_pids[slot],
                                                g_low_half_kheap_multi_exit_return_pid,
                                                switch_from_pid,
                                                switch_to_pid);

    g_low_half_kheap_multi_exit_selftest_armed = 0;
    g_low_half_kheap_multi_exit_current_slot = slot + 1u;
    if (g_low_half_kheap_multi_exit_current_slot >= LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT) {
        g_low_half_kheap_multi_exit_selftest_completed = 1;
        proc_emit_low_half_kheap_multi_exit_ok();
    }

    return 1;
}

static int proc_run_low_half_kheap_multi_exit_proof_selftest(proc_t *owner_proc)
{
    proc_t *controller_proc = current_proc;
    proc_t *exit_proc = NULL;
    uint32_t slot = g_low_half_kheap_multi_exit_current_slot;

    if (!controller_proc || controller_proc->pid <= 0 || !owner_proc ||
        owner_proc->type != PROC_TYPE_USER || owner_proc->pid <= 0) {
        debugcon_write("[[AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_SELFTEST_FAIL]] slot=0 reason=bad_context\n");
        return 0;
    }

    if (g_low_half_kheap_multi_exit_selftest_completed) {
        return 1;
    }

    if (g_low_half_kheap_multi_exit_selftest_armed) {
        return proc_finish_low_half_kheap_multi_exit_proof_selftest(owner_proc);
    }

    if (slot >= LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT) {
        g_low_half_kheap_multi_exit_selftest_completed = 1;
        proc_emit_low_half_kheap_multi_exit_ok();
        return 1;
    }

    exit_proc = proc_create_user_process("phase10-low-half-exit-proof",
                                         low_half_kheap_exit_proof_code,
                                         sizeof(low_half_kheap_exit_proof_code),
                                         PROC_IMAGE_FLAT);
    if (!exit_proc || exit_proc->type != PROC_TYPE_USER) {
        debugcon_write("[[AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_SELFTEST_FAIL]] slot=");
        debugcon_u32(slot + 1u);
        debugcon_write(" reason=create_exit_proc\n");
        return 0;
    }

    if (slot == 0) {
        g_low_half_kheap_multi_exit_owner_pid = (uint32_t)owner_proc->pid;
        g_low_half_kheap_multi_exit_return_pid = (uint32_t)controller_proc->pid;
    } else if (g_low_half_kheap_multi_exit_owner_pid != (uint32_t)owner_proc->pid ||
               g_low_half_kheap_multi_exit_return_pid != (uint32_t)controller_proc->pid) {
        debugcon_write("[[AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_SELFTEST_FAIL]] slot=");
        debugcon_u32(slot + 1u);
        debugcon_write(" reason=context_drift\n");
        return 0;
    }

    g_low_half_kheap_multi_exit_exit_pids[slot] = (uint32_t)exit_proc->pid;
    debugcon_write("[[AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_SELFTEST_ARMED]] slot=");
    debugcon_u32(slot + 1u);
    debugcon_write(" total=");
    debugcon_u32(LOW_HALF_KHEAP_MULTI_EXIT_PROOF_COUNT);
    debugcon_write(" owner_pid=");
    debugcon_u32((uint32_t)owner_proc->pid);
    debugcon_write(" exit_pid=");
    debugcon_u32((uint32_t)exit_proc->pid);
    debugcon_write(" return_pid=");
    debugcon_u32((uint32_t)controller_proc->pid);
    debugcon_write("\n");

    if (!proc_seed_mailbox_candidate(owner_proc, (uint32_t)exit_proc->pid)) {
        debugcon_write("[[AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_SELFTEST_FAIL]] slot=");
        debugcon_u32(slot + 1u);
        debugcon_write(" reason=seed_owner_mailbox\n");
        return 0;
    }

    g_low_half_kheap_multi_exit_selftest_armed = 1;
    sched_validation_arm_exit_successor(controller_proc);
    sched_yield();
    return proc_finish_low_half_kheap_multi_exit_proof_selftest(owner_proc);
}
#endif

#if AYKEN_LOW_HALF_KHEAP_INTERLEAVING_PROOF_SELFTEST == 1
static void proc_emit_low_half_kheap_interleaving_prepared(uint32_t slot,
                                                           uint32_t total,
                                                           uint32_t owner_pid,
                                                           uint32_t exit_pid,
                                                           uint32_t return_pid)
{
    debugcon_write("[[AYKEN_LOW_HALF_KHEAP_INTERLEAVING_SELFTEST_PREPARED]] slot=");
    debugcon_u32(slot);
    debugcon_write(" total=");
    debugcon_u32(total);
    debugcon_write(" owner_pid=");
    debugcon_u32(owner_pid);
    debugcon_write(" exit_pid=");
    debugcon_u32(exit_pid);
    debugcon_write(" return_pid=");
    debugcon_u32(return_pid);
    debugcon_write("\n");
}

static void proc_emit_low_half_kheap_interleaving_armed(uint32_t slot,
                                                        uint32_t total,
                                                        uint32_t owner_pid,
                                                        uint32_t exit_pid,
                                                        uint32_t return_pid)
{
    debugcon_write("[[AYKEN_LOW_HALF_KHEAP_INTERLEAVING_SELFTEST_ARMED]] slot=");
    debugcon_u32(slot);
    debugcon_write(" total=");
    debugcon_u32(total);
    debugcon_write(" owner_pid=");
    debugcon_u32(owner_pid);
    debugcon_write(" exit_pid=");
    debugcon_u32(exit_pid);
    debugcon_write(" return_pid=");
    debugcon_u32(return_pid);
    debugcon_write("\n");
}

static void proc_emit_low_half_kheap_interleaving_lineage(uint32_t slot,
                                                          uint32_t total,
                                                          uint32_t owner_pid,
                                                          uint32_t exit_pid,
                                                          uint32_t return_pid,
                                                          int switch_from_pid,
                                                          int switch_to_pid)
{
    debugcon_write("[[AYKEN_LOW_HALF_KHEAP_INTERLEAVING_LINEAGE]] slot=");
    debugcon_u32(slot);
    debugcon_write(" total=");
    debugcon_u32(total);
    debugcon_write(" owner_pid=");
    debugcon_u32(owner_pid);
    debugcon_write(" exit_pid=");
    debugcon_u32(exit_pid);
    debugcon_write(" return_pid=");
    debugcon_u32(return_pid);
    debugcon_write(" switch_from_pid=");
    debugcon_u32((uint32_t)switch_from_pid);
    debugcon_write(" switch_to_pid=");
    debugcon_u32((uint32_t)switch_to_pid);
    debugcon_write("\n");
}

static void proc_emit_low_half_kheap_interleaving_ok(void)
{
    uint32_t i;

    debugcon_write("[[AYKEN_LOW_HALF_KHEAP_INTERLEAVING_SELFTEST_OK]] owner_pid=");
    debugcon_u32(g_low_half_kheap_interleaving_owner_pid);
    debugcon_write(" total=");
    debugcon_u32(LOW_HALF_KHEAP_INTERLEAVING_PROOF_COUNT);
    debugcon_write(" return_pid=");
    debugcon_u32(g_low_half_kheap_interleaving_return_pid);
    debugcon_write(" exit_pids=");
    for (i = 0; i < LOW_HALF_KHEAP_INTERLEAVING_PROOF_COUNT; ++i) {
        if (i != 0) {
            debugcon_write_char(',');
        }
        debugcon_u32(g_low_half_kheap_interleaving_exit_pids[i]);
    }
    debugcon_write(" prepared_upfront=1\n");
}

static int proc_prepare_low_half_kheap_interleaving_exit_set(proc_t *owner_proc,
                                                             proc_t *controller_proc)
{
    uint32_t slot;

    if (g_low_half_kheap_interleaving_selftest_prepared) {
        return 1;
    }

    g_low_half_kheap_interleaving_owner_pid = (uint32_t)owner_proc->pid;
    g_low_half_kheap_interleaving_return_pid = (uint32_t)controller_proc->pid;
    for (slot = 0; slot < LOW_HALF_KHEAP_INTERLEAVING_PROOF_COUNT; ++slot) {
        proc_t *exit_proc = proc_create_user_process("phase10-low-half-interleave-exit-proof",
                                                     low_half_kheap_exit_proof_code,
                                                     sizeof(low_half_kheap_exit_proof_code),
                                                     PROC_IMAGE_FLAT);
        if (!exit_proc || exit_proc->type != PROC_TYPE_USER) {
            debugcon_write("[[AYKEN_LOW_HALF_KHEAP_INTERLEAVING_SELFTEST_FAIL]] slot=");
            debugcon_u32(slot + 1u);
            debugcon_write(" reason=create_exit_proc\n");
            return 0;
        }

        g_low_half_kheap_interleaving_exit_pids[slot] = (uint32_t)exit_proc->pid;
        proc_emit_low_half_kheap_interleaving_prepared(slot + 1u,
                                                       LOW_HALF_KHEAP_INTERLEAVING_PROOF_COUNT,
                                                       g_low_half_kheap_interleaving_owner_pid,
                                                       (uint32_t)exit_proc->pid,
                                                       g_low_half_kheap_interleaving_return_pid);
    }

    g_low_half_kheap_interleaving_selftest_prepared = 1;
    return 1;
}

static int proc_finish_low_half_kheap_interleaving_proof_selftest(proc_t *owner_proc)
{
    proc_t *controller_proc = current_proc;
    uint32_t slot = g_low_half_kheap_interleaving_current_slot;
    int switch_from_pid = 0;
    int switch_to_pid = 0;

    if (!g_low_half_kheap_interleaving_selftest_armed) {
        if (!g_low_half_kheap_interleaving_selftest_completed &&
            slot >= LOW_HALF_KHEAP_INTERLEAVING_PROOF_COUNT) {
            g_low_half_kheap_interleaving_selftest_completed = 1;
            proc_emit_low_half_kheap_interleaving_ok();
        }
        return 1;
    }

    if (!controller_proc ||
        slot >= LOW_HALF_KHEAP_INTERLEAVING_PROOF_COUNT ||
        !proc_validate_low_half_kheap_exit_round(
            owner_proc,
            controller_proc,
            g_low_half_kheap_interleaving_exit_pids[slot],
            g_low_half_kheap_interleaving_return_pid,
            &switch_from_pid,
            &switch_to_pid)) {
        debugcon_write("[[AYKEN_LOW_HALF_KHEAP_INTERLEAVING_SELFTEST_FAIL]] slot=");
        debugcon_u32(slot + 1u);
        debugcon_write(" reason=switch_contract\n");
        return 0;
    }

    proc_emit_low_half_kheap_interleaving_lineage(slot + 1u,
                                                  LOW_HALF_KHEAP_INTERLEAVING_PROOF_COUNT,
                                                  g_low_half_kheap_interleaving_owner_pid,
                                                  g_low_half_kheap_interleaving_exit_pids[slot],
                                                  g_low_half_kheap_interleaving_return_pid,
                                                  switch_from_pid,
                                                  switch_to_pid);

    g_low_half_kheap_interleaving_selftest_armed = 0;
    g_low_half_kheap_interleaving_current_slot = slot + 1u;
    if (g_low_half_kheap_interleaving_current_slot >= LOW_HALF_KHEAP_INTERLEAVING_PROOF_COUNT) {
        g_low_half_kheap_interleaving_selftest_completed = 1;
        proc_emit_low_half_kheap_interleaving_ok();
    }

    return 1;
}

static int proc_run_low_half_kheap_interleaving_proof_selftest(proc_t *owner_proc)
{
    proc_t *controller_proc = current_proc;
    uint32_t slot = g_low_half_kheap_interleaving_current_slot;

    if (!controller_proc || controller_proc->pid <= 0 || !owner_proc ||
        owner_proc->type != PROC_TYPE_USER || owner_proc->pid <= 0) {
        debugcon_write("[[AYKEN_LOW_HALF_KHEAP_INTERLEAVING_SELFTEST_FAIL]] slot=0 reason=bad_context\n");
        return 0;
    }

    if (g_low_half_kheap_interleaving_selftest_completed) {
        return 1;
    }

    if (!g_low_half_kheap_interleaving_selftest_prepared) {
        if (!proc_prepare_low_half_kheap_interleaving_exit_set(owner_proc, controller_proc)) {
            return 0;
        }
    } else if (g_low_half_kheap_interleaving_owner_pid != (uint32_t)owner_proc->pid ||
               g_low_half_kheap_interleaving_return_pid != (uint32_t)controller_proc->pid) {
        debugcon_write("[[AYKEN_LOW_HALF_KHEAP_INTERLEAVING_SELFTEST_FAIL]] slot=");
        debugcon_u32(slot + 1u);
        debugcon_write(" reason=context_drift\n");
        return 0;
    }

    if (g_low_half_kheap_interleaving_selftest_armed) {
        return proc_finish_low_half_kheap_interleaving_proof_selftest(owner_proc);
    }

    if (slot >= LOW_HALF_KHEAP_INTERLEAVING_PROOF_COUNT) {
        g_low_half_kheap_interleaving_selftest_completed = 1;
        proc_emit_low_half_kheap_interleaving_ok();
        return 1;
    }

    proc_emit_low_half_kheap_interleaving_armed(slot + 1u,
                                                LOW_HALF_KHEAP_INTERLEAVING_PROOF_COUNT,
                                                g_low_half_kheap_interleaving_owner_pid,
                                                g_low_half_kheap_interleaving_exit_pids[slot],
                                                g_low_half_kheap_interleaving_return_pid);

    if (!proc_seed_mailbox_candidate(owner_proc, g_low_half_kheap_interleaving_exit_pids[slot])) {
        debugcon_write("[[AYKEN_LOW_HALF_KHEAP_INTERLEAVING_SELFTEST_FAIL]] slot=");
        debugcon_u32(slot + 1u);
        debugcon_write(" reason=seed_owner_mailbox\n");
        return 0;
    }

    g_low_half_kheap_interleaving_selftest_armed = 1;
    sched_validation_arm_exit_successor(controller_proc);
    sched_yield();
    return proc_finish_low_half_kheap_interleaving_proof_selftest(owner_proc);
}
#endif
#endif

static int proc_alloc_pid(void)
{
    // A real implementation would reuse PIDs. For now, we just increment.
    return next_pid++;
}

static void proc_remove_from_table(proc_t *p)
{
    int i;

    if (!p) {
        return;
    }

    for (i = 0; i < MAX_PROCS; ++i) {
        if (proc_table[i] == p) {
            proc_table[i] = NULL;
            break;
        }
    }
}

static void proc_enqueue_deferred_reap(proc_t *p)
{
    int i;

    if (!p) {
        return;
    }

    for (i = 0; i < MAX_PROCS; ++i) {
        if (g_deferred_reap_queue[i] == p) {
            return;
        }
    }

    for (i = 0; i < MAX_PROCS; ++i) {
        if (g_deferred_reap_queue[i] == NULL) {
            g_deferred_reap_queue[i] = p;
            return;
        }
    }
}

void proc_drain_deferred_reap(void)
{
    proc_t *active_proc = current_proc;
    int i;

    for (i = 0; i < MAX_PROCS; ++i) {
        proc_t *p = g_deferred_reap_queue[i];
        user_as_t as;

        if (!p) {
            continue;
        }
        if (p == active_proc) {
            continue;
        }

        if (p->context.rsp0 != 0) {
            kfree((void *)(uintptr_t)(p->context.rsp0 - AYKEN_FRAME_SIZE));
            p->context.rsp0 = 0;
        }

        if (p->pml4_phys != 0) {
            as.cr3_phys = p->pml4_phys;
            as.pml4_virt = (uint64_t *)paging_phys_to_virt(p->pml4_phys);
            user_as_destroy_root(&as);
            p->pml4_phys = 0;
            p->context.cr3 = 0;
        }

        g_deferred_reap_queue[i] = NULL;
    }
}

proc_t* proc_find_by_pid(int pid)
{
    uint64_t active_cr3 = 0;
    uint64_t kernel_cr3 = paging_get_kernel_pml4_phys();
    uint64_t saved_rflags = 0;
    int switched_to_kernel_cr3 = 0;

    __asm__ volatile("mov %%cr3, %0" : "=r"(active_cr3));
    if (kernel_cr3 &&
        ((active_cr3 & AYKEN_PTE_ADDR_MASK) != (kernel_cr3 & AYKEN_PTE_ADDR_MASK))) {
        __asm__ volatile("pushfq; popq %0" : "=r"(saved_rflags));
        __asm__ volatile("cli");
        __asm__ volatile("mov %0, %%cr3" :: "r"(kernel_cr3) : "memory");
        switched_to_kernel_cr3 = 1;
    }

    proc_t *found = NULL;
    for (int i = 0; i < MAX_PROCS; ++i) {
        if (proc_table[i] && proc_table[i]->pid == pid) {
            found = proc_table[i];
            break;
        }
    }

    if (switched_to_kernel_cr3) {
        __asm__ volatile("mov %0, %%cr3" :: "r"(active_cr3) : "memory");
        if (saved_rflags & (1ULL << 9)) {
            __asm__ volatile("sti");
        }
    }

    return found;
}


typedef struct {
    unsigned char e_ident[16];
    uint16_t e_type;
    uint16_t e_machine;
    uint32_t e_version;
    uint64_t e_entry;
    uint64_t e_phoff;
    uint64_t e_shoff;
    uint32_t e_flags;
    uint16_t e_ehsize;
    uint16_t e_phentsize;
    uint16_t e_phnum;
    uint16_t e_shentsize;
    uint16_t e_shnum;
    uint16_t e_shstrndx;
} elf64_ehdr_t;

typedef struct {
    uint32_t p_type;
    uint32_t p_flags;
    uint64_t p_offset;
    uint64_t p_vaddr;
    uint64_t p_paddr;
    uint64_t p_filesz;
    uint64_t p_memsz;
    uint64_t p_align;
} elf64_phdr_t;

static proc_t *proc_alloc(proc_type_t type, const char *name)
{
    proc_t *p = (proc_t *)kmalloc(sizeof(proc_t));
    if (!p) return NULL;

    memset(p, 0, sizeof(proc_t));
    p->pid = proc_alloc_pid();
    p->type = type;
    p->state = PROC_READY;
    p->execution_role = (type == PROC_TYPE_USER)
        ? PROC_EXECUTION_ROLE_USER
        : PROC_EXECUTION_ROLE_KERNEL;
    p->name = name;
    // PML4 will be set by caller (proc_create_user_process or proc_create_kernel_process)
    p->pml4_phys = 0;  // Initialize to 0, will be set by caller
    p->context.cr3 = 0;  // Initialize to 0, will be set by caller
    p->context.rflags = 0x202;  // IF=1, reserved bits
    p->next_mapping_id = 1;
    
    // Add to process table
    int found_slot = 0;
    for (int i = 0; i < MAX_PROCS; ++i) {
        if (proc_table[i] == NULL) {
            proc_table[i] = p;
            found_slot = 1;
            break;
        }
    }
    if (!found_slot) {
        kfree(p);
        return NULL; // No space in proc_table
    }

    // Set segment selectors based on process type
    if (type == PROC_TYPE_USER) {
        // Ring3: User code and user data segments
        p->context.cs = GDT_USER_CODE;   // User CS
        p->context.ss = GDT_USER_DATA;   // User SS
    } else {
        // Ring0: Kernel code and kernel data segments
        p->context.cs = GDT_KERNEL_CODE;   // Kernel CS
        p->context.ss = GDT_KERNEL_DATA;   // Kernel SS
    }
    
    // rsp0 will be set when the process is created with a kernel stack
    p->context.rsp0 = 0;
    
    return p;
}

proc_mapping_entry_t *proc_find_generic_mapping(proc_t *p, uint64_t user_va)
{
    uint32_t i;

    if (!p || user_va == 0) {
        return NULL;
    }

    for (i = 0; i < AYKEN_MAX_PROC_GENERIC_MAPPINGS; ++i) {
        proc_mapping_entry_t *entry = &p->mapping_ledger[i];
        if (!entry->in_use) {
            continue;
        }
        if (entry->mapping_class != PROC_MAPPING_CLASS_GENERIC) {
            continue;
        }
        if (entry->user_va == user_va) {
            return entry;
        }
    }

    return NULL;
}

int proc_record_generic_mapping(proc_t *p,
                                uint64_t user_va,
                                uint64_t phys_addr,
                                uint64_t flags,
                                uint64_t capability_id,
                                uint64_t page_count,
                                uint64_t *out_map_id)
{
    uint32_t i;
    proc_mapping_entry_t *entry = NULL;

    if (!p || user_va == 0 || phys_addr == 0 || page_count == 0) {
        return -1;
    }

    if (proc_find_generic_mapping(p, user_va) != NULL) {
        return -1;
    }

    if (p->next_mapping_id == 0) {
        return -1;
    }

    for (i = 0; i < AYKEN_MAX_PROC_GENERIC_MAPPINGS; ++i) {
        if (!p->mapping_ledger[i].in_use) {
            entry = &p->mapping_ledger[i];
            break;
        }
    }

    if (!entry) {
        return -1;
    }

    memset(entry, 0, sizeof(*entry));
    entry->in_use = 1;
    entry->map_id = p->next_mapping_id++;
    if (entry->map_id == 0) {
        memset(entry, 0, sizeof(*entry));
        return -1;
    }
    entry->owner_pid = (uint64_t)p->pid;
    entry->user_va = user_va;
    entry->phys_addr = phys_addr;
    entry->flags = flags;
    entry->capability_id = capability_id;
    entry->page_count = page_count;
    entry->mapping_class = PROC_MAPPING_CLASS_GENERIC;

    if (out_map_id) {
        *out_map_id = entry->map_id;
    }

    return 0;
}

int proc_remove_generic_mapping(proc_t *p,
                                uint64_t user_va,
                                proc_mapping_entry_t *removed_entry)
{
    proc_mapping_entry_t *entry;

    if (!p || user_va == 0) {
        return -1;
    }

    entry = proc_find_generic_mapping(p, user_va);
    if (!entry) {
        return -1;
    }

    if (removed_entry) {
        *removed_entry = *entry;
    }
    memset(entry, 0, sizeof(*entry));
    return 0;
}

uint32_t proc_revoke_generic_mappings(proc_t *p)
{
    uint32_t i;
    uint32_t revoked = 0;

    if (!p || p->pml4_phys == 0) {
        return 0;
    }

    for (i = 0; i < AYKEN_MAX_PROC_GENERIC_MAPPINGS; ++i) {
        proc_mapping_entry_t *entry = &p->mapping_ledger[i];
        uint64_t page;

        if (!entry->in_use || entry->mapping_class != PROC_MAPPING_CLASS_GENERIC) {
            continue;
        }

        for (page = 0; page < entry->page_count; ++page) {
            paging_unmap_in_pml4(p->pml4_phys,
                                 entry->user_va + (page * AYKEN_FRAME_SIZE));
        }

        memset(entry, 0, sizeof(*entry));
        revoked++;
    }

    return revoked;
}

static void proc_release_execution_delivery_surfaces(proc_t *p)
{
    uint32_t i;

    if (!p) {
        return;
    }

    if (p->execution_inbox_pa != 0) {
        uint8_t *dst = (uint8_t *)paging_phys_to_virt(p->execution_inbox_pa);
        if (dst) {
            memset(dst, 0, AYKEN_FRAME_SIZE);
        }
        phys_free_frame(p->execution_inbox_pa);
        p->execution_inbox_pa = 0;
    }

    for (i = 0; i < AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES; ++i) {
        if (p->execution_payload_pas[i] == 0) {
            continue;
        }
        {
            uint8_t *dst = (uint8_t *)paging_phys_to_virt(p->execution_payload_pas[i]);
            if (dst) {
                memset(dst, 0, AYKEN_FRAME_SIZE);
            }
        }
        phys_free_frame(p->execution_payload_pas[i]);
        p->execution_payload_pas[i] = 0;
    }

    p->execution_delivery_seq = 0;
}

static void proc_release_mailbox_surface(proc_t *p, int defer_owner_surface)
{
    if (!p || p->mailbox_pa == 0) {
        return;
    }

    /*
     * The scheduler owner mailbox remains authoritative during mailbox-first
     * successor selection. Do not tear its backing down synchronously on the
     * no-return exit path; a later reap slice can reclaim it once ownership is
     * transferred safely.
     */
    if (defer_owner_surface && (uint32_t)p->pid == sched_active_owner_pid()) {
        return;
    }

    {
        uint8_t *dst = (uint8_t *)paging_phys_to_virt(p->mailbox_pa);
        if (dst) {
            memset(dst, 0, AYKEN_FRAME_SIZE);
        }
    }
    phys_free_frame(p->mailbox_pa);
    p->mailbox_pa = 0;
    p->mailbox_last_epoch = 0;
}

static void proc_invalidate_local_page_if_active(uint64_t pml4_phys, uint64_t virt_addr)
{
    uint64_t active_cr3 = 0;

    __asm__ volatile("mov %%cr3, %0" : "=r"(active_cr3));
    if ((active_cr3 & AYKEN_PTE_ADDR_MASK) == (pml4_phys & AYKEN_PTE_ADDR_MASK)) {
        __asm__ volatile("invlpg (%0)" :: "r"(virt_addr) : "memory");
    }
}

static int proc_verify_execution_output_mapping(uint64_t user_pml4,
                                                uint64_t virt_addr,
                                                uint64_t phys_addr)
{
    uint64_t pte = paging_get_pte_in_pml4(user_pml4, virt_addr);

    if (pte == 0) {
        return -1;
    }
    if ((pte & AYKEN_PTE_ADDR_MASK) != phys_addr) {
        return -1;
    }
    if ((pte & AYKEN_PTE_USER) == 0) {
        return -1;
    }
    if ((pte & AYKEN_PTE_WRITABLE) == 0) {
        return -1;
    }
    if ((pte & AYKEN_PTE_NO_EXEC) == 0) {
        return -1;
    }

    return 0;
}

int proc_bind_execution_output_window(proc_t *p,
                                      const uint64_t *output_pas,
                                      uint32_t frame_count,
                                      uint64_t execution_id)
{
    uint32_t i;
    uint8_t mapped_pages[AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES] = {0};

    if (!p || !output_pas || p->pml4_phys == 0 || execution_id == 0) {
        return -1;
    }
    if (frame_count != AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES) {
        return -1;
    }
    if (p->execution_output_mapped_id != 0) {
        return -1;
    }

    for (i = 0; i < frame_count; ++i) {
        uint64_t page_va = EXECUTION_OUTPUT_VA + ((uint64_t)i * AYKEN_FRAME_SIZE);
        uint64_t page_phys = output_pas[i];

        if (page_phys == 0) {
            goto fail;
        }

        paging_map_page_in_pml4(p->pml4_phys,
                                page_va,
                                page_phys,
                                AYKEN_PTE_USER | AYKEN_PTE_WRITABLE | AYKEN_PTE_NO_EXEC);
        proc_invalidate_local_page_if_active(p->pml4_phys, page_va);
        mapped_pages[i] = 1;

        if (proc_verify_execution_output_mapping(p->pml4_phys, page_va, page_phys) != 0) {
            goto fail;
        }
    }

    p->execution_output_mapped_id = execution_id;
    return 0;

fail:
    for (i = 0; i < frame_count; ++i) {
        uint64_t page_va = EXECUTION_OUTPUT_VA + ((uint64_t)i * AYKEN_FRAME_SIZE);

        if (!mapped_pages[i]) {
            continue;
        }

        paging_unmap_in_pml4(p->pml4_phys, page_va);
        proc_invalidate_local_page_if_active(p->pml4_phys, page_va);
    }
    p->execution_output_mapped_id = 0;
    return -1;
}

void proc_unmap_execution_output_window(proc_t *p)
{
    uint32_t i;

    if (!p || p->pml4_phys == 0) {
        return;
    }

    for (i = 0; i < AYKEN_EXECUTION_OUTPUT_WINDOW_PAGES; ++i) {
        uint64_t page_va = EXECUTION_OUTPUT_VA + ((uint64_t)i * AYKEN_FRAME_SIZE);

        paging_unmap_in_pml4(p->pml4_phys, page_va);
        proc_invalidate_local_page_if_active(p->pml4_phys, page_va);
    }

    p->execution_output_mapped_id = 0;
}

static void proc_unmap_execution_delivery_surfaces(proc_t *p)
{
    uint32_t i;

    if (!p || p->pml4_phys == 0) {
        return;
    }

    paging_unmap_in_pml4(p->pml4_phys, EXECUTION_INBOX_VA);
    for (i = 0; i < AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES; ++i) {
        paging_unmap_in_pml4(p->pml4_phys,
                             EXECUTION_PAYLOAD_VA + ((uint64_t)i * AYKEN_FRAME_SIZE));
    }
}

void proc_teardown_exit_surfaces(proc_t *p,
                                 const uint64_t *result_vas,
                                 const uint64_t *hash_vas,
                                 uint32_t result_count)
{
    uint32_t i;
    proc_t *active_proc = current_proc;
    user_as_t user_as;
    uint64_t saved_active_cr3 = 0;
    uint64_t saved_rflags = 0;
    int switched_to_kernel_cr3 = 0;

    if (!p) {
        return;
    }

    switched_to_kernel_cr3 = proc_switch_to_kernel_cr3(&saved_active_cr3, &saved_rflags);

    if (p->pml4_phys != 0) {
        (void)proc_revoke_generic_mappings(p);

        for (i = 0; i < result_count; ++i) {
            uint32_t page;
            if (result_vas == NULL || result_vas[i] == 0) {
                continue;
            }
            for (page = 0; page < AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES; ++page) {
                paging_unmap_in_pml4(p->pml4_phys,
                                     result_vas[i] + ((uint64_t)page * AYKEN_FRAME_SIZE));
            }
            if (hash_vas != NULL && hash_vas[i] != 0) {
                paging_unmap_in_pml4(p->pml4_phys, hash_vas[i]);
            }
        }

        proc_unmap_execution_delivery_surfaces(p);
        proc_unmap_execution_output_window(p);
        if (p->mailbox_pa != 0) {
            paging_unmap_in_pml4(p->pml4_phys, SCHED_MAILBOX_VA);
        }
    }

    proc_release_execution_delivery_surfaces(p);
    proc_release_mailbox_surface(p, 1);

    user_as.cr3_phys = p->pml4_phys;
    user_as.pml4_virt = p->pml4_phys != 0
        ? (uint64_t *)paging_phys_to_virt(p->pml4_phys)
        : NULL;
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
    proc_emit_low_half_kheap_runtime_proof(p, "exit_teardown_pre");
#endif
    user_as_destroy_lower_half(&user_as);
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
    proc_emit_low_half_kheap_runtime_proof(p, "exit_teardown_post");
#endif

    /*
     * The exiting process may still be running on its current Ring0 stack until
     * sched_exit_current() completes the final context switch. Leave current
     * rsp0 backing for deferred reap in that path.
     */
    if (p->context.rsp0 != 0 && p != active_proc) {
        kfree((void *)(uintptr_t)(p->context.rsp0 - 4096));
        p->context.rsp0 = 0;
    }

    if (p->pml4_phys != 0 && p != active_proc) {
        user_as_destroy_root(&user_as);
        p->pml4_phys = 0;
        p->context.cr3 = 0;
    } else if (p == active_proc) {
        proc_enqueue_deferred_reap(p);
        switched_to_kernel_cr3 = 0;
    }

    proc_restore_cr3(saved_active_cr3, saved_rflags, switched_to_kernel_cr3);
}

#if defined(AYKEN_VALIDATION)
/* ============================================================================
 * exit_teardown_alias_phase: Alias eşlemelerini temizler ve doğrulama yapar
 * ============================================================================
 * 
 * Süreç çıkışı sırasında alias_registry'deki tüm alias eşlemelerini PML4'ten
 * temizler, TLB flush yapar ve verifier ile doğrulama gerçekleştirir.
 * 
 * Önkoşullar:
 * - proc != NULL
 * - proc->state == PROC_ZOMBIE
 * - proc->teardown_started == 1 (FREEZE INVARIANT)
 * 
 * FREEZE INVARIANT: teardown_started=1 iken sys_v2_map_memory() bu proc için
 * -EINVAL döner. Yani teardown başladıktan sonra yeni alias kaydı gelmez;
 * verifier penceresi temizdir.
 * 
 * Sonkoşullar:
 * - Tüm alias VA'lar için paging_get_pte_in_pml4() == 0
 * - debugcon'da [[AYKEN_ALIAS_PROOF_OK]] witness mevcut
 * 
 * Fail-closed: leaked_count > 0 ise halt_forever() çağrılır
 * (MEMORY.LEAK.INTENTIONAL NON_OVERRIDABLE kuralı)
 * 
 * CANONICAL/ALIAS MEKANİK SINIR: Bu fonksiyon yalnızca proc->alias_reg üzerinde
 * döngü kurar; proc->mapping_ledger'a hiçbir koşulda dokunmaz. Bu ayrım kod
 * seviyesinde mekanik olmalı: alias_reg döngüsü ve mapping_ledger döngüsü aynı
 * fonksiyonda birleştirilmemeli, ayrı scope'larda tutulmalı. Canonical VA
 * yanlışlıkla silinirse test geçer ama veri modeli sessizce bozulur.
 * 
 * TLB FLUSH ZORUNLU: Her alias VA için invlpg(va) çağrısı zorunludur.
 * proc_invalidate_local_page_if_active() gerçekten invlpg instruction'ı ürettiği
 * kaynak koddan doğrulanmıştır (kernel/proc/proc.c:1427). pte == 0 kontrolü tek
 * başına yeterli değil — TLB'de eski mapping kalabilir; bu olmadan tasarım
 * "page-table-proof" olur, "leak-proof" olmaz.
 * 
 * Validates: Requirements 5.1, 6.6, 7.3, 4.1, 6.4, 6.5
 */
void exit_teardown_alias_phase(proc_t *proc)
{
    alias_proof_result_t result = {0};
    int verdict;

    if (proc == NULL) {
        return;
    }

    /* Önkoşul kontrolü: proc->state == PROC_ZOMBIE */
    if (proc->state != PROC_ZOMBIE) {
        fb_print("[ALIAS_PROOF] ERROR: exit_teardown_alias_phase called on non-ZOMBIE process (pid=");
        fb_print_int(proc->pid);
        fb_print(")\n");
        return;
    }

    /* CANONICAL/ALIAS MEKANİK SINIR: Yalnızca alias_reg üzerinde döngü kur.
     * mapping_ledger'a dokunma — canonical lineage korunumu (Requirement 7).
     * 
     * Bu scope yalnızca alias VA'ları temizler. Canonical VA'lar
     * user_as_destroy_lower_half() tarafından zaten temizlenmiştir.
     */
    {
        alias_registry_t *reg = &proc->alias_reg;

        /* Adım 1: Tüm alias VA'ları PML4'ten temizle ve TLB flush yap
         * 
         * İç içe döngü: entry_count × alias_count
         * Her alias VA için:
         * 1. paging_unmap_in_pml4() — PTE'yi sıfırla
         * 2. proc_invalidate_local_page_if_active() — TLB entry'yi geçersiz kıl
         * 
         * TLB FLUSH ZORUNLU: proc_invalidate_local_page_if_active() gerçekten
         * invlpg instruction'ı üretir (kaynak doğrulanmış: kernel/proc/proc.c:1427).
         * 
         * Memory ordering: teardown_started=1 set edildiğinde smp_wmb() + smp_mb()
         * ile tüm alias_registry_record() yazmaları globally visible yapılmıştır
         * (bkz. kernel/sys/syscall_v2.c:1335-1352). Bu noktada registry snapshot
         * temizdir ve freeze invariant aktiftir.
         */
        for (uint32_t i = 0; i < reg->entry_count; i++) {
            alias_entry_t *entry = &reg->entries[i];

            /* Kullanılmayan entry'leri atla */
            if (entry->in_use == 0) {
                continue;
            }

            /* Her alias VA için PTE temizleme ve TLB flush */
            for (uint32_t j = 0; j < entry->alias_count; j++) {
                uint64_t va = entry->alias_vas[j];

                /* PTE'yi PML4'ten temizle */
                paging_unmap_in_pml4(proc->pml4_phys, va);

                /* TLB entry'yi geçersiz kıl (ZORUNLU)
                 * 
                 * KAYNAK KOD DOĞRULAMA: proc_invalidate_local_page_if_active()
                 * implementasyonu kernel/proc/proc.c:1422-1430 satırlarında
                 * doğrulanmıştır. Fonksiyon gerçekten invlpg instruction'ı üretir:
                 * 
                 *   __asm__ volatile("invlpg (%0)" :: "r"(virt_addr) : "memory");
                 * 
                 * Bu olmadan pte == 0 kontrolü yeterli değildir — TLB'de eski
                 * mapping kalabilir ve bu tasarım "leak-proof" sayılamaz.
                 */
                proc_invalidate_local_page_if_active(proc->pml4_phys, va);
            }
        }
    }

    /* Adım 2: Verifier çağrısı — tüm alias VA'ların temizlendiğini doğrula
     * 
     * alias_verifier_run() her alias VA için paging_get_pte_in_pml4() çağırır
     * ve PTE == 0 kontrolü yapar. Yan etki yoktur — registry değişmez.
     * 
     * Validates: Requirements 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8
     */
    verdict = alias_verifier_run(proc, &result);

    /* Adım 3: Kanıt yayını — debugcon'a deterministik format ile yaz
     * 
     * Çıktı formatı:
     * - leaked_count == 0: [[AYKEN_ALIAS_PROOF_OK]] pid=<N> total=<M> verified=<M> leaked=0 tlb_scope=local
     * - leaked_count > 0: [[AYKEN_ALIAS_LEAK_DETECTED]] pid=<N> total=<M> verified=<V> leaked=<L> first_va=0x<VA> first_phys=0x<PA> tlb_scope=local
     * 
     * tlb_scope=local: v1'in yalnızca local-core TLB flush garantilediğini,
     * remote-core TLB shootdown'ın kapsam dışı olduğunu proof report yüzeyinde
     * açıkça taşır. CI gate bu alanı parse ederek kapsam sınırını evidence'a yansıtır.
     * 
     * Validates: Requirements 6.1, 6.2, 6.3, 6.4
     */
    alias_verifier_emit_proof(&result, proc->pid);

    /* Adım 4: Fail-closed enforcement — sızıntı varsa halt_forever()
     * 
     * MEMORY.LEAK.INTENTIONAL NON_OVERRIDABLE kuralının doğrudan uygulaması.
     * leaked_count > 0 ise sistem durur; sessiz başarısızlığa izin verilmez.
     * 
     * Validates: Requirements 6.5, 6.6
     */
    if (verdict != 0) {
        fb_print("[[AYKEN_ALIAS_LEAK_DETECTED]]\n");
        fb_print("[ALIAS_PROOF] FATAL: Alias leak detected in process (pid=");
        fb_print_int(proc->pid);
        fb_print("), halting system\n");
        for (;;) {
            __asm__ volatile("cli; hlt");
        }
    }
}
#endif /* AYKEN_VALIDATION */

static void proc_cleanup_failed_user_process(proc_t *p)
{
    user_as_t user_as;

    if (!p) {
        return;
    }

    if (p->pml4_phys != 0) {
        proc_unmap_execution_delivery_surfaces(p);
        if (p->mailbox_pa != 0) {
            paging_unmap_in_pml4(p->pml4_phys, SCHED_MAILBOX_VA);
        }
    }

    proc_release_execution_delivery_surfaces(p);
    proc_release_mailbox_surface(p, 0);

    if (p->context.rsp0 != 0) {
        kfree((void *)(uintptr_t)(p->context.rsp0 - AYKEN_FRAME_SIZE));
        p->context.rsp0 = 0;
    }

    user_as.cr3_phys = p->pml4_phys;
    user_as.pml4_virt = p->pml4_phys != 0
        ? (uint64_t *)paging_phys_to_virt(p->pml4_phys)
        : NULL;
    user_as_destroy(&user_as);
    p->pml4_phys = 0;
    p->context.cr3 = 0;

    proc_remove_from_table(p);
    kfree(p);
}

static int proc_verify_execution_delivery_mapping(uint64_t user_pml4,
                                                  uint64_t virt_addr,
                                                  uint64_t phys_addr)
{
    uint64_t pte = paging_get_pte_in_pml4(user_pml4, virt_addr);

    if (pte == 0) {
        return -1;
    }
    if ((pte & AYKEN_PTE_ADDR_MASK) != phys_addr) {
        return -1;
    }
    if ((pte & AYKEN_PTE_USER) == 0) {
        return -1;
    }
    if ((pte & AYKEN_PTE_WRITABLE) != 0) {
        return -1;
    }
    if ((pte & AYKEN_PTE_NO_EXEC) == 0) {
        return -1;
    }

    return 0;
}

static int proc_map_execution_delivery_surfaces(proc_t *p, uint64_t user_pml4)
{
    uint64_t inbox_pa = 0;
    uint64_t payload_pas[AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES] = {0};
    uint32_t i;
    ayken_execution_inbox_v1_t *inbox;

    if (!p || user_pml4 == 0) {
        return -1;
    }

    inbox_pa = phys_alloc_frame();
    if (!inbox_pa) {
        return -1;
    }

    {
        uint8_t *dst = (uint8_t *)paging_phys_to_virt(inbox_pa);
        if (!dst) {
            phys_free_frame(inbox_pa);
            return -1;
        }
        memset(dst, 0, AYKEN_FRAME_SIZE);
    }

    for (i = 0; i < AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES; ++i) {
        uint8_t *dst;

        payload_pas[i] = phys_alloc_frame();
        if (!payload_pas[i]) {
            goto fail;
        }

        dst = (uint8_t *)paging_phys_to_virt(payload_pas[i]);
        if (!dst) {
            goto fail;
        }
        memset(dst, 0, AYKEN_FRAME_SIZE);
    }

    paging_map_page_in_pml4(user_pml4, EXECUTION_INBOX_VA, inbox_pa,
                            AYKEN_PTE_USER | AYKEN_PTE_READ_ONLY | AYKEN_PTE_NO_EXEC);
    for (i = 0; i < AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES; ++i) {
        paging_map_page_in_pml4(user_pml4,
                                EXECUTION_PAYLOAD_VA + ((uint64_t)i * AYKEN_FRAME_SIZE),
                                payload_pas[i],
                                AYKEN_PTE_USER | AYKEN_PTE_READ_ONLY | AYKEN_PTE_NO_EXEC);
    }

    if (proc_verify_execution_delivery_mapping(user_pml4, EXECUTION_INBOX_VA, inbox_pa) != 0) {
        goto fail;
    }
    for (i = 0; i < AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES; ++i) {
        if (proc_verify_execution_delivery_mapping(user_pml4,
                                                   EXECUTION_PAYLOAD_VA + ((uint64_t)i * AYKEN_FRAME_SIZE),
                                                   payload_pas[i]) != 0) {
            goto fail;
        }
    }

    p->execution_inbox_pa = inbox_pa;
    for (i = 0; i < AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES; ++i) {
        p->execution_payload_pas[i] = payload_pas[i];
    }
    p->execution_delivery_seq = 0;

    inbox = (ayken_execution_inbox_v1_t *)paging_phys_to_virt(inbox_pa);
    if (!inbox) {
        proc_release_execution_delivery_surfaces(p);
        return -1;
    }

    inbox->magic = AYKEN_EXECUTION_INBOX_MAGIC;
    inbox->version = AYKEN_EXECUTION_INBOX_VERSION;
    inbox->state = AXIB_STATE_EMPTY;
    inbox->delivery_seq = 0;
    inbox->execution_id = 0;
    inbox->target_context_id = 0;
    inbox->bcib_user_va = EXECUTION_PAYLOAD_VA;
    inbox->bcib_size = 0;
    inbox->bcib_window_size = AYKEN_EXECUTION_PAYLOAD_WINDOW_SIZE;
    inbox->flags = 0;

    return 0;

fail:
    if (inbox_pa != 0) {
        uint8_t *dst = (uint8_t *)paging_phys_to_virt(inbox_pa);
        if (dst) {
            memset(dst, 0, AYKEN_FRAME_SIZE);
        }
        phys_free_frame(inbox_pa);
    }
    for (i = 0; i < AYKEN_EXECUTION_PAYLOAD_WINDOW_PAGES; ++i) {
        if (payload_pas[i] == 0) {
            continue;
        }
        {
            uint8_t *dst = (uint8_t *)paging_phys_to_virt(payload_pas[i]);
            if (dst) {
                memset(dst, 0, AYKEN_FRAME_SIZE);
            }
        }
        phys_free_frame(payload_pas[i]);
    }
    return -1;
}

static uint64_t load_flat_image(uint64_t pml4_phys, const uint8_t *image, uint64_t size)
{
    uint64_t phys = proc_alloc_user_image_frame();
    if (!phys)
        return 0;

    uint8_t *dst = (uint8_t *)paging_phys_to_virt(phys);
    memset(dst, 0, AYKEN_FRAME_SIZE);

    uint64_t copy = size < AYKEN_FRAME_SIZE ? size : AYKEN_FRAME_SIZE;
    if (image && copy)
        memcpy(dst, image, copy);

    debugcon_write("[CODE]=");
    debugcon_hex8(dst[0]);
    outb(0xE9, (uint8_t)'\n');

    paging_map_page_in_pml4(pml4_phys, USER_TEXT_BASE, phys,
                            AYKEN_PTE_USER | AYKEN_PTE_WRITABLE);
    return USER_TEXT_BASE;
}

static uint64_t load_elf_image(uint64_t pml4_phys, const uint8_t *image, uint64_t size)
{
    if (!image || size < sizeof(elf64_ehdr_t))
        return 0;

    const elf64_ehdr_t *ehdr = (const elf64_ehdr_t *)image;
    if (!(ehdr->e_ident[0] == 0x7F && ehdr->e_ident[1] == 'E' &&
          ehdr->e_ident[2] == 'L' && ehdr->e_ident[3] == 'F')) {
        return 0;
    }

    if (ehdr->e_phoff + (uint64_t)ehdr->e_phnum * sizeof(elf64_phdr_t) > size)
        return 0;

    const elf64_phdr_t *phdr = (const elf64_phdr_t *)(image + ehdr->e_phoff);
    for (uint16_t i = 0; i < ehdr->e_phnum; ++i) {
        if (phdr[i].p_type != 1) // PT_LOAD
            continue;

        uint64_t offset = phdr[i].p_offset;
        uint64_t filesz = phdr[i].p_filesz;
        uint64_t memsz  = phdr[i].p_memsz;
        uint64_t vaddr  = phdr[i].p_vaddr;

        for (uint64_t off = 0; off < memsz; off += AYKEN_FRAME_SIZE) {
            uint64_t phys = proc_alloc_user_image_frame();
            if (!phys)
                return 0;

            uint8_t *dst = (uint8_t *)paging_phys_to_virt(phys);
            memset(dst, 0, AYKEN_FRAME_SIZE);

            if (off < filesz) {
                uint64_t copy = filesz - off < AYKEN_FRAME_SIZE ? (filesz - off) : AYKEN_FRAME_SIZE;
                if (offset + off + copy <= size)
                    memcpy(dst, image + offset + off, copy);
            }

            paging_map_page_in_pml4(pml4_phys, vaddr + off, phys,
                                    AYKEN_PTE_USER | AYKEN_PTE_WRITABLE);
        }
    }

    return ehdr->e_entry;
}

static uint64_t load_user_image(proc_image_format_t fmt,
                                uint64_t pml4_phys,
                                const uint8_t *image,
                                uint64_t size)
{
    switch (fmt) {
    case PROC_IMAGE_ELF:
        return load_elf_image(pml4_phys, image, size);
    case PROC_IMAGE_FLAT:
    default:
        return load_flat_image(pml4_phys, image, size);
    }
}

void proc_init(void)
{
    fb_print("[proc] Process subsystem init.\n");
    for (int i = 0; i < MAX_PROCS; ++i) {
        proc_table[i] = NULL;
        g_deferred_reap_queue[i] = NULL;
    }
    next_pid = 1;
}

proc_t *proc_create_kernel_thread(void (*func)(void))
{
    proc_t *p = proc_alloc(PROC_TYPE_KERNEL, "kernel-thread");
    if (!p) return NULL;

    // Kernel threads use kernel PML4
    p->pml4_phys = paging_get_kernel_pml4_phys();
    p->context.cr3 = p->pml4_phys;

    uint64_t stack = (uint64_t)kmalloc(4096);
    p->stack_top = stack + 4096;

    p->context.rip = (uint64_t)func;
    p->context.rsp = p->stack_top - 8;  // SysV ABI: entry %rsp = 8 mod 16

    sched_add(p);
    return p;
}

static proc_t *proc_create_init_process(void)
{
    proc_t *p = proc_alloc(PROC_TYPE_KERNEL, "init");
    if (!p) return NULL;

    // Init process uses kernel PML4
    p->pml4_phys = paging_get_kernel_pml4_phys();
    p->context.cr3 = p->pml4_phys;

    uint64_t stack = (uint64_t)kmalloc(4096);
    p->stack_top = stack + 4096;

    p->context.rip = (uint64_t)kernel_first_entry;  // Back to simple entry for debugging
    p->context.rsp = p->stack_top - 8;  // SysV ABI: entry %rsp = 8 mod 16
    p->context.rflags = 0x202;  // IF=1 (bit 9) + reserved bit 1
    
    // Ring0 process: set CS/SS to kernel segments
    p->context.cs = GDT_KERNEL_CODE;  // 0x08
    p->context.ss = GDT_KERNEL_DATA;  // 0x10
    p->context.rsp0 = 0;  // Not needed for Ring0 process

    // DEBUG: Log init process context
    fb_print("[proc_init] PID=");
    fb_print_int(p->pid);
    fb_print(" RIP=");
    fb_print_hex64((uint64_t)kernel_first_entry);
    fb_print(" RSP=");
    fb_print_hex64(p->context.rsp);
    fb_print(" CS=");
    fb_print_hex(p->context.cs);
    fb_print("\n");

    sched_add(p);
    return p;
}

proc_t *proc_create_user_process(const char *name,
                                 const uint8_t *image,
                                 uint64_t image_size,
                                 proc_image_format_t fmt)
{
    // [K][USER_ELF_PARSE_BEGIN] - Entry point marker
    debugcon_write("[K][USER_ELF_PARSE_BEGIN] name=");
    debugcon_write(name ? name : "<null>");
    debugcon_write(" image_type=");
    debugcon_hex64((uint64_t)fmt);
    debugcon_write(" image_size=");
    debugcon_hex64(image_size);
    debugcon_write(" image_ptr=");
    debugcon_hex64((uint64_t)image);
    debugcon_write("\n");
    
    outb(0xE9, (uint8_t)'U');
    proc_t *p = proc_alloc(PROC_TYPE_USER, name);
    if (!p) {
        outb(0xE9, (uint8_t)'1');
        return NULL;
    }

    uint64_t user_pml4 = paging_create_user_pml4();
    if (!user_pml4) {
        outb(0xE9, (uint8_t)'2');
        goto fail;
    }

    p->pml4_phys = user_pml4;
    p->context.cr3 = user_pml4;
    proc_debug_emit_ring3_creation_snapshot(user_pml4);

    uint64_t entry = load_user_image(fmt, user_pml4, image, image_size);
    if (!entry) {
        outb(0xE9, (uint8_t)'3');
        goto fail;
    }
    debug_dump_pte(user_pml4, USER_TEXT_BASE, "code");
    proc_emit_user_text_root_witness(user_pml4, "load");
    
    // [K][USER_ELF_LOADED] - ELF parsed and loaded successfully
    debugcon_write("[K][USER_ELF_LOADED] pid=");
    debugcon_u32((uint32_t)p->pid);
    debugcon_write(" entry=");
    debugcon_hex64(entry);
    debugcon_write(" cr3=");
    debugcon_hex64(user_pml4);
    debugcon_write("\n");

    // User stack: 2 pages in user space
    for (int i = 0; i < 2; ++i) {
        uint64_t phys = phys_alloc_frame();
        if (!phys) {
            outb(0xE9, (uint8_t)'4');
            goto fail;
        }
        uint64_t virt = USER_STACK_TOP - (i + 1) * AYKEN_FRAME_SIZE;
        uint8_t *dst = (uint8_t *)paging_phys_to_virt(phys);
        memset(dst, 0, AYKEN_FRAME_SIZE);
        paging_map_page_in_pml4(user_pml4, virt, phys,
                                AYKEN_PTE_USER | AYKEN_PTE_WRITABLE);
    }

    // Scratch page for Ring3 INT80 diagnostics (pre/post syscall canary + result buffer)
    uint64_t canary_phys = phys_alloc_frame();
    if (!canary_phys) {
        outb(0xE9, (uint8_t)'5');
        goto fail;
    }
    uint8_t *canary_dst = (uint8_t *)paging_phys_to_virt(canary_phys);
    memset(canary_dst, 0, AYKEN_FRAME_SIZE);
    paging_map_page_in_pml4(user_pml4, RING3_CANARY_ADDR, canary_phys,
                            AYKEN_PTE_USER | AYKEN_PTE_WRITABLE);

    // MVP-1: Allocate and map per-process mailbox at fixed VA (0x700000)
    // This enables Ring3 → Ring0 scheduler bridge communication
    /*
     * Gate-4 publish/accept proof reads this surface back under a non-kernel
     * CR3 after Ring3 has authored it. Keep the mailbox leaf out of the
     * low-phys frame class for the same MMU-visible reason as user text.
     */
    uint64_t mb_pa = phys_alloc_frame_high();
    if (!mb_pa) {
        outb(0xE9, (uint8_t)'6');
        goto fail;
    }
    // Zero-init mailbox frame (mandatory for security)
    uint8_t *mb_dst = (uint8_t *)paging_phys_to_virt(mb_pa);
    memset(mb_dst, 0, AYKEN_FRAME_SIZE);
    // Map mailbox to fixed VA with USER | WRITABLE | PRESENT
    paging_map_page_in_pml4(user_pml4, SCHED_MAILBOX_VA, mb_pa,
                            AYKEN_PTE_USER | AYKEN_PTE_WRITABLE);
    // Store mailbox physical address and initialize epoch tracking
    p->mailbox_pa = mb_pa;
    p->mailbox_last_epoch = 0;
#if defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)
    p->gate4_publish_emitted = 0;
    p->gate4_accept_epoch1_emitted = 0;
#endif
    // Bootstrap mailbox contract so first scheduler handoff has deterministic data.
    // Ring3 code still advances epoch to publish fresh decisions.
    ayken_sched_mailbox_t *mb = (ayken_sched_mailbox_t *)mb_dst;
    mb->magic = AYKEN_SCHED_MB_MAGIC;
    mb->version = AYKEN_SCHED_MB_VERSION;
    mb->kind = AYKEN_SCHED_HINT_CANDIDATE;
    mb->epoch = 1;
    mb->proposer_pid = (uint32_t)p->pid;
    mb->candidate_pid = (uint32_t)p->pid;
    mb->flags = 0;
    mb->status = AYKEN_SCHED_STATUS_EMPTY;
    mb->reject_reason = AYKEN_SCHED_REJECT_NONE;
    mb->reserved = 0;

    if (proc_map_execution_delivery_surfaces(p, user_pml4) != 0) {
        outb(0xE9, (uint8_t)'7');
        goto fail;
    }

    p->stack_top = USER_STACK_TOP;
    p->context.rip = entry;
    p->context.rsp = p->stack_top - 8;  // SysV ABI: entry %rsp = 8 mod 16
    p->context.rflags = 0x202;  // IF=1 + reserved bit 1
    
    // Allocate kernel stack for Ring0 during Ring3→Ring0 transitions (interrupts/syscalls)
    uint64_t kernel_stack = (uint64_t)kmalloc(4096);
    if (kernel_stack == 0) {
        fb_print("[proc] ERROR: kernel stack allocation failed.\n");
        goto fail;
    }
    p->context.rsp0 = kernel_stack + 4096;  // Top of kernel stack

    // DEBUG: Verify RSP0 is reachable through the copied kernel half in user CR3
    debugcon_write("Kernel stack (RSP0) mapping:\n");
    debug_dump_pte(user_pml4, p->context.rsp0 - 8, "rsp0-8");
    debug_dump_pte(user_pml4, p->context.rsp0 - AYKEN_FRAME_SIZE, "rsp0_page");
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1)
    proc_emit_low_half_kheap_runtime_proof(p, "create");
#endif
    proc_emit_user_text_root_witness(user_pml4, "ready");

    fb_print("[DBG] USER cr3=");
    fb_print_hex(p->context.cr3);
    fb_print(" (pml4_phys=");
    fb_print_hex(p->pml4_phys);
    fb_print(")\n");

    // [K][USER_PROC_READY] - Process ready for scheduler
    debugcon_write("[K][USER_PROC_READY] pid=");
    debugcon_u32((uint32_t)p->pid);
    debugcon_write(" rip=");
    debugcon_hex64(p->context.rip);
    debugcon_write(" rsp=");
    debugcon_hex64(p->context.rsp);
    debugcon_write(" rsp0=");
    debugcon_hex64(p->context.rsp0);
    debugcon_write(" cr3=");
    debugcon_hex64(p->context.cr3);
    debugcon_write(" cs=");
    debugcon_hex64(p->context.cs);
    debugcon_write(" ss=");
    debugcon_hex64(p->context.ss);
    debugcon_write("\n");

    sched_add(p);

    outb(0xE9, (uint8_t)'E');
    return p;

fail:
    proc_cleanup_failed_user_process(p);
    return NULL;
}

// MVP-3: Minimal Ring3 scheduler hint test code
// This is the SIMPLEST possible test: write mailbox, loop forever
static const uint8_t ring3_mvp3_sched_hint_test_code[] = {
    // Load mailbox address into rbx
    0x48, 0xBB, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00, 0x00, 0x00,  // mov rbx, 0x700000
    
    // Read current epoch: rax = [rbx + 0]
    0x48, 0x8B, 0x03,                                            // mov rax, [rbx]
    
    // Increment epoch: rax = rax + 1
    0x48, 0xFF, 0xC0,                                            // inc rax
    
    // Write candidate_pid = 1: [rbx + 8] = 1
    0xC7, 0x43, 0x08, 0x01, 0x00, 0x00, 0x00,                    // mov dword [rbx + 8], 1
    
    // Write new epoch: [rbx + 0] = rax
    0x48, 0x89, 0x03,                                            // mov [rbx], rax
    
    // Infinite loop: jmp $
    0xEB, 0xFE                                                   // jmp $
};

// Gate-3: Ring3 runtime validation test code
// Emits "R3OK" via syscall 1010 (debug_putchar) to prove Ring3 execution
static const uint8_t ring3_gate3_test_code[] = {
    // Emit 'R' via syscall 1010
    0xB8, 0xF2, 0x03, 0x00, 0x00,  // mov eax, 1010
    0xBF, 0x52, 0x00, 0x00, 0x00,  // mov edi, 'R'
    0xCD, 0x80,                    // int 0x80
    
    // Emit '3' via syscall 1010
    0xB8, 0xF2, 0x03, 0x00, 0x00,  // mov eax, 1010
    0xBF, 0x33, 0x00, 0x00, 0x00,  // mov edi, '3'
    0xCD, 0x80,                    // int 0x80
    
    // Emit 'O' via syscall 1010
    0xB8, 0xF2, 0x03, 0x00, 0x00,  // mov eax, 1010
    0xBF, 0x4F, 0x00, 0x00, 0x00,  // mov edi, 'O'
    0xCD, 0x80,                    // int 0x80
    
    // Emit 'K' via syscall 1010
    0xB8, 0xF2, 0x03, 0x00, 0x00,  // mov eax, 1010
    0xBF, 0x4B, 0x00, 0x00, 0x00,  // mov edi, 'K'
    0xCD, 0x80,                    // int 0x80
    
    // Infinite loop (kernel will preempt)
    0xEB, 0xFE                     // jmp $
};

static void proc_launch_gate3_ring3_test(void) __attribute__((unused));
static void proc_launch_gate3_ring3_test(void)
{
    fb_print("[Gate-3] =============================================\n");
    fb_print("[Gate-3] Ring3 Runtime Validation Test\n");
    fb_print("[Gate-3] =============================================\n");
    fb_print("[Gate-3] Creating Ring3 process...\n");
    
    // Create Ring3 process with flat image
    proc_t *test_proc = proc_create_user_process(
        "gate3-ring3-test",
        ring3_gate3_test_code,
        sizeof(ring3_gate3_test_code),
        PROC_IMAGE_FLAT
    );
    
    if (!test_proc) {
        fb_print("[Gate-3] ERROR: Failed to create Ring3 test process\n");
        fb_print("[Gate-3] =============================================\n");
        return;
    }
    
    fb_print("[Gate-3] Ring3 process created (PID=");
    fb_print_int(test_proc->pid);
    fb_print(")\n");
    fb_print("[Gate-3] Entry point: 0x");
    fb_print_hex(test_proc->context.rip);
    fb_print("\n");
    fb_print("[Gate-3] =============================================\n");
    fb_print("[Gate-3] Waiting for Ring3 marker validation...\n");
    fb_print("[Gate-3] Expected: [[AYKEN_RING3_OK]] after R3OK sequence\n");
    fb_print("[Gate-3] =============================================\n");
}

static void proc_launch_mvp3_sched_hint_test(void) __attribute__((unused));
static void proc_launch_mvp3_sched_hint_test(void)
{
    fb_print("[MVP-3] =============================================\n");
    fb_print("[MVP-3] Minimal Ring3 Scheduler Hint Test\n");
    fb_print("[MVP-3] =============================================\n");
    fb_print("[MVP-3] Creating Ring3 process...\n");
    
    // Create Ring3 process with flat image
    proc_t *test_proc = proc_create_user_process(
        "mvp3-sched-hint-test",
        ring3_mvp3_sched_hint_test_code,
        sizeof(ring3_mvp3_sched_hint_test_code),
        PROC_IMAGE_FLAT
    );
    
    if (!test_proc) {
        fb_print("[MVP-3] ERROR: Failed to create Ring3 test process\n");
        fb_print("[MVP-3] =============================================\n");
        return;
    }
    
    fb_print("[MVP-3] Ring3 process created (PID=");
    fb_print_int(test_proc->pid);
    fb_print(")\n");
    fb_print("[MVP-3] Entry point: 0x");
    fb_print_hex(test_proc->context.rip);
    fb_print("\n");
    fb_print("[MVP-3] Mailbox VA: 0x");
    fb_print_hex(SCHED_MAILBOX_VA);
    fb_print("\n");
    fb_print("[MVP-3] Mailbox PA: 0x");
    fb_print_hex(test_proc->mailbox_pa);
    fb_print("\n");
    fb_print("[MVP-3] =============================================\n");
    fb_print("[MVP-3] Waiting for timer tick validation...\n");
    fb_print("[MVP-3] Expected: [[AYKEN_SCHED_MB_ACCEPT]] pid=");
    fb_print_int(test_proc->pid);
    fb_print(" epoch=1\n");
    fb_print("[MVP-3] =============================================\n");
}

// Gate-4: Policy Accept Proof
// Ring3 writes mailbox ABI header + epoch=1, kernel seeds pid fields
// deterministically, then timer IRQ validates and emits ACCEPT.
// No syscalls, pure timer-driven validation.
#if defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)
#define MB_MAGIC_B0 ((uint8_t)((AYKEN_SCHED_MB_MAGIC >> 0) & 0xFF))
#define MB_MAGIC_B1 ((uint8_t)((AYKEN_SCHED_MB_MAGIC >> 8) & 0xFF))
#define MB_MAGIC_B2 ((uint8_t)((AYKEN_SCHED_MB_MAGIC >> 16) & 0xFF))
#define MB_MAGIC_B3 ((uint8_t)((AYKEN_SCHED_MB_MAGIC >> 24) & 0xFF))
#define MB_VERSION_B0 ((uint8_t)((AYKEN_SCHED_MB_VERSION >> 0) & 0xFF))
#define MB_VERSION_B1 ((uint8_t)((AYKEN_SCHED_MB_VERSION >> 8) & 0xFF))
#define MB_KIND_B0 ((uint8_t)((AYKEN_SCHED_HINT_CANDIDATE >> 0) & 0xFF))
#define MB_KIND_B1 ((uint8_t)((AYKEN_SCHED_HINT_CANDIDATE >> 8) & 0xFF))
#define GATE45_TARGET_PID 3u

static const uint8_t ring3_gate4_policy_code[] = {
    // Mailbox VA = 0x700000 (SCHED_MAILBOX_VA)
    // Structure offsets (from sched_mailbox_abi.h):
    //   +0:  magic (4 bytes)
    //   +4:  version (2 bytes)
    //   +6:  kind (2 bytes)
    //   +8:  epoch (8 bytes)
    //   +16: proposer_pid (4 bytes)
    //   +20: candidate_pid (4 bytes)
    
    // Load mailbox address into rbx
    0x48, 0xBB, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00, 0x00, 0x00, // mov rbx, 0x700000
    
    // Write magic = AYKEN_SCHED_MB_MAGIC (little-endian immediate bytes)
    0xB8, MB_MAGIC_B0, MB_MAGIC_B1, MB_MAGIC_B2, MB_MAGIC_B3,   // mov eax, AYKEN_SCHED_MB_MAGIC
    0x89, 0x03,                                                 // mov [rbx], eax
    
    // Write version = 1, kind = 1 (CANDIDATE)
    0x66, 0xC7, 0x43, 0x04, MB_VERSION_B0, MB_VERSION_B1,       // mov word [rbx+4], AYKEN_SCHED_MB_VERSION
    0x66, 0xC7, 0x43, 0x06, MB_KIND_B0, MB_KIND_B1,             // mov word [rbx+6], AYKEN_SCHED_HINT_CANDIDATE

#if AYKEN_GATE45_PROOF
    // Gate-4.5: force cross-target mailbox candidate (owner PID2 -> worker PID3).
    0xC7, 0x43, 0x14, (uint8_t)(GATE45_TARGET_PID & 0xFF), 0x00, 0x00, 0x00,
#endif
    
    // Write epoch = 1
    0x48, 0xB8, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov rax, 1
    0x48, 0x89, 0x43, 0x08,                                     // mov [rbx+8], rax

    // Infinite loop (timer IRQ will validate)
    0xF3, 0x90,                                                 // pause
    0xEB, 0xFC                                                  // jmp .-2
};

static const uint8_t ring3_gate45_worker_code[] = {
    0xF3, 0x90, // pause
    0xEB, 0xFC  // jmp .-2
};

static int gate4_seed_mailbox_pid(proc_t *p, uint32_t candidate_pid)
{
    if (!p || !p->mailbox_pa) {
        return 0;
    }
    ayken_sched_mailbox_t *mb = (ayken_sched_mailbox_t *)paging_phys_to_virt(p->mailbox_pa);
    if (!mb) {
        return 0;
    }
    mb->proposer_pid = (uint32_t)p->pid;
    mb->candidate_pid = candidate_pid;
    return 1;
}

void proc_launch_gate4_policy_test(void)
{
    fb_print("[Gate-4] =============================================\n");
    fb_print("[Gate-4] Policy Accept Proof\n");
    fb_print("[Gate-4] =============================================\n");
    fb_print("[Gate-4] Creating Ring3 policy test process...\n");
    
    // Create Ring3 process with flat image
    proc_t *test_proc = proc_create_user_process(
        "gate4-policy-test",
        ring3_gate4_policy_code,
        sizeof(ring3_gate4_policy_code),
        PROC_IMAGE_FLAT
    );
    
    if (!test_proc) {
        fb_print("[Gate-4] ERROR: Failed to create Ring3 test process\n");
        fb_print("[Gate-4] =============================================\n");
        return;
    }

    uint32_t candidate_pid = (uint32_t)test_proc->pid;
#if AYKEN_GATE45_PROOF
    proc_t *worker_proc = proc_create_user_process(
        "gate45-worker",
        ring3_gate45_worker_code,
        sizeof(ring3_gate45_worker_code),
        PROC_IMAGE_FLAT
    );
    if (!worker_proc) {
        fb_print("[Gate-4.5] ERROR: Failed to create worker process\n");
        fb_print("[Gate-4] =============================================\n");
        return;
    }
    if ((uint32_t)worker_proc->pid != GATE45_TARGET_PID) {
        fb_print("[Gate-4.5] ERROR: Worker PID drift (expected ");
        fb_print_int((int)GATE45_TARGET_PID);
        fb_print(", got ");
        fb_print_int(worker_proc->pid);
        fb_print(")\n");
        fb_print("[Gate-4] =============================================\n");
        return;
    }
    // Bootstrap must run owner first; cross-target decision is published by Ring3.
    candidate_pid = (uint32_t)test_proc->pid;
    fb_print("[Gate-4.5] Worker process created (PID=");
    fb_print_int(worker_proc->pid);
    fb_print(")\n");
#endif

    if (!gate4_seed_mailbox_pid(test_proc, candidate_pid)) {
        fb_print("[Gate-4] ERROR: Failed to seed mailbox PID fields\n");
        fb_print("[Gate-4] =============================================\n");
        return;
    }

    gate4_emit_pid_marker((uint32_t)test_proc->pid);
    
    fb_print("[Gate-4] Ring3 process created (PID=");
    fb_print_int(test_proc->pid);
    fb_print(")\n");
    fb_print("[Gate-4] Mailbox VA: 0x");
    fb_print_hex(SCHED_MAILBOX_VA);
    fb_print("\n");
    fb_print("[Gate-4] Mailbox PA: 0x");
    fb_print_hex(test_proc->mailbox_pa);
    fb_print("\n");
    fb_print("[Gate-4] =============================================\n");
    fb_print("[Gate-4] Ring3 will write mailbox header+epoch (epoch=1)\n");
    fb_print("[Gate-4] Kernel seeded proposer pid=");
    fb_print_int(test_proc->pid);
    fb_print(" candidate pid=");
    fb_print_int((int)candidate_pid);
    fb_print("\n");
    fb_print("[Gate-4] Timer IRQ will validate → ACCEPT marker\n");
    fb_print("[Gate-4] Expected: [[AYKEN_SCHED_MB_ACCEPT]] epoch=1 pid=");
    fb_print_int(test_proc->pid);
    fb_print("\n");
    fb_print("[Gate-4] =============================================\n");
}
#endif

// Forward declaration for runtime syscall contract launcher (defined below).
static void proc_launch_ring3_test(void);

// PID 1: init process
void init_process_main(void)
{
    outb(0xE9, (uint8_t)'I');
    fb_print("[init] PID1 running.\n");

    // Phase 10 scheduler-dispatch mode:
    // If a user process is already prepared before sched_start (PID2),
    // skip legacy runtime launchers and hand control to that process.
    proc_t *preloaded = proc_find_by_pid(2);
    if (preloaded && preloaded->type == PROC_TYPE_USER) {
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    (AYKEN_LOW_HALF_KHEAP_INTERLEAVING_PROOF_SELFTEST == 1)
        while (!g_low_half_kheap_interleaving_selftest_completed) {
            fb_print("[init] Running low-half kheap interleaving proof selftest.\n");
            if (!proc_run_low_half_kheap_interleaving_proof_selftest(preloaded)) {
                fb_print("[init] low-half kheap interleaving proof selftest failed.\n");
                for (;;) {
                    __asm__ volatile("cli; hlt");
                }
            }
        }
#elif defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    (AYKEN_LOW_HALF_KHEAP_MULTI_EXIT_PROOF_SELFTEST == 1)
        while (!g_low_half_kheap_multi_exit_selftest_completed) {
            fb_print("[init] Running low-half kheap multi-exit proof selftest.\n");
            if (!proc_run_low_half_kheap_multi_exit_proof_selftest(preloaded)) {
                fb_print("[init] low-half kheap multi-exit proof selftest failed.\n");
                for (;;) {
                    __asm__ volatile("cli; hlt");
                }
            }
        }
#elif defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    (AYKEN_LOW_HALF_KHEAP_EXIT_PROOF_SELFTEST == 1)
        if (!g_low_half_kheap_exit_selftest_completed) {
            if (g_low_half_kheap_exit_selftest_armed) {
                fb_print("[init] Completing low-half kheap exit proof selftest.\n");
                if (!proc_finish_low_half_kheap_exit_proof_selftest(preloaded)) {
                    fb_print("[init] low-half kheap exit proof selftest failed.\n");
                    for (;;) {
                        __asm__ volatile("cli; hlt");
                    }
                }
            } else {
                fb_print("[init] Running low-half kheap exit proof selftest.\n");
                if (!proc_run_low_half_kheap_exit_proof_selftest(preloaded)) {
                    fb_print("[init] low-half kheap exit proof selftest failed.\n");
                    for (;;) {
                        __asm__ volatile("cli; hlt");
                    }
                }
            }
        }
#endif
#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    (AYKEN_ALIAS_PROOF_SELFTEST == 1)
        /* alias proof selftest runs via kernel.c late-init with a dedicated
         * mock proc; no duplicate call needed here */
#endif
        fb_print("[init] Phase10 preloaded user process detected; yielding.\n");
        sched_block_current();
        for (;;)
            __asm__ volatile("hlt");
    }

#if defined(AYKEN_VALIDATION) && (AYKEN_VALIDATION == 1) && \
    defined(AYKEN_GATE4_POLICY_TEST) && (AYKEN_GATE4_POLICY_TEST == 1)
    // Gate-4 isolated mode: run only policy mailbox workload.
    proc_launch_gate4_policy_test();
#else
    // Launch deterministic Ring3 runtime contract workload.
    // This process emits both Gate-3 and syscall-v2 runtime markers.
    proc_launch_ring3_test();

    // Keep scheduler bridge runtime signal active for mailbox validation gates.
    proc_launch_mvp3_sched_hint_test();
#endif

    // Keep PID1 out of runqueue (blocked)
    sched_block_current();
    for (;;)
        __asm__ volatile("hlt");
}

void proc_create_init(void)
{
    proc_t *p = proc_create_init_process();
    if (p)
        fb_print("[proc] init process created (PID1).\n");
    else
        fb_print("[proc] init process creation FAILED.\n");
}

// AI service function removed in Phase 2.5 - Step C completion
// All AI functionality moved to Ring3 userspace

void proc_block_current(void *wait_obj)
{
    if (!current_proc)
        return;

    current_proc->wait_obj = wait_obj;
    sched_block_current();
}

void proc_wake_waiters(void *wait_obj)
{
    sched_wake_all(wait_obj);
}

// ============================================================================
// RING3 EXECUTION-CENTRIC SYSCALL TEST (Phase 2.5 Final)
// ============================================================================

// ============================================================================
// PHASE 4.5: TIMER PREEMPT TEST - TWO RING3 PROCESSES
// ============================================================================

// Process A:
// - Executes runtime syscall contract (time_query + capability bind/revoke).
// - Emits canonical marker [U][SYSCALL_OK].
// - Falls back to preempt loop that prints 'A' via debug_putc syscall.
#define RING3_RUNTIME_TOKEN_PLACEHOLDER 0x1122334455667788ULL

static const uint8_t ring3_process_a_code_template[] = {
    /* emit Gate-3 marker prefix: R3OK */
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x52, 0x00, 0x00, 0x00, 0xCD, 0x80, /* 'R' */
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x33, 0x00, 0x00, 0x00, 0xCD, 0x80, /* '3' */
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x4F, 0x00, 0x00, 0x00, 0xCD, 0x80, /* 'O' */
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x4B, 0x00, 0x00, 0x00, 0xCD, 0x80, /* 'K' */

    0xBB, 0x41, 0x00, 0x00, 0x00, /* mov ebx, 'A' */

    /* time_query(type=0, buffer=0x405080) */
    0xB8, 0xEE, 0x03, 0x00, 0x00, /* mov eax, 1006 */
    0x31, 0xFF,                   /* xor edi, edi */
    0xBE, 0x80, 0x50, 0x40, 0x00, /* mov esi, 0x405080 */
    0xCD, 0x80,                   /* int 0x80 */

    /* valid token @0x4050a0: {id, perms=1, type=8} */
    0x48, 0xB8, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, /* mov rax, placeholder */
    0x48, 0x89, 0x04, 0x25, 0xA0, 0x50, 0x40, 0x00,             /* mov [0x4050a0], rax */
    0xC7, 0x04, 0x25, 0xA8, 0x50, 0x40, 0x00, 0x01, 0x00, 0x00, 0x00, /* perms=1 */
    0xC7, 0x04, 0x25, 0xAC, 0x50, 0x40, 0x00, 0x08, 0x00, 0x00, 0x00, /* type=8 */

    /* capability_bind granted: ctx=2, token=0x4050a0 */
    0xB8, 0xEF, 0x03, 0x00, 0x00, /* mov eax, 1007 */
    0xBF, 0x02, 0x00, 0x00, 0x00, /* mov edi, 2 */
    0xBE, 0xA0, 0x50, 0x40, 0x00, /* mov esi, 0x4050a0 */
    0xCD, 0x80,                   /* int 0x80 */

    /* invalid token @0x4050b0: id=0 -> denied path */
    0x48, 0xC7, 0x04, 0x25, 0xB0, 0x50, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xC7, 0x04, 0x25, 0xB8, 0x50, 0x40, 0x00, 0x01, 0x00, 0x00, 0x00,
    0xC7, 0x04, 0x25, 0xBC, 0x50, 0x40, 0x00, 0x08, 0x00, 0x00, 0x00,

    /* capability_bind denied: ctx=2, token=0x4050b0 */
    0xB8, 0xEF, 0x03, 0x00, 0x00,
    0xBF, 0x02, 0x00, 0x00, 0x00,
    0xBE, 0xB0, 0x50, 0x40, 0x00,
    0xCD, 0x80,

    /* capability_revoke granted then denied */
    0xB8, 0xF0, 0x03, 0x00, 0x00,             /* mov eax, 1008 */
    0x48, 0x8B, 0x3C, 0x25, 0xA0, 0x50, 0x40, 0x00, /* mov rdi, [0x4050a0] */
    0xCD, 0x80,
    0xB8, 0xF0, 0x03, 0x00, 0x00,
    0x48, 0x8B, 0x3C, 0x25, 0xA0, 0x50, 0x40, 0x00,
    0xCD, 0x80,

    /* emit canonical marker: [U][SYSCALL_OK]\n */
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x5B, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x55, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x5D, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x5B, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x53, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x59, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x53, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x43, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x41, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x4C, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x4C, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x5F, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x4F, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x4B, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x5D, 0x00, 0x00, 0x00, 0xCD, 0x80,
    0xB8, 0xF2, 0x03, 0x00, 0x00, 0xBF, 0x0A, 0x00, 0x00, 0x00, 0xCD, 0x80,

    /* idle loop */
    0xF3, 0x90,                         /* pause */
    0xEB, 0xFC                          /* jmp .-2 */
};

static uint8_t ring3_process_a_code[sizeof(ring3_process_a_code_template)];

static int ring3_prepare_process_a_code(uint64_t capability_id)
{
    const uint64_t placeholder = RING3_RUNTIME_TOKEN_PLACEHOLDER;
    int replacements = 0;

    memcpy(ring3_process_a_code,
           ring3_process_a_code_template,
           sizeof(ring3_process_a_code_template));

    for (size_t i = 0; i + sizeof(uint64_t) <= sizeof(ring3_process_a_code); ++i) {
        uint64_t value = 0;
        memcpy(&value, &ring3_process_a_code[i], sizeof(value));
        if (value == placeholder) {
            memcpy(&ring3_process_a_code[i], &capability_id, sizeof(capability_id));
            replacements++;
        }
    }

    if (replacements != 1) {
        fb_print("[preempt_test] ERROR: capability placeholder patch failed\n");
        return 0;
    }

    return 1;
}

// Process B: CPU hog that prints 'B' via syscall
static const uint8_t ring3_process_b_code[] __attribute__((unused)) = {
    // ebx = 'B'
    0xBB, 0x42, 0x00, 0x00, 0x00,
    // loop:
    //   r12d = delay counter (callee-saved across context switches)
    0x41, 0xBC, 0x00, 0x00, 0x02, 0x00,
    // delay:
    //   dec r12d
    0x41, 0xFF, 0xCC,
    //   jnz delay
    0x75, 0xFB,
    //   eax = 1010 (user-space syscall number for SYS_V2_DEBUG_PUTCHAR)
    0xB8, 0xF2, 0x03, 0x00, 0x00,
    //   edi = ebx ('B')
    0x89, 0xDF,
    //   int 0x80
    0xCD, 0x80,
    //   jmp loop
    0xEB, 0xEA
};

// Legacy test codes for backward compatibility
static const uint8_t ring3_int3_test_code[] = {
    0xCC,       // int3 (should cause #BP)
    0xEB, 0xFE  // jmp $ (infinite loop)
};

static const uint8_t ring3_ud2_test_code[] __attribute__((unused)) = {
    0x0F, 0x0B, // ud2 (undefined instruction exception)
    0xEB, 0xFE  // jmp $ (should not reach here if UD2 works)
};

static const uint8_t ring3_int80_test_code[] __attribute__((unused)) = {
    // rbx = 0x405000 (canary address)
    0x48, 0xBB, 0x00, 0x50, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,
    // [canary] = PRE
    0x48, 0xB8, 0x22, 0x22, 0x22, 0x22, 0x11, 0x11, 0x11, 0x11,
    0x48, 0x89, 0x03,
    // syscall: rax=1006 (SYS_V2_TIME_QUERY), rdi=0, rsi=0x405080
    0x48, 0xC7, 0xC0, 0xEE, 0x03, 0x00, 0x00,
    0x48, 0x31, 0xFF,
    0x48, 0xC7, 0xC6, 0x80, 0x50, 0x40, 0x00,
    0xCD, 0x80,
    // [canary] = POST (proves iretq returned to Ring3)
    0x48, 0xB8, 0x44, 0x44, 0x44, 0x44, 0x33, 0x33, 0x33, 0x33,
    0x48, 0x89, 0x03,
    // second syscall after POST write (kernel-side confirmation point)
    0x48, 0xC7, 0xC0, 0xEE, 0x03, 0x00, 0x00,
    0x48, 0x31, 0xFF,
    0x48, 0xC7, 0xC6, 0x80, 0x50, 0x40, 0x00,
    0xCD, 0x80,

    // loop
    0xEB, 0xFE         // jmp $ (infinite loop)
};

// Current test selection (will be modified by proc_launch_ring3_test)
static const uint8_t *current_ring3_test_code = ring3_int3_test_code;
static size_t current_ring3_test_size = sizeof(ring3_int3_test_code);

// Legacy name for compatibility
static const uint8_t *ring3_v2_syscall_test_code __attribute__((unused)) = ring3_int3_test_code;
#define ring3_v2_syscall_test_code_size sizeof(ring3_int3_test_code)

// Test data for the v2 syscall test (capability token structure)
static const uint8_t ring3_v2_test_data[] __attribute__((unused)) = {
    // Capability token at offset 0x08 (capability_token_t structure)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // id (will be assigned by kernel)
    0x01, 0x00, 0x00, 0x00,                          // permissions (CAP_PERM_READ)
    0x01, 0x00, 0x00, 0x00,                          // resource_type (CAP_RESOURCE_MEMORY)
};

/**
 * Creates a unified Ring3+Syscall test process for Phase 4.4 closure validation
 * This process will:
 * 1. Execute in Ring3 (user mode) with proper privilege level
 * 2. Write 'U' to debugcon to prove user code execution
 * 3. Execute one syscall (SYS_V2_TIME_QUERY) to validate syscall mechanism
 * 4. Write syscall markers before and after syscall execution
 * 5. Validate Ring3→Ring0→Ring3 transitions work correctly
 * 
 * @param name Process name for identification
 * @return proc_t* pointer to created process, NULL on failure
 */
static proc_t *proc_create_ring3_syscall_test(const char *name)
{
    outb(0xE9, (uint8_t)'C');
    fb_print("[ring3_test] Creating unified Ring3+Syscall test: ");
    fb_print(name);
    fb_print("\n");
    
    // Create user process with flat image format
    outb(0xE9, (uint8_t)'c');
    proc_t *test_proc = proc_create_user_process(name, 
                                                current_ring3_test_code,
                                                current_ring3_test_size,
                                                PROC_IMAGE_FLAT);
    
    if (!test_proc) {
        fb_print("[ring3_test] ERROR: Failed to create unified Ring3+Syscall test process\n");
        return NULL;
    }
    outb(0xE9, (uint8_t)'P');
    
    fb_print("[ring3_test] Unified Ring3+Syscall test process created successfully\n");
    fb_print("[ring3_test] - PID: ");
    fb_print_int(test_proc->pid);
    fb_print("\n");
    fb_print("[ring3_test] - Entry point: 0x");
    fb_print_hex(test_proc->context.rip);
    fb_print("\n");
    fb_print("[ring3_test] - Stack top: 0x");
    fb_print_hex(test_proc->context.rsp);
    fb_print("\n");
    fb_print("[ring3_test] - CS: 0x");
    fb_print_hex(test_proc->context.cs);
    fb_print(" (Ring3)\n");
    fb_print("[ring3_test] - SS: 0x");
    fb_print_hex(test_proc->context.ss);
    fb_print(" (Ring3)\n");

    fb_print("[DBG] USER ctx: cs=");
    fb_print_hex(test_proc->context.cs);
    fb_print(" ss=");
    fb_print_hex(test_proc->context.ss);
    fb_print(" rip=");
    fb_print_hex(test_proc->context.rip);
    fb_print(" rsp=");
    fb_print_hex(test_proc->context.rsp);
    fb_print(" rsp0=");
    fb_print_hex(test_proc->context.rsp0);
    fb_print("\n");
    
    // NOTE: sched_add() already called by proc_create_user_process()
    // No need to call it again here - would cause runqueue corruption
    
    outb(0xE9, (uint8_t)'D');
    return test_proc;
}

/*
 * Runtime syscall contract launcher (single Ring3 workload).
 * Uses Process A payload only:
 *   - time_query
 *   - capability_bind (granted + denied)
 *   - capability_revoke (granted + denied)
 *   - canonical debug marker [U][SYSCALL_OK]
 */
static void proc_launch_ring3_test(void)
{
    capability_token_t runtime_token;

    outb(0xE9, (uint8_t)'L');
    fb_print("[syscall_rt] =============================================\n");
    fb_print("[syscall_rt] Launching Ring3 syscall contract process\n");
    fb_print("[syscall_rt] =============================================\n");

    runtime_token = capability_create(
        CAPABILITY_RESOURCE_TIME,
        CAPABILITY_PERM_READ,
        RING3_CANARY_ADDR + 0x80,
        sizeof(uint64_t)
    );
    if (runtime_token.id == 0) {
        fb_print("[syscall_rt] ERROR: runtime capability create failed\n");
        fb_print("[syscall_rt] =============================================\n");
        return;
    }
    if (!ring3_prepare_process_a_code(runtime_token.id)) {
        fb_print("[syscall_rt] ERROR: runtime payload patch failed\n");
        fb_print("[syscall_rt] =============================================\n");
        return;
    }

    // Launch deterministic contract process.
    current_ring3_test_code = ring3_process_a_code;
    current_ring3_test_size = sizeof(ring3_process_a_code);
    proc_t *proc_a = proc_create_ring3_syscall_test("ring3-process-A");
    if (!proc_a) {
        fb_print("[syscall_rt] ERROR: process creation failed\n");
        fb_print("[syscall_rt] =============================================\n");
        return;
    }

    fb_print("[syscall_rt] Process created (PID=");
    fb_print_int(proc_a->pid);
    fb_print(")\n");
    fb_print("[syscall_rt] Expected marker: [U][SYSCALL_OK]\n");
    fb_print("[syscall_rt] =============================================\n");
}
