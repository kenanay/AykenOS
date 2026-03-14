// userspace/libayken/vfs_test.c
// AykenOS Phase 2.2 - Ring3 VFS Library Test
//
// Test suite for the Ring3 VFS interface to validate API design
// and ensure proper integration with capability system.
//
// Author: Kenan AY
// Project: AykenOS - Advanced AI-Integrated Operating System
// Created: January 10, 2026

#include "../vfs.h"

// Forward declarations
void fb_print(const char *s);
void fb_print_int(int64_t value);
void fb_print_hex(uint64_t v);

// Simple string length function
static size_t strlen(const char *str) {
    size_t len = 0;
    if (str) {
        while (str[len]) len++;
    }
    return len;
}

// Simple memset function
static void *memset(void *s, int c, size_t n) {
    unsigned char *p = (unsigned char *)s;
    if (p) {
        while (n--) {
            *p++ = (unsigned char)c;
        }
    }
    return s;
}

// ============================================================================
// TEST FRAMEWORK
// ============================================================================

static int g_test_count = 0;
static int g_test_passed = 0;
static int g_test_failed = 0;

#define TEST_ASSERT(condition, message) \
    do { \
        g_test_count++; \
        if (condition) { \
            g_test_passed++; \
            fb_print("[PASS] "); \
            fb_print(message); \
            fb_print("\n"); \
        } else { \
            g_test_failed++; \
            fb_print("[FAIL] "); \
            fb_print(message); \
            fb_print("\n"); \
        } \
    } while(0)

#define TEST_SECTION(name) \
    do { \
        fb_print("\n=== "); \
        fb_print(name); \
        fb_print(" ===\n"); \
    } while(0)

// ============================================================================
// VFS API DESIGN TESTS
// ============================================================================

/**
 * test_vfs_initialization - Test VFS library initialization
 */
void test_vfs_initialization(void)
{
    TEST_SECTION("VFS Initialization Tests");
    
    // Test VFS initialization
    int result = vfs_init_userspace();
    TEST_ASSERT(result == VFS_SUCCESS, "VFS initialization should succeed");
    
    // Test getting VFS interface
    userspace_vfs_t *vfs = get_userspace_vfs();
    TEST_ASSERT(vfs != NULL, "VFS interface should be available");
    
    // Test function pointers are set
    TEST_ASSERT(vfs->open != NULL, "VFS open function should be set");
    TEST_ASSERT(vfs->close != NULL, "VFS close function should be set");
    TEST_ASSERT(vfs->read != NULL, "VFS read function should be set");
    TEST_ASSERT(vfs->write != NULL, "VFS write function should be set");
    TEST_ASSERT(vfs->seek != NULL, "VFS seek function should be set");
    
    // Test directory operations
    TEST_ASSERT(vfs->mkdir != NULL, "VFS mkdir function should be set");
    TEST_ASSERT(vfs->rmdir != NULL, "VFS rmdir function should be set");
    TEST_ASSERT(vfs->readdir != NULL, "VFS readdir function should be set");
    
    // Test file system operations
    TEST_ASSERT(vfs->stat != NULL, "VFS stat function should be set");
    TEST_ASSERT(vfs->unlink != NULL, "VFS unlink function should be set");
    TEST_ASSERT(vfs->rename != NULL, "VFS rename function should be set");
    
    // Test capability operations
    TEST_ASSERT(vfs->request_file_capability != NULL, "VFS capability request should be set");
    TEST_ASSERT(vfs->bind_file_capability != NULL, "VFS capability bind should be set");
    TEST_ASSERT(vfs->revoke_file_capability != NULL, "VFS capability revoke should be set");
}

/**
 * test_vfs_context_management - Test VFS context management
 */
void test_vfs_context_management(void)
{
    TEST_SECTION("VFS Context Management Tests");
    
    // Test context creation
    uint64_t exec_ctx_id = 1001;
    vfs_context_t *context = vfs_create_context(exec_ctx_id);
    TEST_ASSERT(context != NULL, "VFS context creation should succeed");
    TEST_ASSERT(context->execution_context_id == exec_ctx_id, "Context should have correct execution context ID");
    TEST_ASSERT(context->capability_count == 0, "New context should have no capabilities");
    
    // Test setting current context
    int result = vfs_set_current_context(context);
    TEST_ASSERT(result == VFS_SUCCESS, "Setting current context should succeed");
    
    // Test getting current context
    vfs_context_t *current = vfs_get_current_context();
    TEST_ASSERT(current == context, "Current context should match set context");
    
    // Test context cleanup
    vfs_destroy_context(context);
    fb_print("[INFO] Context destroyed successfully\n");
}

