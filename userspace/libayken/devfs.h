// userspace/libayken/devfs.h - Ring3 device proxy
// AykenOS Phase 2.2 - Ring3 Device Proxy API Design
//
// This header defines the Ring3 device proxy interface that provides secure,
// capability-based device access. All device policy and management logic
// operates in Ring3 userspace, while Ring0 provides only the mechanism
// (capability validation, memory mapping).
//
// Requirements:
// - FR-2.3.1: Device access must use capability tokens
// - FR-2.3.2: Device policy must execute entirely in Ring3
// - FR-2.3.3: Ring0 must provide only device access mechanism
// - FR-2.3.4: Device operations must be secure and isolated

#ifndef __USERSPACE_LIBAYKEN_DEVFS_H
#define __USERSPACE_LIBAYKEN_DEVFS_H

#include <stddef.h>
#include <stdint.h>

// Forward declaration of capability token (defined in kernel headers)
typedef struct capability_token capability_token_t;

// ============================================================================
// DEVICE TYPES AND CAPABILITIES
// ============================================================================

/**
 * @brief Device types for classification and capability management
 */
typedef enum {
    DEVICE_TYPE_CHARACTER = 0,  /**< Character devices (keyboard, serial, console) */
    DEVICE_TYPE_BLOCK = 1,      /**< Block devices (disk, partition) */
    DEVICE_TYPE_NETWORK = 2,    /**< Network devices (ethernet, wifi) */
    DEVICE_TYPE_SPECIAL = 3,    /**< Special devices (null, zero, random) */
    DEVICE_TYPE_GPU = 4,        /**< GPU compute devices */
    DEVICE_TYPE_AUDIO = 5,      /**< Audio devices (speakers, microphone) */
    DEVICE_TYPE_SENSOR = 6,     /**< Sensor devices (temperature, accelerometer) */
    DEVICE_TYPE_CUSTOM = 7      /**< Custom/experimental devices */
} device_type_t;

/**
 * @brief Device capability flags for fine-grained access control
 */
#define DEVICE_CAP_READ         (1 << 0)    /**< Read access permission */
#define DEVICE_CAP_WRITE        (1 << 1)    /**< Write access permission */
#define DEVICE_CAP_IOCTL        (1 << 2)    /**< Control operations permission */
#define DEVICE_CAP_SEEK         (1 << 3)    /**< Seek operations (block devices) */
#define DEVICE_CAP_MMAP         (1 << 4)    /**< Memory mapping permission */
#define DEVICE_CAP_EXCLUSIVE    (1 << 5)    /**< Exclusive access permission */
#define DEVICE_CAP_ADMIN        (1 << 6)    /**< Administrative operations */
#define DEVICE_CAP_DEBUG        (1 << 7)    /**< Debug/diagnostic access */

/**
 * @brief Device status flags
 */
#define DEVICE_STATUS_ONLINE    (1 << 0)    /**< Device is online and available */
#define DEVICE_STATUS_BUSY      (1 << 1)    /**< Device is currently busy */
#define DEVICE_STATUS_ERROR     (1 << 2)    /**< Device has an error condition */
#define DEVICE_STATUS_READONLY  (1 << 3)    /**< Device is in read-only mode */

// ============================================================================
// DEVICE METADATA AND INFORMATION
// ============================================================================

/**
 * @brief Device metadata structure
 */
typedef struct device_metadata {
    device_type_t type;             /**< Device type classification */
    uint32_t capabilities;          /**< Supported capability flags */
    uint32_t status;                /**< Current device status flags */
    uint64_t size;                  /**< Device size (for block devices) */
    uint32_t block_size;            /**< Block size (for block devices) */
    char name[64];                  /**< Device name (e.g., "console", "disk0") */
    char description[128];          /**< Human-readable description */
    char driver[32];                /**< Driver name */
    uint32_t major_version;         /**< Driver major version */
    uint32_t minor_version;         /**< Driver minor version */
} device_metadata_t;

/**
 * @brief Device handle for Ring3 operations
 */
