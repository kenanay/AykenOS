# Capability System Reference

**Project:** AykenOS  
**Version:** 1.0  
**Status:** ACTIVE  
**Effective Date:** 2026-02-21  
**Owner:** AykenOS Core Architecture Team

---

## 1. Genel Bakış

AykenOS capability-based security sistemi, geleneksel permission modellerini unforgeable token'larla değiştirir. Bu sistem, execution-centric mimarinin güvenlik temelini oluşturur.

### Temel Prensipler

- **Unforgeable Tokens:** Capability token'lar forge edilemez
- **Least Privilege:** Her execution context sadece ihtiyacı olan capability'lere sahip
- **Syscall-Only Binding:** Capability'ler sadece syscall ile bind edilir
- **No Kernel Bypass:** Kernel, capability olmadan kaynak erişimine izin vermez

---

## 2. Capability Token Yapısı

### 2.1 Core Token

```c
typedef struct capability_token {
    uint64_t id;              // Unique capability identifier
    uint32_t permissions;     // Permission bitmask
    uint32_t resource_type;   // Resource type identifier
} capability_token_t;
```

### 2.2 Extended Capability

```c
typedef struct capability_extended {
    capability_token_t token;     // Core token
    uint32_t state;               // Capability state
    uint64_t owner_context;       // Owning execution context
    uint64_t resource_address;    // Physical resource address
    uint64_t resource_size;       // Resource size in bytes
    uint64_t creation_time;       // Creation timestamp
    uint64_t expiration_time;     // Expiration timestamp (0 = no expiry)
    uint32_t reference_count;     // Reference count
    uint32_t flags;               // Additional flags
} capability_extended_t;
```

---

## 3. Permission Matrix

### 3.1 Permission Flags

| Flag | Value | Açıklama |
|------|-------|----------|
| `CAP_PERM_READ` | 0x01 | Read access permission |
| `CAP_PERM_WRITE` | 0x02 | Write access permission |
| `CAP_PERM_EXECUTE` | 0x04 | Execute access permission |
| `CAP_PERM_ADMIN` | 0x08 | Administrative access permission |

### 3.2 Permission Combinations

```c
// Read-only access
uint32_t perms = CAP_PERM_READ;

// Read-write access
uint32_t perms = CAP_PERM_READ | CAP_PERM_WRITE;

// Execute-only access (code segment)
uint32_t perms = CAP_PERM_EXECUTE;

// Full access
uint32_t perms = CAP_PERM_READ | CAP_PERM_WRITE | 
                 CAP_PERM_EXECUTE | CAP_PERM_ADMIN;
```

### 3.3 Resource Type Matrix

| Resource Type | ID | Permissions | Açıklama |
|---------------|-----|-------------|----------|
| `CAP_RESOURCE_MEMORY` | 1 | R/W/X | Memory region access |
| `CAP_RESOURCE_DEVICE` | 2 | R/W/A | Device access |
| `CAP_RESOURCE_EXECUTION` | 3 | X/A | Execution context access |
| `CAP_RESOURCE_TIME` | 4 | R | Time service access |

---

## 4. Capability Lifecycle

### 4.1 Creation

```c
// Capability oluştur
capability_token_t token = capability_create(
    CAP_RESOURCE_MEMORY,           // Resource type
    CAP_PERM_READ | CAP_PERM_WRITE, // Permissions
    0x100000,                       // Physical address
    4096                            // Size (4KB)
);

if (token.id == 0) {
    // Creation failed
    handle_error();
}
```

### 4.2 Binding

```c
// Capability'yi execution context'e bind et
uint64_t result = sys_v2_capability_bind(
    execution_ctx_id,  // Execution context ID
    &token             // Capability token
);

if (result != ESYS_V2_SUCCESS) {
    // Binding failed
    handle_error();
}
```

### 4.3 Validation

```c
// Capability'yi validate et
int result = capability_validate(&token);

if (result != 0) {
    // Validation failed
    switch (result) {
        case CAPABILITY_ERROR_INVALID_TOKEN:
            // Token geçersiz
            break;
        case CAPABILITY_ERROR_NOT_FOUND:
            // Capability bulunamadı
            break;
        case CAPABILITY_ERROR_EXPIRED:
            // Capability süresi dolmuş
            break;
        case CAPABILITY_ERROR_REVOKED:
            // Capability iptal edilmiş
            break;
    }
}
```

### 4.4 Revocation

```c
// Capability'yi iptal et
uint64_t result = sys_v2_capability_revoke(token.id);

if (result != ESYS_V2_SUCCESS) {
    // Revocation failed
    handle_error();
}
```

---

## 5. Capability States

### 5.1 State Diagram

