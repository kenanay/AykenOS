#ifndef USERSPACE_AI_RUNTIME_H
#define USERSPACE_AI_RUNTIME_H

#include <stdint.h>
#include <stddef.h>

// Forward declarations for kernel types that will be used in Ring3
typedef struct lm_model lm_model_t;
typedef struct capability_token capability_token_t;

/**
 * Ring3 AI Runtime Interface
 * 
 * This interface defines the API for AI operations in Ring3 userspace.
 * It provides a secure, capability-based approach to AI inference that
 * operates independently of Ring0 kernel code.
 * 
 * Design Principles:
 * - AI operations are isolated in Ring3
 * - Resource access is mediated through capability tokens
 * - No direct Ring0 AI code dependencies
 * - Secure memory management for AI workloads
 * 
 * Requirements: FR-3.1.1, FR-3.1.4 - AI services must run isolated in Ring3
 */

// ============================================================================
// Core AI Runtime Structure
// ============================================================================

/**
 * Main AI runtime structure for Ring3 operations
 * 
 * This structure encapsulates all resources needed for AI inference
 * in userspace, including model data, workspace memory, and security
 * capabilities.
 */
typedef struct ai_runtime {
    lm_model_t *model;              // AI model data structure
    float *workspace;               // Working memory for computations
    capability_token_t gpu_access;  // GPU access capability token
    
    // Additional runtime state
    uint64_t model_size;            // Size of loaded model in bytes
    uint32_t max_tokens;            // Maximum tokens per inference
    uint32_t max_context;           // Maximum context length
    
    // Security and isolation
    uint64_t execution_context_id;  // Associated execution context
    capability_token_t memory_cap;  // Memory access capability
    capability_token_t ai_cap;      // AI operations capability
    
    // Runtime configuration
    float temperature;              // Sampling temperature (0.0-2.0)
    uint32_t max_output_tokens;     // Maximum output length
    bool initialized;               // Runtime initialization status
} ai_runtime_t;

// ============================================================================
// Ring0 Kernel Stub Interface
// ============================================================================

/**
 * Userspace AI inference function (called by kernel stubs)
 * 
 * This function is called by Ring0 kernel stubs to perform actual
 * AI inference in Ring3 userspace. This provides the bridge between
 * kernel AI interface and userspace AI implementation.
 * 
 * @param prompt Input text prompt for AI inference
 * @param out Output buffer for generated text
 * @param max_out Maximum size of output buffer
 * @return Number of characters generated, or negative error code
 * 
 * Requirements: FR-3.1.4 - AI services must be isolated in Ring3
 */
int userspace_ai_infer(const char *prompt, char *out, int max_out);

/**
 * Userspace AI core initialization function (called by kernel stubs)
 * 
 * This function is called by Ring0 kernel stubs to initialize the
 * AI core in Ring3 userspace. This provides the bridge between
 * kernel AI interface and userspace AI implementation.
 * 
 * @return 0 on success, negative error code on failure
 * 
 * Requirements: FR-3.1.4 - AI services must be isolated in Ring3
 */
int userspace_ai_core_init(void);

/**
 * Userspace AI model getter function (called by kernel stubs)
 * 
 * This function is called by Ring0 kernel stubs to get the AI model
 * from Ring3 userspace. Returns a pointer to the userspace model.
 * 
 * @return Pointer to AI model structure, or NULL on failure
 * 
 * Requirements: FR-3.1.4 - AI services must be isolated in Ring3
 */
void* userspace_ai_get_model(void);

/**
 * Userspace tokenizer function (called by kernel stubs)
 * 
 * This function is called by Ring0 kernel stubs to perform tokenization
 * in Ring3 userspace. This provides the bridge between kernel tokenizer
 * interface and userspace tokenizer implementation.
 * 
 * @param text Input text to tokenize
 * @param out_tokens Output buffer for tokens
 * @param max_tokens Maximum number of tokens to generate
 * @return Number of tokens generated, or negative error code
 * 
 * Requirements: FR-3.1.4 - AI services must be isolated in Ring3
 */
