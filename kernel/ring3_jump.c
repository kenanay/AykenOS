// kernel/ring3_jump.c
// ============================================================================
// AykenOS Phase 10-A: Ring3 process preparation
// ============================================================================

#include <stdint.h>
#include "arch/x86_64/port_io.h"
#include "drivers/console/fb_console.h"
#include "include/embedded_elf.h"
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
    }
    halt_forever();
}

// Prepare Ring3 process from embedded ELF and enqueue it for scheduler startup.
void jump_to_ring3(void)
{
    debugcon_write("[K][PHASE10] KERNEL_BEFORE_RING3\n");

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

    proc_t *registered = proc_find_by_pid(ring3_proc->pid);
    if (registered != ring3_proc) {
        ring3_prep_panic("[[AYKEN_RING3_PREP_FAIL]] not_registered",
                         "[PANIC] Phase10: Ring3 process not registered.");
    }

    if (ring3_proc->state != PROC_READY) {
        ring3_prep_panic("[[AYKEN_RING3_PREP_FAIL]] not_runnable",
                         "[PANIC] Phase10: Ring3 process not runnable.");
    }

    debugcon_write("[K][PHASE10] RING3_PREP_OK\n");
    debugcon_write("[[AYKEN_RING3_PREP_OK]]\n");
    fb_print("[PHASE10] Ring3 process prepared and queued.\n");
}
