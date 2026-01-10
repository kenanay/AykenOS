/**
 * userspace/ai-runtime/test_ai_runtime.c - AI Runtime Test
 * 
 * This file provides a simple test for the capability-based AI runtime
 * implementation to verify that Step C is working correctly.
 */

#include "lm_runtime.h"

/**
 * Test the AI runtime initialization and basic functionality
 */
int test_ai_runtime_basic(void)
{
    ai_runtime_t runtime;
    uint64_t execution_ctx_id = 1;
    
    // Test initialization
    int init_result = ai_runtime_init(&runtime, execution_ctx_id);
    if (init_result != AI_RUNTIME_SUCCESS) {
        return -1;  // Initialization failed
    }
    
    // Test status reporting
    char status_buffer[256];
    int status_result = ai_runtime_get_status(&runtime, status_buffer, sizeof(status_buffer));
    if (status_result != AI_RUNTIME_SUCCESS) {
        ai_runtime_shutdown(&runtime);
        return -2;  // Status check failed
    }
    
    // Test model loading
    int load_result = ai_runtime_load_model(&runtime, "/system/ai/test.model");
    if (load_result != AI_RUNTIME_SUCCESS) {
        // Model loading failure is acceptable for this test
    }
    
    // Test AI inference
    char output_buffer[512];
    int inference_result = ai_runtime_infer(&runtime, "Hello AI", output_buffer, sizeof(output_buffer));
    if (inference_result < 0) {
        ai_runtime_shutdown(&runtime);
        return -3;  // Inference failed
    }
    
    // Test shutdown
    int shutdown_result = ai_runtime_shutdown(&runtime);
    if (shutdown_result != AI_RUNTIME_SUCCESS) {
        return -4;  // Shutdown failed
    }
    
    return 0;  // All tests passed
}

/**
 * Test the capability system integration
 */
int test_capability_system(void)
{
    // Test capability request
    capability_token_t gpu_cap = ai_runtime_request_capability(AI_OP_GPU_ACCESS, NULL);
    if (gpu_cap.id == 0) {
        return -1;  // Failed to get GPU capability
    }
    
    capability_token_t memory_cap = ai_runtime_request_capability(AI_OP_MEMORY_ACCESS, NULL);
    if (memory_cap.id == 0) {
        return -2;  // Failed to get memory capability
    }
    
    capability_token_t ai_cap = ai_runtime_request_capability(AI_OP_INFERENCE, NULL);
    if (ai_cap.id == 0) {
        return -3;  // Failed to get AI capability
    }
    
    // Test capability validation
    ai_runtime_t runtime;
    uint64_t execution_ctx_id = 1;
    
    int init_result = ai_runtime_init(&runtime, execution_ctx_id);
    if (init_result != AI_RUNTIME_SUCCESS) {
        return -4;  // Runtime initialization failed
    }
    
    // Test valid operation
    int valid_result = ai_runtime_validate_operation(&runtime, "inference", &ai_cap);
    if (valid_result != 1) {
        ai_runtime_shutdown(&runtime);
        return -5;  // Valid operation was rejected
    }
    
    // Test dangerous operation (should be rejected)
    int dangerous_result = ai_runtime_validate_operation(&runtime, "system_shutdown", &ai_cap);
    if (dangerous_result != 0) {
        ai_runtime_shutdown(&runtime);
        return -6;  // Dangerous operation was allowed
    }
    
    ai_runtime_shutdown(&runtime);
    return 0;  // All capability tests passed
}

/**
 * Test userspace AI functions
 */
int test_userspace_functions(void)
{
    // Test userspace AI core initialization
    int core_init_result = userspace_ai_core_init();
    if (core_init_result != 0) {
        return -1;  // Core initialization failed
    }
    
    // Test userspace AI inference
    char output_buffer[256];
    int inference_result = userspace_ai_infer("Test prompt", output_buffer, sizeof(output_buffer));
    if (inference_result < 0) {
        return -2;  // Inference failed
    }
    
    // Test userspace tokenization
    int tokens[32];
    int tokenize_result = userspace_lm_tokenize("Hello world", tokens, 32);
    if (tokenize_result < 0) {
        return -3;  // Tokenization failed
    }
    
    // Test model access
    void *model = userspace_ai_get_model();
    // Model can be NULL, that's acceptable
    
    return 0;  // All userspace function tests passed
}

/**
 * Run all AI runtime tests
 */
int run_ai_runtime_tests(void)
{
    int basic_test = test_ai_runtime_basic();
    if (basic_test != 0) {
        return basic_test;
    }
    
    int capability_test = test_capability_system();
    if (capability_test != 0) {
        return capability_test + 100;  // Offset to distinguish test types
    }
    
    int userspace_test = test_userspace_functions();
    if (userspace_test != 0) {
        return userspace_test + 200;  // Offset to distinguish test types
    }
    
    return 0;  // All tests passed
}