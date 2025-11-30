#include "ai_boot_analyzer.h"
#include "../mm/phys_mem.h"
#include "../sched/scheduler.h"
#include "../drivers/console/fb_console.h"

void ai_boot_analyzer()
{
    fb_print("[AI] Running boot analyzer...\n");

    // Sistem profilini çıkaralım:
    uint64_t ram_total = phys_mem_total();
    uint32_t cpu_cores = cpu_get_core_count();

    // AI yorumlashası (ileride LLM ile yapılacak)
    if (ram_total < (2ULL * 1024 * 1024 * 1024)) {
        fb_print("[AI] Low RAM detected → enabling compact mode.\n");
        scheduler_set_compact_mode();
    }

    fb_print("[AI] Boot analysis completed.\n");
}