typedef struct device_handle {
    uint64_t device_id;             /**< Unique device identifier */
    capability_token_t *capability; /**< Associated capability token */
    device_metadata_t metadata;     /**< Device metadata */
    uint32_t access_flags;          /**< Current access permissions */
    void *private_data;             /**< Private data for device proxy */
} device_handle_t;

// ============================================================================
// RING3 DEVICE PROXY INTERFACE
// ============================================================================

/**
 * @brief Ring3 Device Proxy Structure
 * 
 * This structure defines the function pointers for device operations that
 * execute entirely in Ring3 userspace. The proxy handles device policy,
 * capability validation, and coordinates with Ring0 for mechanism operations.
 */
typedef struct device_proxy {
    /**
     * @brief Open a device with capability-based access control
     * 
     * Opens a device for access using capability tokens. The proxy validates
     * the capability and establishes a secure connection to the device.
     * 
     * @param device_path Path to the device (e.g., "/dev/console")
     * @param access_flags Requested access flags (DEVICE_CAP_*)
     * @param capability Capability token for device access
     * @return Device handle on success, NULL on failure
     */
    device_handle_t* (*device_open)(const char *device_path, 
                                   uint32_t access_flags,
                                   capability_token_t *capability);
    
    /**
     * @brief Read data from a device
     * 
     * Reads data from the specified device using capability-based security.
     * The operation is performed entirely in Ring3 with Ring0 providing
     * only the memory mapping mechanism.
     * 
     * @param handle Device handle from device_open
     * @param buffer Buffer to store read data
     * @param count Number of bytes to read
     * @return Number of bytes read on success, negative error code on failure
     */
    int (*device_read)(device_handle_t *handle, void *buffer, size_t count);
    
    /**
     * @brief Write data to a device
     * 
     * Writes data to the specified device using capability-based security.
     * The operation is performed entirely in Ring3 with Ring0 providing
     * only the memory mapping mechanism.
     * 
     * @param handle Device handle from device_open
     * @param buffer Buffer containing data to write
     * @param count Number of bytes to write
     * @return Number of bytes written on success, negative error code on failure
     */
    int (*device_write)(device_handle_t *handle, const void *buffer, size_t count);
    
    /**
     * @brief Perform device control operations
     * 
     * Executes device-specific control operations (ioctl) using capability
     * tokens to ensure secure access to device configuration and control.
     * 
     * @param handle Device handle from device_open
     * @param command Control command identifier
     * @param arg Command-specific argument
     * @return 0 on success, negative error code on failure
     */
    int (*device_ioctl)(device_handle_t *handle, uint32_t command, void *arg);
    
    /**
     * @brief Seek to a position in a device (block devices)
     * 
     * Changes the current position in a block device for subsequent read/write
     * operations. Only available for devices with DEVICE_CAP_SEEK capability.
     * 
     * @param handle Device handle from device_open
     * @param offset Offset to seek to
     * @param whence Seek origin (SEEK_SET, SEEK_CUR, SEEK_END)
     * @return New position on success, negative error code on failure
     */
    int64_t (*device_seek)(device_handle_t *handle, int64_t offset, int whence);
    
    /**
     * @brief Memory map a device region
     * 
     * Maps a device memory region into the process address space using
     * capability tokens and Ring0 memory mapping mechanism.
     * 
     * @param handle Device handle from device_open
     * @param offset Offset within device to map
     * @param length Length of region to map
     * @param flags Mapping flags (read/write/execute)
     * @return Mapped address on success, NULL on failure
     */
    void* (*device_mmap)(device_handle_t *handle, uint64_t offset, 
                        size_t length, uint32_t flags);
    
    /**
     * @brief Unmap a device memory region
     * 
     * Unmaps a previously mapped device memory region and revokes
     * associated capability tokens.
     * 
     * @param handle Device handle from device_open
     * @param addr Address returned by device_mmap
     * @param length Length of region to unmap
     * @return 0 on success, negative error code on failure
     */
    int (*device_munmap)(device_handle_t *handle, void *addr, size_t length);
    
    /**
     * @brief Close a device handle
     * 
     * Closes the device handle and revokes associated capability tokens.
     * All resources associated with the handle are cleaned up.
     * 
     * @param handle Device handle to close
     * @return 0 on success, negative error code on failure
     */
    int (*device_close)(device_handle_t *handle);
    
    /**
     * @brief Get device metadata
     * 
     * Retrieves metadata information about the device, including capabilities,
     * status, and driver information.
     * 
     * @param handle Device handle from device_open
     * @param metadata Pointer to metadata structure to fill
     * @return 0 on success, negative error code on failure
     */
    int (*device_get_metadata)(device_handle_t *handle, device_metadata_t *metadata);
    
    /**
     * @brief Enumerate available devices
     * 
     * Lists all devices accessible with the current capability set.
     * Only devices for which the caller has appropriate capabilities are listed.
     * 
     * @param device_list Array to store device information
     * @param max_devices Maximum number of devices to list
     * @param capability Capability token for device enumeration
     * @return Number of devices listed, negative error code on failure
     */
    int (*device_enumerate)(device_metadata_t *device_list, size_t max_devices,
                           capability_token_t *capability);
} device_proxy_t;

