// tools/dump_abi_layout.c
// Constitutional ABI Layout Dumper
// Generates JSON representation of mailbox struct layout for CI verification
//
// Author: Kenan AY
// Project: AykenOS - Constitutional Runtime Lock (Gate-5)

#include <stdio.h>
#include <stddef.h>
#include "../kernel/include/sched_mailbox_abi.h"

int main(void) {
    printf("{\n");
    printf("  \"mailbox\": {\n");
    printf("    \"struct_name\": \"ayken_sched_mailbox_t\",\n");
    printf("    \"size\": %zu,\n", sizeof(ayken_sched_mailbox_t));
    printf("    \"alignment\": %zu,\n", _Alignof(ayken_sched_mailbox_t));
    printf("    \"attributes\": [\"packed\", \"aligned(64)\"],\n");
    printf("    \"fields\": {\n");
    printf("      \"magic\": {\n");
    printf("        \"type\": \"uint32_t\",\n");
    printf("        \"offset\": %zu,\n", offsetof(ayken_sched_mailbox_t, magic));
    printf("        \"size\": %zu\n", sizeof(((ayken_sched_mailbox_t*)0)->magic));
    printf("      },\n");
    printf("      \"version\": {\n");
    printf("        \"type\": \"uint16_t\",\n");
    printf("        \"offset\": %zu,\n", offsetof(ayken_sched_mailbox_t, version));
    printf("        \"size\": %zu\n", sizeof(((ayken_sched_mailbox_t*)0)->version));
    printf("      },\n");
    printf("      \"kind\": {\n");
    printf("        \"type\": \"uint16_t\",\n");
    printf("        \"offset\": %zu,\n", offsetof(ayken_sched_mailbox_t, kind));
    printf("        \"size\": %zu\n", sizeof(((ayken_sched_mailbox_t*)0)->kind));
    printf("      },\n");
    printf("      \"epoch\": {\n");
    printf("        \"type\": \"uint64_t\",\n");
    printf("        \"offset\": %zu,\n", offsetof(ayken_sched_mailbox_t, epoch));
    printf("        \"size\": %zu\n", sizeof(((ayken_sched_mailbox_t*)0)->epoch));
    printf("      },\n");
    printf("      \"proposer_pid\": {\n");
    printf("        \"type\": \"uint32_t\",\n");
    printf("        \"offset\": %zu,\n", offsetof(ayken_sched_mailbox_t, proposer_pid));
    printf("        \"size\": %zu\n", sizeof(((ayken_sched_mailbox_t*)0)->proposer_pid));
    printf("      },\n");
    printf("      \"candidate_pid\": {\n");
    printf("        \"type\": \"uint32_t\",\n");
    printf("        \"offset\": %zu,\n", offsetof(ayken_sched_mailbox_t, candidate_pid));
    printf("        \"size\": %zu\n", sizeof(((ayken_sched_mailbox_t*)0)->candidate_pid));
    printf("      },\n");
    printf("      \"flags\": {\n");
    printf("        \"type\": \"uint32_t\",\n");
    printf("        \"offset\": %zu,\n", offsetof(ayken_sched_mailbox_t, flags));
    printf("        \"size\": %zu\n", sizeof(((ayken_sched_mailbox_t*)0)->flags));
    printf("      },\n");
    printf("      \"status\": {\n");
    printf("        \"type\": \"uint32_t\",\n");
    printf("        \"offset\": %zu,\n", offsetof(ayken_sched_mailbox_t, status));
    printf("        \"size\": %zu\n", sizeof(((ayken_sched_mailbox_t*)0)->status));
    printf("      },\n");
    printf("      \"reject_reason\": {\n");
    printf("        \"type\": \"uint32_t\",\n");
    printf("        \"offset\": %zu,\n", offsetof(ayken_sched_mailbox_t, reject_reason));
    printf("        \"size\": %zu\n", sizeof(((ayken_sched_mailbox_t*)0)->reject_reason));
    printf("      },\n");
    printf("      \"reserved\": {\n");
    printf("        \"type\": \"uint32_t\",\n");
    printf("        \"offset\": %zu,\n", offsetof(ayken_sched_mailbox_t, reserved));
    printf("        \"size\": %zu\n", sizeof(((ayken_sched_mailbox_t*)0)->reserved));
    printf("      }\n");
    printf("    }\n");
    printf("  },\n");
    printf("  \"description\": \"Constitutional mailbox ABI baseline. Changes require version bump.\",\n");
    printf("  \"source\": \"kernel/include/sched_mailbox_abi.h\",\n");
    printf("  \"rules\": {\n");
    printf("    \"size_change\": \"Requires MAJOR version bump\",\n");
    printf("    \"field_offset_change\": \"Requires MAJOR version bump\",\n");
    printf("    \"field_add\": \"Requires MINOR version bump (append only)\",\n");
    printf("    \"field_remove\": \"Requires MAJOR version bump\"\n");
    printf("  }\n");
    printf("}\n");
    
    return 0;
}