```
CREATED → ACTIVE → REVOKED
    ↓        ↓
    └─→ EXPIRED
```

### 5.2 State Definitions

| State | Value | Açıklama |
|-------|-------|----------|
| `CAPABILITY_STATE_CREATED` | 0x01 | Capability oluşturuldu, henüz bind edilmedi |
| `CAPABILITY_STATE_ACTIVE` | 0x02 | Capability aktif ve kullanılabilir |
| `CAPABILITY_STATE_EXPIRED` | 0x04 | Capability süresi doldu |
| `CAPABILITY_STATE_REVOKED` | 0x08 | Capability iptal edildi |

### 5.3 State Transitions

```c
// CREATED → ACTIVE (binding sırasında)
capability_bind(ctx_id, &token);

// ACTIVE → EXPIRED (expiration time geçince)
if (current_time > cap->expiration_time) {
    cap->state = CAPABILITY_STATE_EXPIRED;
}

// ACTIVE → REVOKED (revocation sırasında)
capability_revoke(token.id);
```

---

## 6. Execution Context Bindings

### 6.1 Context-Capability Mapping

Her execution context, birden fazla capability'ye sahip olabilir:

```c
typedef struct execution_context_capabilities {
    uint64_t context_id;                              // Context ID
    uint64_t capability_ids[MAX_CAPABILITIES_PER_CONTEXT]; // Capability IDs
    uint32_t capability_count;                        // Number of capabilities
} execution_context_capabilities_t;
```

### 6.2 Binding Limits

```c
#define MAX_CAPABILITIES_PER_CONTEXT 32  // Per-context limit
#define MAX_EXECUTION_CONTEXTS 256       // System-wide limit
#define MAX_CAPABILITIES 1024            // Total capability limit
```

### 6.3 Binding API

```c
// Context için capability bind et
int bind_capability_to_context(uint64_t ctx_id, uint64_t cap_id) {
    execution_context_capabilities_t *ctx_caps = 
        find_context_capabilities(ctx_id);
    
    if (!ctx_caps) {
        ctx_caps = create_context_capabilities(ctx_id);
        if (!ctx_caps) return -1;
    }
    
    if (ctx_caps->capability_count >= MAX_CAPABILITIES_PER_CONTEXT) {
        return -1;  // Limit exceeded
    }
    
    ctx_caps->capability_ids[ctx_caps->capability_count++] = cap_id;
    return 0;
}
```

---

## 7. Security Enforcement

### 7.1 Memory Access Check

```c
// Memory access öncesi capability check
bool check_memory_access(uint64_t ctx_id, uint64_t addr, 
                        uint64_t size, uint32_t required_perms) {
    execution_context_capabilities_t *ctx_caps = 
        find_context_capabilities(ctx_id);
    
    if (!ctx_caps) return false;
    
    // Her capability'yi kontrol et
    for (uint32_t i = 0; i < ctx_caps->capability_count; i++) {
        capability_extended_t *cap = 
            find_capability_by_id(ctx_caps->capability_ids[i]);
        
        if (!cap) continue;
        if (cap->state != CAPABILITY_STATE_ACTIVE) continue;
        if (cap->token.resource_type != CAP_RESOURCE_MEMORY) continue;
        
        // Address range check
        uint64_t cap_start = cap->resource_address;
        uint64_t cap_end = cap_start + cap->resource_size;
        
        if (addr >= cap_start && (addr + size) <= cap_end) {
            // Permission check
            if ((cap->token.permissions & required_perms) == required_perms) {
                return true;  // Access granted
            }
        }
    }
    
    return false;  // Access denied
}
```

### 7.2 Syscall Integration

```c
// sys_v2_map_memory implementation
uint64_t sys_v2_map_memory(uint64_t virt_addr, uint64_t phys_addr, 
                          uint64_t flags) {
    // Get current execution context
    uint64_t ctx_id = get_current_execution_context();
    
    // Check if context has memory capability
    if (!check_memory_access(ctx_id, phys_addr, PAGE_SIZE, 
                            CAP_PERM_READ | CAP_PERM_WRITE)) {
        return ESYS_V2_NO_CAPABILITY;
    }
    
    // Proceed with mapping
    return do_map_memory(virt_addr, phys_addr, flags);
}
```

### 7.3 Privilege Escalation Prevention

```c
// Capability creation sadece privileged context'ten
capability_token_t capability_create(...) {
    uint64_t ctx_id = get_current_execution_context();
    
    // Check if context has admin capability
    if (!has_admin_capability(ctx_id)) {
        return invalid_token;  // Denied
    }
    
    // Proceed with creation
    ...
}
```

---

## 8. Error Codes

### 8.1 Capability Errors

