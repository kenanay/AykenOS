# BCIB Submission Protocol

**Project:** AykenOS  
**Version:** 1.0  
**Status:** ACTIVE  
**Effective Date:** 2026-02-21  
**Owner:** AykenOS Core Architecture Team

---

## 1. Genel Bakış

Binary Compressed Instruction Buffer (BCIB) submission protocol, AykenOS'un execution-centric mimarisinin temelini oluşturur. Bu protokol, veri-odaklı instruction graph'larının Ring0'a güvenli şekilde submit edilmesini sağlar.

### Temel Prensipler

- **Data-Centric:** Instruction'lar veri olarak temsil edilir
- **Graph-Based:** Instruction'lar directed acyclic graph (DAG) yapısındadır
- **Capability-Protected:** Submission, capability token gerektirir
- **Validation-First:** Graph, execution öncesi validate edilir

---

## 2. BCIB Format (v0.2)

### 2.1 BCIB Header

```c
typedef struct bcib_header {
    uint32_t magic;           // Magic number: 0x42434942 ("BCIB")
    uint16_t version_major;   // Major version (0)
    uint16_t version_minor;   // Minor version (2)
    uint32_t node_count;      // Number of nodes in graph
    uint32_t edge_count;      // Number of edges in graph
    uint32_t entry_node;      // Entry point node ID
    uint32_t flags;           // Execution flags
    uint64_t timestamp;       // Creation timestamp
    uint64_t checksum;        // Header + graph checksum
} bcib_header_t;
```

### 2.2 BCIB Node

```c
typedef struct bcib_node {
    uint32_t node_id;         // Unique node identifier
    uint32_t opcode;          // Operation code
    uint32_t input_count;     // Number of input edges
    uint32_t output_count;    // Number of output edges
    uint64_t data_offset;     // Offset to node data
    uint64_t data_size;       // Size of node data
    uint32_t flags;           // Node flags
    uint32_t reserved;        // Reserved for future use
} bcib_node_t;
```

### 2.3 BCIB Edge

```c
typedef struct bcib_edge {
    uint32_t edge_id;         // Unique edge identifier
    uint32_t source_node;     // Source node ID
    uint32_t dest_node;       // Destination node ID
    uint32_t source_port;     // Source output port
    uint32_t dest_port;       // Destination input port
    uint32_t data_type;       // Data type flowing through edge
    uint32_t flags;           // Edge flags
    uint32_t reserved;        // Reserved for future use
} bcib_edge_t;
```

---

## 3. Submission Flow

### 3.1 High-Level Flow

```
1. Ring3: BCIB graph oluştur
2. Ring3: Graph'ı serialize et
3. Ring3: Execution context oluştur
4. Ring3: Capability token bind et
5. Ring3: sys_v2_submit_execution() çağır
6. Ring0: Graph'ı validate et
7. Ring0: Execution context'e graph'ı ata
8. Ring0: Execution ID return et
9. Ring3: sys_v2_wait_result() ile sonuç bekle
10. Ring0: Execution tamamlandığında result return et
```

### 3.2 Detailed Submission

```c
// Ring3 code
uint64_t submit_bcib_graph(bcib_graph_t *graph) {
    // 1. Serialize graph
    void *serialized = bcib_serialize(graph);
    uint64_t size = bcib_get_size(graph);
    
    // 2. Create execution context
    uint64_t ctx_id = create_execution_context();
    
    // 3. Bind memory capability
    capability_token_t mem_cap = create_memory_capability(
        serialized, size, CAP_PERM_READ
    );
    sys_v2_capability_bind(ctx_id, &mem_cap);
    
    // 4. Submit execution
    uint64_t exec_id = sys_v2_submit_execution(
        serialized,  // BCIB graph buffer
        size,        // Buffer size
        ctx_id       // Execution context
    );
    
    if (exec_id == (uint64_t)-1) {
        // Submission failed
        handle_error();
    }
    
    return exec_id;
}
```

---

## 4. Graph Validation

### 4.1 Validation Stages

Ring0, graph'ı şu aşamalarda validate eder:

1. **Header Validation:** Magic number, version, checksum
2. **Structure Validation:** Node/edge counts, sizes
3. **DAG Validation:** Cycle detection, reachability
4. **Capability Validation:** Memory access permissions
5. **Resource Validation:** Memory limits, execution limits

### 4.2 Header Validation