/**
 * test_vfs_file_operations - Test basic file operations
 */
void test_vfs_file_operations(void)
{
    TEST_SECTION("VFS File Operations Tests");
    
    userspace_vfs_t *vfs = get_userspace_vfs();
    
    // Test file opening
    int fd = vfs->open("/test/file.txt", VFS_MODE_READ);
    TEST_ASSERT(fd >= 0, "File open should return valid file descriptor");
    
    if (fd >= 0) {
        // Test file reading (stub implementation)
        char buffer[64];
        ssize_t bytes_read = vfs->read(fd, buffer, sizeof(buffer));
        TEST_ASSERT(bytes_read >= 0, "File read should not return error");
        
        // Test file seeking
        off_t new_pos = vfs->seek(fd, 10, VFS_SEEK_SET);
        TEST_ASSERT(new_pos >= 0, "File seek should return valid position");
        
        // Test file closing
        int close_result = vfs->close(fd);
        TEST_ASSERT(close_result == VFS_SUCCESS, "File close should succeed");
    }
    
    // Test invalid file descriptor operations
    ssize_t read_result = vfs->read(-1, NULL, 0);
    TEST_ASSERT(read_result == VFS_ERROR_INVALID_FD, "Read with invalid FD should return error");
    
    int close_result = vfs->close(-1);
    TEST_ASSERT(close_result == VFS_ERROR_INVALID_FD, "Close with invalid FD should return error");
}

/**
 * test_vfs_write_operations - Test file write operations
 */
void test_vfs_write_operations(void)
{
    TEST_SECTION("VFS Write Operations Tests");
    
    userspace_vfs_t *vfs = get_userspace_vfs();
    
    // Test file opening for write
    int fd = vfs->open("/test/write_test.txt", VFS_MODE_CREATE | VFS_MODE_WRITE);
    TEST_ASSERT(fd >= 0, "File open for write should return valid file descriptor");
    
    if (fd >= 0) {
        // Test file writing
        const char *test_data = "Hello, AykenOS Ring3 VFS!";
        ssize_t bytes_written = vfs->write(fd, test_data, strlen(test_data));
        TEST_ASSERT(bytes_written == (ssize_t)strlen(test_data), "File write should return correct byte count");
        
        // Test file sync
        int sync_result = vfs->sync(fd);
        TEST_ASSERT(sync_result == VFS_ERROR_NOT_SUPPORTED || sync_result == VFS_SUCCESS, 
                   "File sync should return expected result");
        
        // Close file
        vfs->close(fd);
    }
    
    // Test write to read-only file
    fd = vfs->open("/test/readonly.txt", VFS_MODE_READ);
    if (fd >= 0) {
        const char *data = "test";
        ssize_t write_result = vfs->write(fd, data, strlen(data));
        TEST_ASSERT(write_result == VFS_ERROR_PERMISSION, "Write to read-only file should fail");
        vfs->close(fd);
    }
}

/**
 * test_vfs_capability_integration - Test capability system integration
 */
void test_vfs_capability_integration(void)
{
    TEST_SECTION("VFS Capability Integration Tests");
    
    userspace_vfs_t *vfs = get_userspace_vfs();
    
    // Test capability request
    capability_token_t cap = vfs->request_file_capability("/test/cap_test.txt", CAPABILITY_PERM_READ);
    TEST_ASSERT(cap.resource_type == CAPABILITY_RESOURCE_FILE, "Capability should be for file resource");
    TEST_ASSERT(cap.permissions == CAPABILITY_PERM_READ, "Capability should have read permission");
    
    // Test capability binding (will use stub implementation)
    vfs_context_t *context = vfs_create_context(1002);
    vfs_set_current_context(context);
    
    int bind_result = vfs->bind_file_capability(&cap);
    TEST_ASSERT(bind_result == VFS_SUCCESS || bind_result == VFS_ERROR_CAPABILITY, 
               "Capability bind should return expected result");
    
    // Test capability revocation
    if (cap.id != 0) {
        int revoke_result = vfs->revoke_file_capability(cap.id);
        TEST_ASSERT(revoke_result == VFS_SUCCESS || revoke_result == VFS_ERROR_CAPABILITY,
                   "Capability revoke should return expected result");
    }
    
    vfs_destroy_context(context);
}

/**
 * test_vfs_directory_operations - Test directory operations
 */