int userspace_lm_tokenize(const char *text, int *out_tokens, int max_tokens);

// ============================================================================
// AI Runtime Lifecycle Management
// ============================================================================

/**
 * Initialize AI runtime in Ring3
 * 
 * Sets up the AI runtime environment, allocates necessary memory,
 * and acquires required capability tokens for secure operation.
 * 
 * @param runtime Pointer to ai_runtime_t structure to initialize
 * @param execution_ctx_id Execution context ID for capability binding
 * @return 0 on success, negative error code on failure
 * 
 * Requirements: FR-3.1.1 - AI runtime must initialize in Ring3
 */
int ai_runtime_init(ai_runtime_t *runtime, uint64_t execution_ctx_id);

/**
 * Shutdown AI runtime and release resources
 * 
 * Cleans up all allocated resources, revokes capability tokens,
 * and ensures secure cleanup of sensitive AI data.
 * 
 * @param runtime Pointer to ai_runtime_t structure to shutdown
 * @return 0 on success, negative error code on failure
 */
int ai_runtime_shutdown(ai_runtime_t *runtime);

/**
 * Load AI model into runtime
 * 
 * Loads an AI model from the specified path using capability-based
 * file access. The model is validated and prepared for inference.
 * 
 * @param runtime Pointer to initialized ai_runtime_t
 * @param model_path Path to the AI model file
 * @return 0 on success, negative error code on failure
 * 
 * Requirements: FR-3.1.1 - Model loading must use capability system
 */
int ai_runtime_load_model(ai_runtime_t *runtime, const char *model_path);

// ============================================================================
// AI Inference Operations
// ============================================================================

/**
 * Perform AI inference on input prompt
 * 
 * Executes AI inference using the loaded model and input prompt.
 * All operations are performed in Ring3 with capability-mediated
 * resource access.
 * 
 * @param runtime Pointer to initialized ai_runtime_t with loaded model
 * @param prompt Input text prompt for inference
 * @param output Buffer to store generated output
 * @param max_output_len Maximum length of output buffer
 * @return Number of characters generated, or negative error code
 * 
 * Requirements: FR-3.1.2 - AI inference < 1 second for simple queries
 */
int ai_runtime_infer(ai_runtime_t *runtime, const char *prompt, 
                     char *output, size_t max_output_len);

/**
 * Perform streaming AI inference
 * 
 * Executes AI inference with streaming output, allowing for
 * real-time response generation and user interaction.
 * 
 * @param runtime Pointer to initialized ai_runtime_t with loaded model
 * @param prompt Input text prompt for inference
 * @param callback Function called for each generated token
 * @param callback_data User data passed to callback function
 * @return Total tokens generated, or negative error code
 */
int ai_runtime_infer_stream(ai_runtime_t *runtime, const char *prompt,
                           void (*callback)(const char *token, void *data),
                           void *callback_data);

// ============================================================================
// AI Security and Capability Management
// ============================================================================

/**
 * Request AI operation capability
 * 
 * Requests a capability token for specific AI operations.
 * This enforces the security boundary between AI suggestions
 * and system control.
 * 
 * @param operation_type Type of AI operation requested
 * @param resource_path Path to resource (if applicable)
 * @return Capability token, or invalid token on failure
 * 
 * Requirements: FR-3.4.1 - AI must never have direct system control
 */
capability_token_t ai_runtime_request_capability(uint32_t operation_type, 
                                                 const char *resource_path);

/**
 * Validate AI operation against security policy
 * 
 * Checks if a proposed AI operation is allowed under current
 * security policy. This prevents dangerous AI actions.
 * 
 * @param runtime Pointer to ai_runtime_t
 * @param operation Description of proposed operation
 * @param capability Required capability token
 * @return 1 if allowed, 0 if denied, negative on error
 * 
 * Requirements: FR-3.4.2 - All AI suggestions must pass security validation
 */
int ai_runtime_validate_operation(ai_runtime_t *runtime, 
                                  const char *operation,
                                  capability_token_t *capability);

// ============================================================================
// AI Runtime Configuration
// ============================================================================