// ============================================================================
// DEVICE PROXY MANAGEMENT FUNCTIONS
// ============================================================================

/**
 * @brief Initialize the Ring3 device proxy system
 * 
 * Initializes the device proxy system and establishes communication
 * with Ring0 device mechanisms.
 * 
 * @return 0 on success, negative error code on failure
 */
int device_proxy_init(void);

/**
 * @brief Register a device proxy implementation
 * 
 * Registers a device proxy implementation for a specific device type.
 * Multiple proxies can be registered for different device types.
 * 
 * @param device_type Device type this proxy handles
 * @param proxy Pointer to device proxy implementation
 * @return 0 on success, negative error code on failure
 */
int device_proxy_register(device_type_t device_type, const device_proxy_t *proxy);

/**
 * @brief Unregister a device proxy implementation
 * 
 * Unregisters a previously registered device proxy implementation.
 * 
 * @param device_type Device type to unregister
 * @return 0 on success, negative error code on failure
 */
int device_proxy_unregister(device_type_t device_type);

/**
 * @brief Get device proxy for a device type
 * 
 * Retrieves the registered device proxy for a specific device type.
 * 
 * @param device_type Device type to get proxy for
 * @return Pointer to device proxy, NULL if not registered
 */
const device_proxy_t* device_proxy_get(device_type_t device_type);

/**
 * @brief Request device capability token
 * 
 * Requests a capability token for accessing a specific device with
 * specified permissions. This interfaces with Ring0 capability system.
 * 
 * @param device_path Path to the device
 * @param access_flags Requested access permissions
 * @return Capability token on success, NULL on failure
 */
capability_token_t* device_request_capability(const char *device_path, 
                                             uint32_t access_flags);

/**
 * @brief Release device capability token
 * 
 * Releases a previously acquired capability token and revokes
 * associated device access permissions.
 * 
 * @param capability Capability token to release
 * @return 0 on success, negative error code on failure
 */
int device_release_capability(capability_token_t *capability);

// ============================================================================
// CONVENIENCE FUNCTIONS
// ============================================================================

/**
 * @brief Open device with automatic capability acquisition
 * 
 * Convenience function that automatically requests appropriate capability
 * tokens and opens the device.
 * 
 * @param device_path Path to the device
 * @param access_flags Requested access permissions
 * @return Device handle on success, NULL on failure
 */
device_handle_t* device_open_auto(const char *device_path, uint32_t access_flags);

/**
 * @brief Check if device path is valid
 * 
 * Validates that a device path refers to an accessible device.
 * 
 * @param device_path Path to validate
 * @return 1 if valid, 0 if invalid
 */
int device_path_is_valid(const char *device_path);

