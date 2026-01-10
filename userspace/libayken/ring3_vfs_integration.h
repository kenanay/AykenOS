/**
 * @file ring3_vfs_integration.h
 * @brief Ring3 VFS Integration Header
 * 
 * This header provides integration functions for using the Ring3 VFS
 * implementation from kernel code. It demonstrates how VFS operations
 * work entirely via Ring0 mechanism only using the new syscall interface.
 * 
 * @author Kenan AY
 * @project AykenOS - Advanced AI-Integrated Operating System
 * @created January 3, 2026
 * @phase Phase 2.2 - Ring3 Runtime Development
 * @task 2.2.1.3 - Implement Ring3 VFS using new syscalls (Step C)
 */

#ifndef RING3_VFS_INTEGRATION_H
#define RING3_VFS_INTEGRATION_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ========================================================================
 * Ring3 VFS Integration Functions
 * ======================================================================== */

/**
 * @brief Initialize Ring3 VFS system
 * 
 * Initializes the complete Ring3 VFS implementation using the new
 * execution-centric syscall interface. This function sets up:
 * 
 * 1. Ring0 proxy VFS implementation
 * 2. Capability system integration
 * 3. Memory mapping subsystem
 * 4. Syscall interface functions
 * 
 * Requirements: VFS operations via Ring0 mechanism only
 * 
 * @return 0 on success, -1 on error
 */
int ring3_vfs_initialize(void);

/**
 * @brief Demonstrate Ring3 VFS functionality
 * 
 * This function demonstrates the complete Ring3 VFS implementation
 * by performing various file operations using sys_v2_map_memory
 * and capability tokens.
 * 
 * Operations demonstrated:
 * - File opening with capability tokens
 * - Memory-mapped file reading
 * - File seeking within mapped regions
 * - File closing and resource cleanup
 * - Multiple concurrent file access
 * 
 * @return 0 on success, -1 on error
 */
int ring3_vfs_demonstrate(void);

/**
 * @brief Test Ring3 VFS performance
 * 
 * Performs performance testing of the Ring3 VFS implementation
 * to ensure it meets the requirements for Ring0 mechanism efficiency.
 * 
 * @return 0 on success, -1 on error
 */
int ring3_vfs_performance_test(void);

/**
 * @brief Get Ring3 VFS statistics
 * 
 * Returns statistics about Ring3 VFS operations for monitoring
 * and debugging purposes.
 * 
 * @param stats_buffer Buffer to store statistics
 * @param buffer_size Size of statistics buffer
 * @return Number of bytes written to buffer, -1 on error
 */
int ring3_vfs_get_statistics(char *stats_buffer, size_t buffer_size);

/* ========================================================================
 * Ring3 VFS Test Functions
 * ======================================================================== */

/**
 * @brief Run comprehensive VFS test suite
 * 
 * Runs all VFS tests to validate the Ring3 implementation.
 * 
 * @return 0 if all tests pass, -1 if any test fails
 */
int run_vfs_tests(void);

/**
 * @brief Test basic VFS operations
 * 
 * Tests basic file operations using the Ring3 VFS.
 * 
 * @return 0 on success, -1 on error
 */
int vfs_test_basic_operations(void);

/**
 * @brief Test multiple file operations
 * 
 * Tests concurrent access to multiple files.
 * 
 * @return 0 on success, -1 on error
 */
int vfs_test_multiple_files(void);

/* ========================================================================
 * Ring3 VFS Configuration
 * ======================================================================== */

/**
 * @brief VFS configuration structure
 */
typedef struct {
    uint32_t max_open_files;        /**< Maximum number of open files */
    uint32_t max_mmap_regions;      /**< Maximum memory-mapped regions */
    uint32_t default_file_size;     /**< Default file size for new files */
    uint32_t capability_timeout;    /**< Capability token timeout (seconds) */
    uint8_t  enable_statistics;     /**< Enable statistics collection */
    uint8_t  enable_debug_logging;  /**< Enable debug logging */
} ring3_vfs_config_t;

/**
 * @brief Configure Ring3 VFS system
 * 
 * Configures the Ring3 VFS with custom parameters.
 * 
 * @param config Configuration structure
 * @return 0 on success, -1 on error
 */
int ring3_vfs_configure(const ring3_vfs_config_t *config);

/**
 * @brief Get default Ring3 VFS configuration
 * 
 * Returns the default configuration for Ring3 VFS.
 * 
 * @param config Pointer to configuration structure to fill
 * @return 0 on success, -1 on error
 */
int ring3_vfs_get_default_config(ring3_vfs_config_t *config);

/* ========================================================================
 * Ring3 VFS Status and Monitoring
 * ======================================================================== */

/**
 * @brief VFS status information
 */
typedef struct {
    uint32_t files_open;            /**< Currently open files */
    uint32_t mmap_regions_active;   /**< Active memory mappings */
    uint64_t total_bytes_read;      /**< Total bytes read */
    uint64_t total_bytes_written;   /**< Total bytes written */
    uint64_t syscalls_made;         /**< Total syscalls made */
    uint64_t capabilities_active;   /**< Active capability tokens */
    uint32_t last_error_code;       /**< Last error code */
    char     last_error_msg[128];   /**< Last error message */
} ring3_vfs_status_t;

/**
 * @brief Get Ring3 VFS status
 * 
 * Returns current status and statistics of the Ring3 VFS system.
 * 
 * @param status Pointer to status structure to fill
 * @return 0 on success, -1 on error
 */
int ring3_vfs_get_status(ring3_vfs_status_t *status);

/**
 * @brief Reset Ring3 VFS statistics
 * 
 * Resets all statistics counters to zero.
 * 
 * @return 0 on success, -1 on error
 */
int ring3_vfs_reset_statistics(void);

#ifdef __cplusplus
}
#endif

#endif /* RING3_VFS_INTEGRATION_H */