void test_vfs_directory_operations(void)
{
    TEST_SECTION("VFS Directory Operations Tests");
    
    userspace_vfs_t *vfs = get_userspace_vfs();
    
    // Test directory creation (stub implementation)
    int mkdir_result = vfs->mkdir("/test/new_dir", 0755);
    TEST_ASSERT(mkdir_result == VFS_ERROR_NOT_SUPPORTED, "mkdir should return not supported (stub)");
    
    // Test directory removal (stub implementation)
    int rmdir_result = vfs->rmdir("/test/empty_dir");
    TEST_ASSERT(rmdir_result == VFS_ERROR_NOT_SUPPORTED, "rmdir should return not supported (stub)");
    
    // Test directory reading (stub implementation)
    int dir_fd = vfs->open("/test", VFS_MODE_READ);
    if (dir_fd >= 0) {
        vfs_directory_entry_t *entries = NULL;
        int readdir_result = vfs->readdir(dir_fd, &entries);
        TEST_ASSERT(readdir_result == VFS_ERROR_NOT_SUPPORTED, "readdir should return not supported (stub)");
        vfs->close(dir_fd);
    }
}

/**
 * test_vfs_path_utilities - Test path utility functions
 */
void test_vfs_path_utilities(void)
{
    TEST_SECTION("VFS Path Utilities Tests");
    
    // Test absolute path detection
    int is_abs1 = vfs_is_absolute_path("/absolute/path");
    TEST_ASSERT(is_abs1 == 1, "Absolute path should be detected");
    
    int is_abs2 = vfs_is_absolute_path("relative/path");
    TEST_ASSERT(is_abs2 == 0, "Relative path should be detected");
    
    int is_abs3 = vfs_is_absolute_path("");
    TEST_ASSERT(is_abs3 == 0, "Empty path should be relative");
    
    // Test path normalization
    char normalized[256];
    int norm_result = vfs_path_normalize("/test//path/", normalized, sizeof(normalized));
    TEST_ASSERT(norm_result == VFS_SUCCESS, "Path normalization should succeed");
    
    // Test path too long
    char long_path[1024];
    memset(long_path, 'a', sizeof(long_path) - 1);
    long_path[sizeof(long_path) - 1] = '\0';
    
    char small_buffer[10];
    int long_result = vfs_path_normalize(long_path, small_buffer, sizeof(small_buffer));
    TEST_ASSERT(long_result == VFS_ERROR_NAME_TOO_LONG, "Long path should return error");
}

/**
 * test_vfs_error_handling - Test error handling
 */
void test_vfs_error_handling(void)
{
    TEST_SECTION("VFS Error Handling Tests");
    
    userspace_vfs_t *vfs = get_userspace_vfs();
    
    // Test NULL path handling
    int fd = vfs->open(NULL, VFS_MODE_READ);
    TEST_ASSERT(fd == VFS_ERROR_INVALID_PATH, "NULL path should return error");
    
    // Test invalid buffer handling
    fd = vfs->open("/test/file.txt", VFS_MODE_READ);
    if (fd >= 0) {
        ssize_t read_result = vfs->read(fd, NULL, 100);
        TEST_ASSERT(read_result < 0, "NULL buffer should return error");
        vfs->close(fd);
    }
    
    // Test capability errors
    capability_token_t invalid_cap = {0};
    int bind_result = vfs->bind_file_capability(&invalid_cap);
    TEST_ASSERT(bind_result == VFS_ERROR_CAPABILITY, "Invalid capability should return error");
    
    int revoke_result = vfs->revoke_file_capability(0);
    TEST_ASSERT(revoke_result == VFS_ERROR_CAPABILITY, "Invalid capability ID should return error");
}

/**
 * test_vfs_statistics - Test VFS statistics
 */
void test_vfs_statistics(void)
{
    TEST_SECTION("VFS Statistics Tests");
    
    // Reset statistics
    vfs_reset_stats();
    
    // Get initial statistics
    vfs_stats_t stats;
    int result = vfs_get_stats(&stats);
    TEST_ASSERT(result == VFS_SUCCESS, "Getting stats should succeed");
    TEST_ASSERT(stats.open_calls == 0, "Initial open calls should be zero");
    TEST_ASSERT(stats.read_calls == 0, "Initial read calls should be zero");
    
    // Perform some operations to update statistics
    userspace_vfs_t *vfs = get_userspace_vfs();
    int fd = vfs->open("/test/stats_test.txt", VFS_MODE_READ);
    if (fd >= 0) {
        char buffer[64];
        vfs->read(fd, buffer, sizeof(buffer));
        vfs->close(fd);
    }
    
    // Check updated statistics
    result = vfs_get_stats(&stats);
    TEST_ASSERT(result == VFS_SUCCESS, "Getting updated stats should succeed");
    TEST_ASSERT(stats.open_calls > 0, "Open calls should be incremented");
    TEST_ASSERT(stats.read_calls > 0, "Read calls should be incremented");
    
    fb_print("[INFO] VFS Statistics:\n");
    fb_print("  Open calls: ");
    fb_print_int(stats.open_calls);
    fb_print("\n  Read calls: ");
    fb_print_int(stats.read_calls);
    fb_print("\n  Close calls: ");
    fb_print_int(stats.close_calls);
    fb_print("\n");
}

