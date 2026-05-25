// kernel/sys/vcp_evidence.c
// VCP diagnostic evidence emission stubs

#include "../include/vcp_runtime.h"
#include "../include/execution_slot.h"

#define memset __builtin_memset

static vcp_diagnostic_evidence_entry_t
    g_vcp_diagnostic_evidence[VCP_DIAGNOSTIC_EVIDENCE_CAPACITY];
static uint32_t g_vcp_diagnostic_evidence_head;
static uint32_t g_vcp_diagnostic_evidence_count;
static uint32_t g_vcp_diagnostic_evidence_next_index;

static void vcp_evidence_debugcon_write_char(char ch)
{
    __asm__ volatile("outb %0, %1" : : "a"((uint8_t)ch), "Nd"((uint16_t)0xE9));
}

static void vcp_evidence_debugcon_write(const char *text)
{
    if (!text) {
        return;
    }

    while (*text != '\0') {
        vcp_evidence_debugcon_write_char(*text);
        text++;
    }
}

static void vcp_evidence_debugcon_write_u64_hex(uint64_t value)
{
    static const char hex_digits[] = "0123456789ABCDEF";
    int shift;

    vcp_evidence_debugcon_write("0x");
    for (shift = 60; shift >= 0; shift -= 4) {
        vcp_evidence_debugcon_write_char(hex_digits[(value >> (uint32_t)shift) & 0x0Fu]);
    }
}

static uint32_t vcp_evidence_hash_string(const char *text)
{
    uint32_t hash = 2166136261u;

    if (!text) {
        text = "unspecified";
    }

    while (*text != '\0') {
        hash ^= (uint8_t)*text;
        hash *= 16777619u;
        text++;
    }

    if (hash == 0) {
        hash = 1u;
    }

    return hash;
}

static void vcp_evidence_fill_slot_fields(vcp_diagnostic_evidence_entry_t *entry,
                                          const exec_slot_t *slot)
{
    const vcp_validation_state_t *state;

    if (!entry) {
        return;
    }

    if (!slot) {
        return;
    }

    entry->slot_id = slot->execution_id;
    entry->generation = slot->generation;
    entry->owner_pid = slot->owner_pid;
    entry->target_context_id = slot->target_context_id;
    entry->slot_state = (uint64_t)slot->state;
    entry->error_code = (uint64_t)slot->error_code;

    state = slot->validation_state;
    if (!state) {
        return;
    }

    entry->context_hash = state->context_hash;
    entry->nonce = state->nonce;
    entry->capability_id = state->capability_id;
    entry->evidence_id = state->evidence_id;
}

static void vcp_evidence_append(vcp_diagnostic_evidence_entry_t *entry)
{
    if (!entry) {
        return;
    }

    entry->index = g_vcp_diagnostic_evidence_next_index++;
    g_vcp_diagnostic_evidence[g_vcp_diagnostic_evidence_head] = *entry;
    g_vcp_diagnostic_evidence_head =
        (g_vcp_diagnostic_evidence_head + 1u) % VCP_DIAGNOSTIC_EVIDENCE_CAPACITY;

    if (g_vcp_diagnostic_evidence_count < VCP_DIAGNOSTIC_EVIDENCE_CAPACITY) {
        g_vcp_diagnostic_evidence_count++;
    }
}

static void vcp_evidence_emit_common(uint32_t event_type,
                                     struct exec_slot *slot,
                                     int result,
                                     const char *reason,
                                     const char *label)
{
    vcp_diagnostic_evidence_entry_t entry;

    memset(&entry, 0, sizeof(entry));
    entry.event_type = event_type;
    entry.result = result;
    entry.event_result = (uint64_t)(uint32_t)result;
    entry.reason_hash = vcp_evidence_hash_string(reason);
    entry.label_hash = vcp_evidence_hash_string(label);
    vcp_evidence_fill_slot_fields(&entry, slot);
    vcp_evidence_append(&entry);
}

void vcp_emit_validation_check(struct exec_slot *slot, int result)
{
    vcp_evidence_emit_common(VCP_DIAG_EVENT_VALIDATION_CHECK,
                             slot,
                             result,
                             "validation_check",
                             "vcp_runtime_validate");

    vcp_evidence_debugcon_write("[VCP_EVIDENCE][VALIDATION_CHECK] result=");
    vcp_evidence_debugcon_write_u64_hex((uint64_t)(uint32_t)result);
    vcp_evidence_debugcon_write(" slot=");
    if (slot) {
        vcp_evidence_debugcon_write_u64_hex(slot->execution_id);
    } else {
        vcp_evidence_debugcon_write("null");
    }
    vcp_evidence_debugcon_write("\n");
}

