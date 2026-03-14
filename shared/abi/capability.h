#ifndef AYKEN_CAPABILITY_H
#define AYKEN_CAPABILITY_H

#include <stdint.h>

typedef struct capability_token {
    uint64_t id;
    uint32_t permissions;
    uint32_t resource_type;
} capability_token_t;

#define CAPABILITY_RESOURCE_MEMORY      0x01
#define CAPABILITY_RESOURCE_DEVICE      0x02
#define CAPABILITY_RESOURCE_FILE        0x03
#define CAPABILITY_RESOURCE_NETWORK     0x04
#define CAPABILITY_RESOURCE_GPU         0x05
#define CAPABILITY_RESOURCE_AI_MODEL    0x06
#define CAPABILITY_RESOURCE_EXECUTION   0x07
#define CAPABILITY_RESOURCE_TIME        0x08
#define CAPABILITY_RESOURCE_IPC         0x09
#define CAPABILITY_RESOURCE_SYSTEM      0x0A

#define CAPABILITY_PERM_READ            (1 << 0)
#define CAPABILITY_PERM_WRITE           (1 << 1)
#define CAPABILITY_PERM_EXECUTE         (1 << 2)
#define CAPABILITY_PERM_DELETE          (1 << 3)
#define CAPABILITY_PERM_CREATE          (1 << 4)
#define CAPABILITY_PERM_MODIFY_META     (1 << 5)
#define CAPABILITY_PERM_GRANT           (1 << 6)
#define CAPABILITY_PERM_REVOKE          (1 << 7)
#define CAPABILITY_PERM_ADMIN           (1 << 8)
#define CAPABILITY_PERM_DEBUG           (1 << 9)
#define CAPABILITY_PERM_EXCLUSIVE       (1 << 10)
#define CAPABILITY_PERM_PERSISTENT      (1 << 11)

#define CAPABILITY_PERM_READ_WRITE      (CAPABILITY_PERM_READ | CAPABILITY_PERM_WRITE)
#define CAPABILITY_PERM_FULL_ACCESS     (CAPABILITY_PERM_READ | CAPABILITY_PERM_WRITE | \
                                        CAPABILITY_PERM_EXECUTE | CAPABILITY_PERM_DELETE)

typedef enum {
    CAPABILITY_STATE_INVALID = 0,
    CAPABILITY_STATE_ACTIVE = 1,
    CAPABILITY_STATE_SUSPENDED = 2,
    CAPABILITY_STATE_REVOKED = 3,
    CAPABILITY_STATE_EXPIRED = 4
} capability_state_t;

typedef struct capability_extended {
    capability_token_t token;
    capability_state_t state;
    uint64_t owner_context;
    uint64_t resource_address;
    uint64_t resource_size;
    uint64_t creation_time;
    uint64_t expiration_time;
    uint32_t reference_count;
    uint32_t flags;
} capability_extended_t;

#define CAPABILITY_FLAG_TRANSFERABLE    (1 << 0)
#define CAPABILITY_FLAG_DELEGATABLE     (1 << 1)
#define CAPABILITY_FLAG_AUDITABLE       (1 << 2)
#define CAPABILITY_FLAG_TIME_LIMITED    (1 << 3)
#define CAPABILITY_FLAG_SINGLE_USE      (1 << 4)
#define CAPABILITY_FLAG_CONTEXT_BOUND   (1 << 5)

capability_token_t capability_create(uint32_t resource_type, uint32_t permissions,
                                   uint64_t resource_address, uint64_t resource_size);
int capability_validate(const capability_token_t *token);
int capability_revoke(uint64_t capability_id);
int capability_suspend(uint64_t capability_id);
int capability_resume(uint64_t capability_id);
int capability_bind_to_context(uint64_t execution_ctx, const capability_token_t *token);
int capability_unbind_from_context(uint64_t execution_ctx, uint64_t capability_id);
capability_token_t *capability_get_by_context(uint64_t execution_ctx, uint32_t resource_type);
int capability_check_permission(const capability_token_t *token, uint32_t required_permission);
int capability_check_resource_access(const capability_token_t *token, uint64_t resource_address,
                                    uint64_t access_size, uint32_t access_type);
capability_token_t capability_derive(const capability_token_t *parent, uint32_t new_permissions);
int capability_transfer(const capability_token_t *token, uint64_t source_ctx, uint64_t dest_ctx);

void capability_system_init(void);
void capability_system_cleanup(void);
int capability_system_status(void);

typedef struct {
    uint64_t total_capabilities;
    uint64_t active_capabilities;
    uint64_t revoked_capabilities;
    uint64_t expired_capabilities;
    uint64_t memory_usage;
} capability_stats_t;

int capability_get_stats(capability_stats_t *stats);
void capability_dump_table(void);

static inline void *capability_to_syscall_param(const capability_token_t *token) {
    return (void *)token;
}

static inline const capability_token_t *capability_from_syscall_param(const void *param) {
    return (const capability_token_t *)param;
}

#define CAPABILITY_SUCCESS              0
#define CAPABILITY_ERROR_INVALID_TOKEN  -1
#define CAPABILITY_ERROR_PERMISSION     -2
#define CAPABILITY_ERROR_NOT_FOUND      -3
#define CAPABILITY_ERROR_ALREADY_EXISTS -4
#define CAPABILITY_ERROR_REVOKED        -5
#define CAPABILITY_ERROR_EXPIRED        -6
#define CAPABILITY_ERROR_CONTEXT_BOUND  -7
#define CAPABILITY_ERROR_NOT_TRANSFERABLE -8
#define CAPABILITY_ERROR_RESOURCE_BUSY  -9
#define CAPABILITY_ERROR_SYSTEM_LIMIT   -10

#endif