/**
 * test_vfs_advanced_features - Test advanced VFS features
 */
void test_vfs_advanced_features(void)
{
    TEST_SECTION("VFS Advanced Features Tests");
    
    userspace_vfs_t *vfs = get_userspace_vfs();
    
    // Test file stat operations (stub)
    vfs_file_info_t info;
    int stat_result = vfs->stat("/test/file.txt", &info);
    TEST_ASSERT(stat_result == VFS_ERROR_NOT_SUPPORTED, "stat should return not supported (stub)");
    
    // Test file linking operations (stub)
    int link_result = vfs->link("/test/target.txt", "/test/link.txt");
    TEST_ASSERT(link_result == VFS_ERROR_NOT_SUPPORTED, "link should return not supported (stub)");
    
    int symlink_result = vfs->symlink("/test/target.txt", "/test/symlink.txt");
    TEST_ASSERT(symlink_result == VFS_ERROR_NOT_SUPPORTED, "symlink should return not supported (stub)");
    
    // Test mount operations (stub)
    int mount_result = vfs->mount("/dev/disk", "/mnt/test", "ext4", 0, NULL);
    TEST_ASSERT(mount_result == VFS_ERROR_NOT_SUPPORTED, "mount should return not supported (stub)");
    
    // Test memory mapping (stub)
    int fd = vfs->open("/test/mmap_test.txt", VFS_MODE_READ);
    if (fd >= 0) {
        void *mapped = vfs->mmap_file(fd, 0, 4096, 0);
        TEST_ASSERT(mapped == NULL, "mmap should return NULL (stub)");
        vfs->close(fd);
    }
}

// ============================================================================
// MAIN TEST RUNNER
// ============================================================================

/**
 * run_vfs_api_tests - Run all VFS API design tests
 */
void run_vfs_api_tests(void)
{
    fb_print("\n");
    fb_print("========================================\n");
    fb_print("AykenOS Ring3 VFS API Design Tests\n");
    fb_print("Phase 2.2 - Step A: API Design\n");
    fb_print("========================================\n");
    
    // Initialize test counters
    g_test_count = 0;
    g_test_passed = 0;
    g_test_failed = 0;
    
    // Run test suites
    test_vfs_initialization();
    test_vfs_context_management();
    test_vfs_file_operations();
    test_vfs_write_operations();
    test_vfs_capability_integration();
    test_vfs_directory_operations();
    test_vfs_path_utilities();
    test_vfs_error_handling();
    test_vfs_statistics();
    test_vfs_advanced_features();
    
    // Cleanup
    vfs_cleanup_userspace();
    
    // Print test summary
    fb_print("\n========================================\n");
    fb_print("VFS API Design Test Summary\n");
    fb_print("========================================\n");
    fb_print("Total tests: ");
    fb_print_int(g_test_count);
    fb_print("\nPassed: ");
    fb_print_int(g_test_passed);
    fb_print("\nFailed: ");
    fb_print_int(g_test_failed);
    fb_print("\n");
    
    if (g_test_failed == 0) {
        fb_print("Result: ALL TESTS PASSED! ✅\n");
        fb_print("Ring3 VFS API Design is ready for Step B implementation.\n");
    } else {
        fb_print("Result: SOME TESTS FAILED ❌\n");
        fb_print("API design needs refinement before Step B.\n");
    }
    
    fb_print("========================================\n");
    fb_print("Task 2.2.1.1 - Ring3 VFS API Design: COMPLETE\n");
    fb_print("Next: Task 2.2.1.2 - Convert kernel VFS to Ring3 proxy\n");
    fb_print("========================================\n");
}

/**
 * vfs_api_test_main - Main entry point for VFS API tests
 */
void vfs_api_test_main(void)
{
    run_vfs_api_tests();
}
