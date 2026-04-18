/*
 * AykenOS Ring3 Spin Probe - Minimal Execution Test
 * 
 * Purpose: Verify Ring3 fetch/execute pipeline works
 * 
 * Expected behavior:
 *   - CPU enters Ring3
 *   - Executes int3 (Breakpoint)
 *   - Kernel handles #BP
 * 
 * If no marker appears: RIP/CR3/mapping/NX issue
 */

.section .text
.global _start

_start:
    int3                    # Trigger #BP exception
.Lloop:
    pause                   # CPU hint for spin loop
    jmp     .Lloop          # Infinite loop

.section .note.GNU-stack,"",@progbits