| Error Code | Value | Açıklama |
|------------|-------|----------|
| `CAPABILITY_ERROR_SUCCESS` | 0 | Operation successful |
| `CAPABILITY_ERROR_INVALID_TOKEN` | -1 | Invalid capability token |
| `CAPABILITY_ERROR_NOT_FOUND` | -2 | Capability not found |
| `CAPABILITY_ERROR_EXPIRED` | -3 | Capability expired |
| `CAPABILITY_ERROR_REVOKED` | -4 | Capability revoked |
| `CAPABILITY_ERROR_NO_PERMISSION` | -5 | Insufficient permissions |
| `CAPABILITY_ERROR_TABLE_FULL` | -6 | Capability table full |
| `CAPABILITY_ERROR_CONTEXT_FULL` | -7 | Context capability limit reached |

### 8.2 Syscall Errors

| Error Code | Value | Açıklama |
|------------|-------|----------|
| `ESYS_V2_NO_CAPABILITY` | -5 | Missing required capability |
| `ESYS_V2_NO_PERMISSION` | -3 | Insufficient permissions |

---

## 9. Performance Characteristics

### 9.1 Latency

- **Capability Creation:** < 1 μs
- **Capability Binding:** < 500 ns
- **Capability Validation:** < 200 ns
- **Access Check:** < 100 ns (per capability)

### 9.2 Memory Overhead

- **Per Capability:** 64 bytes (extended)
- **Per Context Binding:** 32 bytes + (8 bytes × capability count)
- **Total System:** < 100 KB (typical workload)

### 9.3 Scalability

- **Max Capabilities:** 1024 (system-wide)
- **Max Contexts:** 256
- **Max Capabilities per Context:** 32

---

## 10. Best Practices

### 10.1 Least Privilege

```c
// BAD: Full access capability
capability_token_t token = capability_create(
    CAP_RESOURCE_MEMORY,
    CAP_PERM_READ | CAP_PERM_WRITE | CAP_PERM_EXECUTE | CAP_PERM_ADMIN,
    addr, size
);

// GOOD: Minimal required permissions
capability_token_t token = capability_create(
    CAP_RESOURCE_MEMORY,
    CAP_PERM_READ,  // Read-only
    addr, size
);
```

### 10.2 Capability Lifetime

```c
// BAD: No expiration
capability_token_t token = capability_create(...);
token.expiration_time = 0;  // Never expires

// GOOD: Time-limited capability
capability_token_t token = capability_create(...);
token.expiration_time = current_time + 3600;  // 1 hour
```

### 10.3 Revocation

```c
// Capability'yi kullanım sonrası revoke et
capability_token_t token = capability_create(...);
use_capability(&token);
sys_v2_capability_revoke(token.id);  // Cleanup
```

---

## 11. Testing

### 11.1 Unit Tests

```c
// Test: Capability creation
void test_capability_creation(void) {
    capability_token_t token = capability_create(
        CAP_RESOURCE_MEMORY, CAP_PERM_READ, 0x1000, 4096
    );
    assert(token.id != 0);
    assert(token.resource_type == CAP_RESOURCE_MEMORY);
    assert(token.permissions == CAP_PERM_READ);
}

// Test: Access control
void test_access_control(void) {
    capability_token_t token = capability_create(
        CAP_RESOURCE_MEMORY, CAP_PERM_READ, 0x1000, 4096
    );
    
    // Read access should succeed
    assert(check_memory_access(ctx_id, 0x1000, 100, CAP_PERM_READ));
    
    // Write access should fail
    assert(!check_memory_access(ctx_id, 0x1000, 100, CAP_PERM_WRITE));
}

// Test: Revocation
void test_revocation(void) {
    capability_token_t token = capability_create(...);
    sys_v2_capability_revoke(token.id);
    
    // Access should fail after revocation
    assert(!check_memory_access(ctx_id, addr, size, CAP_PERM_READ));
}
```

### 11.2 Integration Tests

```bash
# Capability system tests
make test-capability

# CI gate
make ci-gate-boundary
```

---

## 12. Gelecek Geliştirmeler

### 12.1 Delegation

- Capability delegation (transfer between contexts)
- Derived capabilities (subset of permissions)
- Capability chains (hierarchical delegation)

### 12.2 Auditing

- Capability usage logging
- Access pattern analysis
- Security audit trail

### 12.3 Dynamic Permissions

- Runtime permission adjustment
- Conditional capabilities
- Context-aware permissions

---

## 13. Referanslar

- `kernel/include/capability.h` - Capability type definitions
- `kernel/sys/capability_manager.c` - Capability implementation
- `kernel/sys/syscall_v2.h` - Syscall interface
- `ARCHITECTURE_FREEZE.md` - Freeze sözleşmesi

---

**© 2026 Kenan AY - AykenOS Project**