void vcp_emit_execution_block(struct exec_slot *slot, const char *reason)
{
    vcp_evidence_emit_common(VCP_DIAG_EVENT_EXECUTION_BLOCK,
                             slot,
                             VCP_FAIL_CLOSED,
                             reason,
                             "vcp_fail_closed");

    vcp_evidence_debugcon_write("[VCP_FAIL_CLOSED][BLOCK] reason=");
    vcp_evidence_debugcon_write(reason ? reason : "unspecified");
    vcp_evidence_debugcon_write(" slot=");
    if (slot) {
        vcp_evidence_debugcon_write_u64_hex(slot->execution_id);
        vcp_evidence_debugcon_write(" state=");
        vcp_evidence_debugcon_write_u64_hex((uint64_t)slot->state);
        vcp_evidence_debugcon_write(" error=");
        vcp_evidence_debugcon_write_u64_hex((uint64_t)slot->error_code);
    } else {
        vcp_evidence_debugcon_write("null");
    }
    vcp_evidence_debugcon_write("\n");
}

void vcp_emit_contract_execution(struct exec_slot *slot, const char *contract_id)
{
    vcp_evidence_emit_common(VCP_DIAG_EVENT_CONTRACT_EXECUTION,
                             slot,
                             VCP_VALID,
                             "contract_execution",
                             contract_id);

    vcp_evidence_debugcon_write("[VCP_EVIDENCE][CONTRACT_EXECUTION] slot=");
    if (slot) {
        vcp_evidence_debugcon_write_u64_hex(slot->execution_id);
    } else {
        vcp_evidence_debugcon_write("null");
    }
    vcp_evidence_debugcon_write(" label_hash=");
    vcp_evidence_debugcon_write_u64_hex((uint64_t)vcp_evidence_hash_string(contract_id));
    vcp_evidence_debugcon_write("\n");
}

void vcp_emit_boundary_crossing(struct exec_slot *slot, const char *boundary_id)
{
    vcp_evidence_emit_common(VCP_DIAG_EVENT_BOUNDARY_CROSSING,
                             slot,
                             VCP_VALID,
                             "boundary_crossing",
                             boundary_id);

    vcp_evidence_debugcon_write("[VCP_EVIDENCE][BOUNDARY_CROSSING] slot=");
    if (slot) {
        vcp_evidence_debugcon_write_u64_hex(slot->execution_id);
    } else {
        vcp_evidence_debugcon_write("null");
    }
    vcp_evidence_debugcon_write(" label_hash=");
    vcp_evidence_debugcon_write_u64_hex((uint64_t)vcp_evidence_hash_string(boundary_id));
    vcp_evidence_debugcon_write("\n");
}

#if AYKEN_VCP_TEST_HOOKS
void vcp_test_reset_diagnostic_evidence(void)
{
    memset(g_vcp_diagnostic_evidence, 0, sizeof(g_vcp_diagnostic_evidence));
    g_vcp_diagnostic_evidence_head = 0;
    g_vcp_diagnostic_evidence_count = 0;
    g_vcp_diagnostic_evidence_next_index = 0;
}

uint32_t vcp_test_diagnostic_evidence_count(void)
{
    return g_vcp_diagnostic_evidence_count;
}

int vcp_test_get_diagnostic_evidence(uint32_t logical_index,
                                     vcp_diagnostic_evidence_entry_t *out)
{
    uint32_t oldest;
    uint32_t physical_index;

    if (!out || logical_index >= g_vcp_diagnostic_evidence_count) {
        return -1;
    }

    oldest = (g_vcp_diagnostic_evidence_head +
              VCP_DIAGNOSTIC_EVIDENCE_CAPACITY -
              g_vcp_diagnostic_evidence_count) %
             VCP_DIAGNOSTIC_EVIDENCE_CAPACITY;
    physical_index = (oldest + logical_index) % VCP_DIAGNOSTIC_EVIDENCE_CAPACITY;

    *out = g_vcp_diagnostic_evidence[physical_index];
    return 0;
}
#endif
