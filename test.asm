%if AYKEN_RING3_ENTRY_MEM_PROFILE == 1
extern entry_diag_index
%endif
global _start
_start:
%if AYKEN_RING3_ENTRY_MEM_PROFILE == 1
    mov eax, [entry_diag_index]
%endif
    ret
