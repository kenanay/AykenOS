// SPDX-License-Identifier: ASAL-1.0
// Copyright (C) 2026 Kenan AY
//
// Ring3 Process Preparation (Scheduler Dispatch Path)
// Authority: Phase 10-A2
// Constitutional: Ring0 mechanism only (no policy)

#include <stdint.h>
#include "arch/x86_64/port_io.h"
#include "drivers/console/fb_console.h"
#include "embedded_elf.h"
#include "ring3_jump.h"
#include "gdt_idt.h"
#include "include/proc.h"

static void debugcon_write(const char *s)
{
    if (!s) {
        return;
    }
    while (*s) {
        outb(0xE9, (uint8_t)*s);
        s++;
    }
}

static void halt_forever(void)
{
    for (;;) {
        __asm__ volatile("cli; hlt");
    }
}

static void ring3_prep_panic(const char *marker, const char *msg)
{
    if (marker) {
        debugcon_write(marker);
        debugcon_write("\n");
    }
    if (msg) {
        fb_print(msg);
        fb_print("\n");
        debugcon_write(msg);
        debugcon_write("\n");
    }
    halt_forever();
}

void jump_to_ring3(void)
{
    const uint64_t min_elf64_ehdr_size = 64;

    debugcon_write("[K][PHASE10] KERNEL_BEFORE_RING3\n");
    fb_print("[PHASE10] Preparing Ring3 process for scheduler dispatch...\n");

    if (embedded_elf_size < min_elf64_ehdr_size) {
        ring3_prep_panic("[[AYKEN_RING3_PREP_FAIL]] no_elf",
                         "[PANIC] Phase10: Embedded ELF is missing or truncated.");
    }
    if (embedded_elf[0] != 0x7F || embedded_elf[1] != 'E' ||
        embedded_elf[2] != 'L' || embedded_elf[3] != 'F') {
        ring3_prep_panic("[[AYKEN_RING3_PREP_FAIL]] bad_magic",
                         "[PANIC] Phase10: Embedded ELF magic is invalid.");
    }

    proc_t *ring3_proc = proc_create_user_process(
        "phase10-minimal",
        embedded_elf,
        (uint64_t)embedded_elf_size,
        PROC_IMAGE_ELF
    );
    if (!ring3_proc) {
        ring3_prep_panic("[[AYKEN_RING3_PREP_FAIL]] create",
                         "[PANIC] Phase10: Ring3 process creation failed.");
    }

    if (proc_find_by_pid(ring3_proc->pid) != ring3_proc) {
        ring3_prep_panic("[[AYKEN_RING3_PREP_FAIL]] not_registered",
                         "[PANIC] Phase10: Ring3 process not present in pid table.");
    }
    if (ring3_proc->state != PROC_READY) {
        ring3_prep_panic("[[AYKEN_RING3_PREP_FAIL]] not_ready",
                         "[PANIC] Phase10: Ring3 process is not runnable.");
    }
    if (ring3_proc->context.cs != GDT_USER_CODE || ring3_proc->context.ss != GDT_USER_DATA) {
        ring3_prep_panic("[[AYKEN_RING3_PREP_FAIL]] bad_segments",
                         "[PANIC] Phase10: Ring3 selectors are invalid.");
    }
    if (!ring3_proc->context.rsp0) {
        ring3_prep_panic("[[AYKEN_RING3_PREP_FAIL]] no_rsp0",
                         "[PANIC] Phase10: Ring3 process has no kernel rsp0.");
    }

    debugcon_write("[[AYKEN_RING3_PREP_OK]]\n");
    debugcon_write("P10_SCHED_ARMED\n");
    fb_print("[PHASE10] Ring3 process prepared and queued.\n");
}
