#include <stdint.h>
#include <stddef.h>

#if defined(AYKEN_RING3_ENTRY_MEM_PROFILE) && (AYKEN_RING3_ENTRY_MEM_PROFILE == 1)

extern void timer_debugcon_write(const char *msg);
extern void timer_debugcon_hex(uint32_t val);
extern void timer_debugcon_hex64(uint64_t val);

struct entry_diag_sample {
    uint32_t phase;
    uint32_t aux;
    uint64_t tsc;
};

/* Defined in ring3_enter.S */
extern struct entry_diag_sample entry_diag_buffer[1024];
extern uint32_t entry_diag_index;
extern uint32_t entry_diag_enabled;

static uint32_t entry_diag_dumped = 0;

static inline uint64_t entry_diag_read_tsc(void) {
    uint32_t lo, hi;
    asm volatile("lfence\nrdtsc" : "=a"(lo), "=d"(hi) :: "memory");
    return ((uint64_t)hi << 32) | lo;
}

void entry_diag_record_c(uint32_t phase, uint32_t aux) {
    if (!entry_diag_enabled) {
        return;
    }
    
    uint32_t idx = entry_diag_index;
    if (idx >= 1024) {
        return;
    }
    
    entry_diag_buffer[idx].phase = phase;
    entry_diag_buffer[idx].aux = aux;
    entry_diag_buffer[idx].tsc = entry_diag_read_tsc();
    
    asm volatile("" ::: "memory");
    
    entry_diag_index = idx + 1;
}

void entry_diag_dump(void) {
    if (!entry_diag_enabled || entry_diag_dumped) {
        return;
    }
    
    uint32_t count = entry_diag_index;
    if (count == 0) {
        return;
    }
    if (count > 1024) {
        count = 1024;
    }
    
    entry_diag_dumped = 1;
    
    /* Prevent compiler from reordering reads around dump start */
    asm volatile("" ::: "memory");
    
    timer_debugcon_write("\n=== ENTRY_DIAG_DUMP START ===\n");
    
    for (uint32_t i = 0; i < count; i++) {
        timer_debugcon_write("ENTRY_DIAG_SAMPLE[");
        timer_debugcon_hex(i);
        
        timer_debugcon_write("] phase=");
        timer_debugcon_hex(entry_diag_buffer[i].phase);
        
        timer_debugcon_write(" aux=");
        timer_debugcon_hex(entry_diag_buffer[i].aux);
        
        timer_debugcon_write(" tsc=");
        timer_debugcon_hex64(entry_diag_buffer[i].tsc);
        
        timer_debugcon_write("\n");
    }
    
    timer_debugcon_write("=== ENTRY_DIAG_DUMP END ===\n");
    
    /* Stop any later samples */
    entry_diag_enabled = 0;
    
    asm volatile("" ::: "memory");
}

#endif