```c
int validate_bcib_header(bcib_header_t *header) {
    // Magic number check
    if (header->magic != 0x42434942) {
        return BCIB_ERROR_INVALID_MAGIC;
    }
    
    // Version check
    if (header->version_major != 0 || header->version_minor != 2) {
        return BCIB_ERROR_UNSUPPORTED_VERSION;
    }
    
    // Sanity checks
    if (header->node_count == 0 || header->node_count > MAX_NODES) {
        return BCIB_ERROR_INVALID_NODE_COUNT;
    }
    
    if (header->edge_count > MAX_EDGES) {
        return BCIB_ERROR_INVALID_EDGE_COUNT;
    }
    
    // Checksum validation
    uint64_t computed_checksum = compute_checksum(header);
    if (computed_checksum != header->checksum) {
        return BCIB_ERROR_CHECKSUM_MISMATCH;
    }
    
    return BCIB_SUCCESS;
}
```

### 4.3 DAG Validation

```c
int validate_bcib_dag(bcib_graph_t *graph) {
    // Cycle detection (DFS-based)
    if (has_cycle(graph)) {
        return BCIB_ERROR_CYCLE_DETECTED;
    }
    
    // Reachability check
    if (!is_reachable(graph, graph->header.entry_node)) {
        return BCIB_ERROR_UNREACHABLE_NODES;
    }
    
    // Edge validation
    for (uint32_t i = 0; i < graph->header.edge_count; i++) {
        bcib_edge_t *edge = &graph->edges[i];
        
        // Source/dest node existence
        if (!node_exists(graph, edge->source_node) ||
            !node_exists(graph, edge->dest_node)) {
            return BCIB_ERROR_INVALID_EDGE;
        }
        
        // Port validation
        bcib_node_t *src = get_node(graph, edge->source_node);
        bcib_node_t *dst = get_node(graph, edge->dest_node);
        
        if (edge->source_port >= src->output_count ||
            edge->dest_port >= dst->input_count) {
            return BCIB_ERROR_INVALID_PORT;
        }
    }
    
    return BCIB_SUCCESS;
}
```

### 4.4 Capability Validation

```c
int validate_bcib_capabilities(uint64_t ctx_id, bcib_graph_t *graph) {
    // Check if context has execution capability
    if (!has_capability(ctx_id, CAP_RESOURCE_EXECUTION, CAP_PERM_EXECUTE)) {
        return BCIB_ERROR_NO_EXEC_CAPABILITY;
    }
    
    // Check memory access for graph buffer
    void *graph_buffer = (void *)graph;
    uint64_t graph_size = compute_graph_size(graph);
    
    if (!check_memory_access(ctx_id, (uint64_t)graph_buffer, 
                            graph_size, CAP_PERM_READ)) {
        return BCIB_ERROR_NO_MEMORY_CAPABILITY;
    }
    
    // Check memory access for node data
    for (uint32_t i = 0; i < graph->header.node_count; i++) {
        bcib_node_t *node = &graph->nodes[i];
        
        if (node->data_size > 0) {
            uint64_t data_addr = (uint64_t)graph_buffer + node->data_offset;
            
            if (!check_memory_access(ctx_id, data_addr, 
                                    node->data_size, CAP_PERM_READ)) {
                return BCIB_ERROR_NO_DATA_CAPABILITY;
            }
        }
    }
    
    return BCIB_SUCCESS;
}
```

---

## 5. Execution Management

### 5.1 Execution Context

```c
typedef struct bcib_execution {
    uint64_t execution_id;        // Unique execution ID
    uint64_t context_id;          // Associated execution context
    bcib_graph_t *graph;          // BCIB graph
    uint32_t status;              // Execution status
    uint64_t start_time;          // Execution start time
    uint64_t end_time;            // Execution end time
    void *result_buffer;          // Result buffer
    uint64_t result_size;         // Result size
    uint32_t error_code;          // Error code (if failed)
} bcib_execution_t;
```

### 5.2 Execution States

| State | Value | Açıklama |
|-------|-------|----------|
| `BCIB_EXEC_CREATED` | 0x01 | Execution created, not started |
| `BCIB_EXEC_VALIDATING` | 0x02 | Graph validation in progress |
| `BCIB_EXEC_READY` | 0x04 | Validation passed, ready to execute |
| `BCIB_EXEC_RUNNING` | 0x08 | Execution in progress |
| `BCIB_EXEC_COMPLETED` | 0x10 | Execution completed successfully |
| `BCIB_EXEC_FAILED` | 0x20 | Execution failed |
| `BCIB_EXEC_TIMEOUT` | 0x40 | Execution timed out |

### 5.3 Execution Lifecycle

