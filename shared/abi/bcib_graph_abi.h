#ifndef BCIB_GRAPH_ABI_H
#define BCIB_GRAPH_ABI_H

#include <stdint.h>

/*
 * BCIB Graph ABI - Minimal Bootstrap Version
 * 
 * This is the minimal graph structure for bootstrap testing.
 * Real BCIB graphs will be more complex, but this establishes
 * the basic contract between userspace and kernel.
 */

#define BCIB_GRAPH_MAGIC 0x42434942  /* "BCIB" in little-endian */
#define BCIB_GRAPH_VERSION 1

/*
 * Minimal BCIB Graph Structure
 * 
 * This structure must be passed to SYS_V2_SUBMIT_EXECUTION.
 * Kernel validates magic and version before accepting execution.
 */
typedef struct {
    uint32_t magic;         /* Must be BCIB_GRAPH_MAGIC */
    uint32_t version;       /* Must be BCIB_GRAPH_VERSION */
    uint64_t entry_node;    /* Entry point node ID (0 for bootstrap) */
    uint64_t node_count;    /* Number of nodes in graph (1 for bootstrap) */
} __attribute__((packed)) bcib_graph_t;

#endif /* BCIB_GRAPH_ABI_H */
