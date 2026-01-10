# Design Document - AykenOS Phase 1 Critical Fixes

**Author:** Kenan AY  
**Project:** AykenOS - Advanced AI-Integrated Operating System

## Overview

This design addresses the critical infrastructure gaps in AykenOS Phase 1 that must be resolved before proceeding to Phase 2 AI integration features. AykenOS represents a groundbreaking approach to operating system design, incorporating AI capabilities directly into the kernel architecture. The system currently has incomplete toolchain validation, incorrect Ring3 context switching, minimal DevFS implementation, and untested core functionality. This design provides a comprehensive solution to establish a stable foundation for AI integration.

The solution focuses on four main areas:
1. **Build System Validation** - Automated toolchain verification and QEMU testing
2. **Ring3 Context Switching** - Correct GDT selector usage and privilege separation
3. **DevFS Enhancement** - Extended device support and VFS integration
4. **Core Functionality Testing** - Comprehensive validation of kernel components

## Architecture

The architecture follows a layered approach with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│                    Build Validation Layer                   │
├─────────────────────────────────────────────────────────────┤
│  Toolchain Checker  │  QEMU Validator  │  Dependency Mgr   │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                   Kernel Core Layer                         │
├─────────────────────────────────────────────────────────────┤
│  Context Switch  │  Syscall Handler  │  Privilege Manager  │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                    Device Layer                             │
├─────────────────────────────────────────────────────────────┤
│    DevFS Core    │  Device Drivers   │   VFS Integration   │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                   Testing Framework                         │
├─────────────────────────────────────────────────────────────┤
│  Smoke Tests  │  Integration Tests  │  Validation Suite    │
└─────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### Build Validation System

**Toolchain Checker**
- Interface: `toolchain_verify() -> ValidationResult`
- Validates presence of x86_64-elf-gcc, clang, ld
- Checks version compatibility
- Reports missing dependencies with installation guidance

**QEMU Validator**
- Interface: `qemu_test_boot(kernel_path) -> BootResult`
- Automated kernel boot testing
- Timeout-based validation
- Boot log analysis for success indicators

**Dependency Manager**
- Interface: `install_guide_generate(platform) -> InstallSteps`
- Platform-specific installation instructions
- Windows/WSL environment support
- Automated setup scripts where possible

### Ring3 Context Switching System

**Context Switch Manager**
- Interface: `context_switch_fixed(old_ctx, new_ctx) -> SwitchResult`
- Corrected GDT selector constants (0x23 for user code, 0x1b for user data)
- Proper IRET frame construction
- Privilege level validation

**Syscall Handler**
- Interface: `syscall_dispatch(syscall_num, args) -> Result`
- Ring3 to Ring0 transition validation
- Parameter validation and sanitization
- Return path verification

### Enhanced DevFS System

**Device Registry**
- Interface: `devfs_register_enhanced(name, ops, metadata) -> DeviceHandle`
- Extended device metadata support
- Device capability flags
- Hot-plug/unplug support

**Input Device Support**
- Keyboard device: `/dev/kbd` with input buffering
- Serial devices: `/dev/ttyS0`, `/dev/ttyS1`
- Standard input: `/dev/stdin` as keyboard alias

**Block Device Support**
- Virtual block devices: `/dev/vda`, `/dev/vdb`
- Device size and geometry reporting
- Read/write operation validation

**VFS Integration Layer**
- Interface: `devfs_mount(mount_point) -> MountResult`
- Proper VFS mount flow integration
- Device node creation and cleanup
- Permission and access control

## Data Models

### ValidationResult Structure
```c
typedef struct {
    bool success;
    char missing_tools[256];
    char error_message[512];
    platform_info_t platform;
} validation_result_t;
```

### ContextSwitchState Structure
```c
typedef struct {
    uint64_t registers[16];  // General purpose registers
    uint64_t rip;           // Instruction pointer
    uint64_t rsp;           // Stack pointer
    uint64_t rflags;        // Flags register
    uint64_t cr3;           // Page table base
    uint16_t cs;            // Code segment (corrected values)
    uint16_t ss;            // Stack segment (corrected values)
    privilege_level_t level; // Ring 0/3 indicator
} context_state_t;
```