/**
 * Configure AI runtime parameters
 * 
 * Sets runtime parameters such as temperature, max tokens,
 * and other inference configuration options.
 * 
 * @param runtime Pointer to ai_runtime_t
 * @param temperature Sampling temperature (0.0-2.0)
 * @param max_tokens Maximum tokens per inference
 * @param max_context Maximum context length
 * @return 0 on success, negative error code on failure
 */
int ai_runtime_configure(ai_runtime_t *runtime, float temperature,
                        uint32_t max_tokens, uint32_t max_context);

/**
 * Get AI runtime status and statistics
 * 
 * Retrieves current runtime status, performance metrics,
 * and resource usage information.
 * 
 * @param runtime Pointer to ai_runtime_t
 * @param status_out Buffer to store status information
 * @param status_size Size of status buffer
 * @return 0 on success, negative error code on failure
 */
int ai_runtime_get_status(ai_runtime_t *runtime, char *status_out, 
                         size_t status_size);

// ============================================================================
// Error Codes
// ============================================================================

#define AI_RUNTIME_SUCCESS           0
#define AI_RUNTIME_ERROR_INIT       -1
#define AI_RUNTIME_ERROR_MEMORY     -2
#define AI_RUNTIME_ERROR_CAPABILITY -3
#define AI_RUNTIME_ERROR_MODEL      -4
#define AI_RUNTIME_ERROR_INFERENCE  -5
#define AI_RUNTIME_ERROR_SECURITY   -6
#define AI_RUNTIME_ERROR_CONFIG     -7
#define AI_RUNTIME_ERROR_INVALID    -8

// ============================================================================
// AI Operation Types (for capability requests)
// ============================================================================

#define AI_OP_INFERENCE        0x01
#define AI_OP_MODEL_LOAD       0x02
#define AI_OP_MEMORY_ACCESS    0x04
#define AI_OP_GPU_ACCESS       0x08
#define AI_OP_FILE_ACCESS      0x10
#define AI_OP_SYSTEM_QUERY     0x20

// ============================================================================
// Capability System Integration Constants
// ============================================================================

// Capability permissions (from kernel/include/capability.h)
#define CAPABILITY_PERM_READ            (1 << 0)    // Read access
#define CAPABILITY_PERM_WRITE           (1 << 1)    // Write access
#define CAPABILITY_PERM_EXECUTE         (1 << 2)    // Execute access
#define CAPABILITY_PERM_DELETE          (1 << 3)    // Delete/destroy access

// Capability resource types (from kernel/include/capability.h)
#define CAPABILITY_RESOURCE_MEMORY      0x01    // Physical/virtual memory regions
#define CAPABILITY_RESOURCE_DEVICE      0x02    // Hardware devices (DevFS)
#define CAPABILITY_RESOURCE_FILE        0x03    // File system objects (VFS)
#define CAPABILITY_RESOURCE_NETWORK     0x04    // Network interfaces
#define CAPABILITY_RESOURCE_GPU         0x05    // GPU compute resources
#define CAPABILITY_RESOURCE_AI_MODEL    0x06    // AI model weights and inference
#define CAPABILITY_RESOURCE_EXECUTION   0x07    // Execution contexts and BCIB graphs
#define CAPABILITY_RESOURCE_TIME        0x08    // Timer and scheduling resources
#define CAPABILITY_RESOURCE_IPC         0x09    // Inter-process communication
#define CAPABILITY_RESOURCE_SYSTEM      0x0A    // System-level operations

// ============================================================================
// AI Runtime Limits and Constants
// ============================================================================

#define AI_MAX_PROMPT_LENGTH    4096
#define AI_MAX_OUTPUT_LENGTH    2048
#define AI_MAX_MODEL_SIZE       (256 * 1024 * 1024)  // 256MB
#define AI_DEFAULT_TEMPERATURE  0.7f
#define AI_DEFAULT_MAX_TOKENS   256
#define AI_DEFAULT_MAX_CONTEXT  512

#endif // USERSPACE_AI_RUNTIME_H