```c
// Submit execution
uint64_t sys_v2_submit_execution(void *bcib_graph, uint64_t graph_size, 
                                 uint64_t context_id) {
    // 1. Allocate execution structure
    bcib_execution_t *exec = allocate_execution();
    if (!exec) return (uint64_t)-1;
    
    // 2. Initialize execution
    exec->execution_id = next_execution_id++;
    exec->context_id = context_id;
    exec->graph = (bcib_graph_t *)bcib_graph;
    exec->status = BCIB_EXEC_CREATED;
    exec->start_time = get_system_time();
    
    // 3. Validate graph
    exec->status = BCIB_EXEC_VALIDATING;
    int result = validate_bcib_graph(exec->graph);
    if (result != BCIB_SUCCESS) {
        exec->status = BCIB_EXEC_FAILED;
        exec->error_code = result;
        return (uint64_t)-1;
    }
    
    // 4. Validate capabilities
    result = validate_bcib_capabilities(context_id, exec->graph);
    if (result != BCIB_SUCCESS) {
        exec->status = BCIB_EXEC_FAILED;
        exec->error_code = result;
        return (uint64_t)-1;
    }
    
    // 5. Mark as ready
    exec->status = BCIB_EXEC_READY;
    
    // 6. Schedule execution (Ring3 policy decides when)
    schedule_bcib_execution(exec);
    
    return exec->execution_id;
}
```

---

## 6. Result Retrieval

### 6.1 Wait for Result

```c
// Ring3 code
uint64_t wait_for_result(uint64_t exec_id, uint64_t timeout_ms) {
    uint64_t result = sys_v2_wait_result(exec_id, timeout_ms);
    
    if (result == (uint64_t)-1) {
        // Wait failed or timed out
        handle_error();
    }
    
    return result;
}

// Ring0 implementation
uint64_t sys_v2_wait_result(uint64_t execution_id, uint64_t timeout_ms) {
    bcib_execution_t *exec = find_execution(execution_id);
    if (!exec) return (uint64_t)-1;
    
    uint64_t start_time = get_system_time();
    
    // Wait for completion
    while (exec->status != BCIB_EXEC_COMPLETED &&
           exec->status != BCIB_EXEC_FAILED &&
           exec->status != BCIB_EXEC_TIMEOUT) {
        
        // Check timeout
        if (timeout_ms > 0) {
            uint64_t elapsed = get_system_time() - start_time;
            if (elapsed >= timeout_ms) {
                exec->status = BCIB_EXEC_TIMEOUT;
                return (uint64_t)-1;
            }
        }
        
        // Yield CPU
        sched_yield();
    }
    
    // Return result
    if (exec->status == BCIB_EXEC_COMPLETED) {
        return (uint64_t)exec->result_buffer;
    }
    
    return (uint64_t)-1;
}
```

### 6.2 Result Format

```c
typedef struct bcib_result {
    uint64_t execution_id;        // Execution ID
    uint32_t status;              // Final status
    uint32_t error_code;          // Error code (if failed)
    uint64_t execution_time_us;   // Execution time in microseconds
    void *output_buffer;          // Output data buffer
    uint64_t output_size;         // Output data size
    uint32_t output_node_count;   // Number of output nodes
} bcib_result_t;
```

---

## 7. Error Handling

### 7.1 Error Codes

| Error Code | Value | Açıklama |
|------------|-------|----------|
| `BCIB_SUCCESS` | 0 | Operation successful |
| `BCIB_ERROR_INVALID_MAGIC` | -1 | Invalid magic number |
| `BCIB_ERROR_UNSUPPORTED_VERSION` | -2 | Unsupported BCIB version |
| `BCIB_ERROR_INVALID_NODE_COUNT` | -3 | Invalid node count |
| `BCIB_ERROR_INVALID_EDGE_COUNT` | -4 | Invalid edge count |
| `BCIB_ERROR_CHECKSUM_MISMATCH` | -5 | Checksum validation failed |
| `BCIB_ERROR_CYCLE_DETECTED` | -6 | Cycle detected in DAG |
| `BCIB_ERROR_UNREACHABLE_NODES` | -7 | Unreachable nodes in graph |
| `BCIB_ERROR_INVALID_EDGE` | -8 | Invalid edge definition |
| `BCIB_ERROR_INVALID_PORT` | -9 | Invalid port number |
| `BCIB_ERROR_NO_EXEC_CAPABILITY` | -10 | Missing execution capability |
| `BCIB_ERROR_NO_MEMORY_CAPABILITY` | -11 | Missing memory capability |
| `BCIB_ERROR_NO_DATA_CAPABILITY` | -12 | Missing data access capability |
| `BCIB_ERROR_EXECUTION_FAILED` | -13 | Execution failed |
| `BCIB_ERROR_TIMEOUT` | -14 | Execution timed out |

### 7.2 Error Recovery