/**
 * @brief Get device type from path
 * 
 * Determines the device type from a device path.
 * 
 * @param device_path Path to analyze
 * @return Device type, or DEVICE_TYPE_CUSTOM if unknown
 */
device_type_t device_get_type_from_path(const char *device_path);

// ============================================================================
// ERROR CODES
// ============================================================================

#define DEVICE_SUCCESS              0   /**< Operation successful */
#define DEVICE_ERROR_INVALID_PATH   -1  /**< Invalid device path */
#define DEVICE_ERROR_NO_CAPABILITY  -2  /**< Missing required capability */
#define DEVICE_ERROR_ACCESS_DENIED  -3  /**< Access denied */
#define DEVICE_ERROR_DEVICE_BUSY    -4  /**< Device is busy */
#define DEVICE_ERROR_DEVICE_ERROR   -5  /**< Device hardware error */
#define DEVICE_ERROR_NOT_SUPPORTED  -6  /**< Operation not supported */
#define DEVICE_ERROR_INVALID_HANDLE -7  /**< Invalid device handle */
#define DEVICE_ERROR_BUFFER_TOO_SMALL -8 /**< Buffer too small */
#define DEVICE_ERROR_TIMEOUT        -9  /**< Operation timed out */
#define DEVICE_ERROR_NO_MEMORY      -10 /**< Out of memory */

// ============================================================================
// STANDARD DEVICE PATHS
// ============================================================================

#define DEVICE_PATH_CONSOLE     "/dev/console"      /**< System console */
#define DEVICE_PATH_NULL        "/dev/null"         /**< Null device */
#define DEVICE_PATH_ZERO        "/dev/zero"         /**< Zero device */
#define DEVICE_PATH_RANDOM      "/dev/random"       /**< Random device */
#define DEVICE_PATH_KEYBOARD    "/dev/keyboard"     /**< Keyboard input */
#define DEVICE_PATH_MOUSE       "/dev/mouse"        /**< Mouse input */
#define DEVICE_PATH_DISK0       "/dev/disk0"        /**< Primary disk */
#define DEVICE_PATH_GPU0        "/dev/gpu0"         /**< Primary GPU */

// ============================================================================
// INTEGRATION WITH V2 SYSCALLS
// ============================================================================

/**
 * @brief Device proxy syscall interface
 * 
 * These functions interface with the Ring0 v2 syscall system to provide
 * the mechanism for device operations while maintaining Ring3 policy.
 */

/**
 * @brief Map device memory using sys_v2_map_memory
 * 
 * Uses the v2 memory mapping syscall to map device memory regions
 * with appropriate capability validation.
 * 
 * @param device_id Device identifier
 * @param offset Offset within device
 * @param size Size to map
 * @param capability Capability token
 * @return Mapped address on success, NULL on failure
 */
void* device_syscall_map_memory(uint64_t device_id, uint64_t offset, 
                               size_t size, capability_token_t *capability);

/**
 * @brief Unmap device memory using sys_v2_unmap_memory
 * 
 * Uses the v2 memory unmapping syscall to unmap device memory regions.
 * 
 * @param addr Address to unmap
 * @param size Size to unmap
 * @return 0 on success, negative error code on failure
 */
int device_syscall_unmap_memory(void *addr, size_t size);

/**
 * @brief Bind device capability using sys_v2_capability_bind
 * 
 * Uses the v2 capability binding syscall to bind device access capabilities
 * to the current execution context.
 * 
 * @param capability Capability token to bind
 * @return Capability ID on success, negative error code on failure
 */
uint64_t device_syscall_bind_capability(capability_token_t *capability);

/**
 * @brief Revoke device capability using sys_v2_capability_revoke
 * 
 * Uses the v2 capability revocation syscall to revoke device access
 * capabilities.
 * 
 * @param capability_id Capability ID to revoke
 * @return 0 on success, negative error code on failure
 */
int device_syscall_revoke_capability(uint64_t capability_id);

#endif // __USERSPACE_LIBAYKEN_DEVFS_H
