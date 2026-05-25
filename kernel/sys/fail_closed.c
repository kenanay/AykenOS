// kernel/sys/fail_closed.c
// VCP fail-closed enforcement mechanism

#include "../include/vcp_runtime.h"
#include "../include/execution_slot.h"

int vcp_fail_closed_is_active(const struct exec_slot *slot)
{
    return slot &&
           slot->in_use &&
           slot->state == EXEC_SLOT_ABORTED &&
           slot->error_code == VCP_FAIL_CLOSED_SLOT_ERROR_CODE;
}

int vcp_fail_closed(struct exec_slot *slot, const char *reason)
{
    if (!slot || !slot->in_use) {
        vcp_emit_execution_block(slot, reason);
        return VCP_FAIL_CLOSED;
    }

    if (vcp_fail_closed_is_active(slot)) {
        vcp_emit_execution_block(slot, reason);
        return VCP_FAIL_CLOSED;
    }

    slot->state = EXEC_SLOT_ABORTED;
    slot->error_code = VCP_FAIL_CLOSED_SLOT_ERROR_CODE;
    vcp_emit_execution_block(slot, reason);

    return VCP_FAIL_CLOSED;
}