```c
// Submission error recovery
uint64_t submit_with_retry(bcib_graph_t *graph, int max_retries) {
    for (int i = 0; i < max_retries; i++) {
        uint64_t exec_id = submit_bcib_graph(graph);
        
        if (exec_id != (uint64_t)-1) {
            return exec_id;  // Success
        }
        
        // Wait before retry
        sleep_ms(100 * (i + 1));
    }
    
    return (uint64_t)-1;  // All retries failed
}

// Execution error recovery
uint64_t execute_with_fallback(bcib_graph_t *graph, 
                               bcib_graph_t *fallback_graph) {
    uint64_t exec_id = submit_bcib_graph(graph);
    
    if (exec_id == (uint64_t)-1) {
        // Primary graph failed, try fallback
        exec_id = submit_bcib_graph(fallback_graph);
    }
    
    return exec_id;
}
```

---

## 8. Performance Characteristics

### 8.1 Latency

- **Submission Latency:** < 10 μs (validation included)
- **Validation Latency:** < 5 μs (typical graph)
- **Execution Latency:** Variable (graph-dependent)
- **Result Retrieval Latency:** < 1 μs

### 8.2 Throughput

- **Submissions per Second:** > 100,000
- **Concurrent Executions:** > 1,000

### 8.3 Limits

```c
#define MAX_NODES 10000           // Max nodes per graph
#define MAX_EDGES 50000           // Max edges per graph
#define MAX_NODE_DATA_SIZE (1024*1024)  // 1 MB per node
#define MAX_GRAPH_SIZE (10*1024*1024)   // 10 MB per graph
#define MAX_CONCURRENT_EXECUTIONS 1024  // System-wide limit
```

---

## 9. Best Practices

### 9.1 Graph Design

```c
// BAD: Monolithic graph
bcib_graph_t *create_monolithic_graph() {
    // Single large graph with 10000 nodes
    // Hard to debug, slow to validate
}

// GOOD: Modular graphs
bcib_graph_t *create_modular_graph() {
    // Multiple smaller graphs (100-1000 nodes each)
    // Easy to debug, fast to validate
    // Can be composed at runtime
}
```

### 9.2 Error Handling

```c
// BAD: Ignore errors
uint64_t exec_id = sys_v2_submit_execution(graph, size, ctx_id);
sys_v2_wait_result(exec_id, 0);  // No error check

// GOOD: Handle errors
uint64_t exec_id = sys_v2_submit_execution(graph, size, ctx_id);
if (exec_id == (uint64_t)-1) {
    log_error("Submission failed");
    return handle_submission_error();
}

uint64_t result = sys_v2_wait_result(exec_id, timeout);
if (result == (uint64_t)-1) {
    log_error("Execution failed or timed out");
    return handle_execution_error();
}
```

### 9.3 Resource Management

```c
// Cleanup after execution
void cleanup_execution(uint64_t exec_id) {
    // Wait for result
    uint64_t result = sys_v2_wait_result(exec_id, timeout);
    
    // Free result buffer
    if (result != (uint64_t)-1) {
        free_result_buffer((void *)result);
    }
    
    // Revoke capabilities
    revoke_execution_capabilities(exec_id);
}
```

---

## 10. Testing

### 10.1 Unit Tests

```c
// Test: Valid graph submission
void test_valid_submission(void) {
    bcib_graph_t *graph = create_test_graph();
    uint64_t exec_id = submit_bcib_graph(graph);
    assert(exec_id != (uint64_t)-1);
}

// Test: Invalid graph rejection
void test_invalid_graph(void) {
    bcib_graph_t *graph = create_invalid_graph();  // Has cycle
    uint64_t exec_id = submit_bcib_graph(graph);
    assert(exec_id == (uint64_t)-1);
}

// Test: Capability enforcement
void test_capability_enforcement(void) {
    bcib_graph_t *graph = create_test_graph();
    uint64_t ctx_id = create_context_without_exec_capability();
    uint64_t exec_id = sys_v2_submit_execution(graph, size, ctx_id);
    assert(exec_id == (uint64_t)-1);  // Should fail
}
```

### 10.2 Integration Tests

```bash
# BCIB submission tests
make test-bcib

# CI gate
make ci-gate-syscall-v2-runtime
```

---

## 11. Gelecek Geliştirmeler

### 11.1 Optimization

- JIT compilation for hot graphs
- Graph caching and reuse
- Parallel execution of independent subgraphs

### 11.2 Debugging

- Graph visualization tools
- Execution tracing
- Performance profiling

### 11.3 Advanced Features

- Dynamic graph modification
- Conditional execution
- Graph composition and linking

---

## 12. Referanslar

- `kernel/sys/syscall_v2.h` - Syscall interface
- `userspace/bcib-runtime/ARCHITECTURE.md` - BCIB runtime architecture
- `ayken-core/crates/bcib/` - BCIB format implementation
- `ARCHITECTURE_FREEZE.md` - Freeze sözleşmesi

---

**© 2026 Kenan AY - AykenOS Project**