### DeviceMetadata Structure
```c
typedef struct {
    char name[32];
    device_type_t type;     // CHAR, BLOCK, NETWORK
    uint32_t capabilities;  // READ, WRITE, IOCTL flags
    uint64_t size;         // For block devices
    void *driver_data;
} device_metadata_t;
```

### TestResult Structure
```c
typedef struct {
    bool passed;
    char test_name[64];
    char failure_reason[256];
    uint64_t execution_time_ms;
} test_result_t;
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

**Property 1: Toolchain Detection Accuracy**
*For any* required toolchain component, the detection script should correctly identify its presence and version compatibility
**Validates: Requirements 1.1, 1.2**

**Property 2: QEMU Boot Validation**
*For any* valid kernel build, QEMU boot testing should correctly detect successful initialization through log analysis
**Validates: Requirements 1.3, 4.1**

**Property 3: Context Switch Selector Consistency**
*For any* context switch operation, the system should use GDT selector constants that match the defined values (0x23/0x1b)
**Validates: Requirements 2.1, 2.2**

**Property 4: DevFS Stub Device Registration**
*For any* registered stub device (/dev/kbd, /dev/ttyS0, /dev/sda), the device should be accessible through standard DevFS operations
**Validates: Requirements 3.1, 3.2, 3.3, 3.5**

**Property 5: VFS-DevFS Integration Path**
*For any* DevFS mount operation, the integration with VFS should follow the documented interface specification
**Validates: Requirements 3.4**

## Error Handling

### Build System Errors
- **Missing Toolchain**: Provide specific installation instructions for detected platform
- **QEMU Boot Failure**: Capture boot logs and provide debugging guidance
- **Version Incompatibility**: Report required vs. found versions with upgrade paths

### Context Switch Errors
- **Invalid Privilege Transition**: Log attempt and maintain system security
- **Corrupted Context**: Detect and recover from invalid context states
- **GDT Inconsistency**: Validate selectors before use, fail safely

### DevFS Errors
- **Device Registration Failure**: Log error and continue with available devices
- **Mount Failure**: Provide fallback device access methods
- **Device Operation Failure**: Return appropriate error codes to applications

### Testing Errors
- **Test Framework Failure**: Continue with manual validation procedures
- **Timeout Errors**: Provide extended timeout options for slow systems
- **Resource Exhaustion**: Clean up and retry with reduced resource usage

## Testing Strategy

This design employs a **script-based testing approach** suitable for kernel development, focusing on automated validation through external tools rather than embedded test frameworks.

### Script-Based Testing Approach
**PowerShell/Bash Automation:**
- Toolchain detection scripts with dependency checking
- QEMU boot validation with log parsing and timeout handling
- Automated build verification (make all/make run)
- Boot success detection through QEMU monitor interface

**QEMU Integration Testing:**
- Automated kernel boot with success/failure detection
- Ring3 user process execution validation
- Syscall roundtrip testing through QEMU scripting
- DevFS device operation verification

### Rust-Side Testing (ayken-core)
Property-based and unit tests will be implemented in the Rust components where appropriate:
- ABDF/BCIB format validation
- Data structure serialization/deserialization
- Core library functionality

### Kernel Validation Scripts
**Boot Validation:**
- QEMU startup with timeout monitoring
- Boot log analysis for success indicators
- Kernel panic detection and reporting

**Functional Testing:**
- Ring3 context switch validation through QEMU debugging
- DevFS mount and device access verification
- Syscall interface testing via user-mode test programs

**Integration Reporting:**
- Automated test result compilation
- Build status and dependency validation reports
- Windows/WSL environment setup verification

This approach provides practical validation suitable for kernel development while avoiding the complexity of embedded test frameworks in the kernel environment.