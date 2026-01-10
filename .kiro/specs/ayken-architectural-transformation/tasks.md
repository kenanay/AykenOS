# AykenOS Architectural Transformation - Implementation Tasks

**Author:** Kenan AY  
**Project:** AykenOS - Advanced AI-Integrated Operating System  
**Created:** January 3, 2026

## 🎯 Phase 1.5 - Stabilization and Completion (Priority 1)

**Objective:** Complete Phase 1 properly before any architectural changes
**Timeline:** 1-2 weeks
**Status:** ✅ COMPLETED - Ready for Phase 2.1

### 🚨 CRITICAL SCOPE BOUNDARIES (Phase 1.5)
- **NO v2 syscall code** will be written in this phase
- **NO execution-centric syscall interface** development
- **ONLY existing POSIX-like (v1) syscall set** testing and validation
- **ONLY round-trip functionality** of current read/write/open/close/exit syscalls
- **BLOCKING NATURE:** Phase 2 cannot begin until Phase 1.5 is 100% complete

### Task 1.5.1: Toolchain Setup and Validation
- [x] **1.5.1.1** Execute Windows toolchain setup





  - Run `./tools/setup/setup_windows_dev.ps1 -AutoInstall`
  - Verify x86_64-elf-gcc, clang, linker installation
  - Document any installation issues and fixes
  - _Requirements: Complete toolchain validation_

- [x] **1.5.1.2** Execute cross-platform validation





  - Run `./tools/validation/validate_toolchain.ps1 --verbose`
  - Ensure all build dependencies are met
  - Generate toolchain validation report
  - _Requirements: 100% toolchain validation success_

- [x] **1.5.1.3** QEMU environment validation





  - Validate QEMU installation and boot capability
  - Test `make run` automation with success/failure detection
  - Ensure QEMU log parsing works correctly
  - _Requirements: Reliable QEMU boot validation_

### Task 1.5.2: Ring3 Round-Trip Validation
- [x] **1.5.2.1** Create Ring3 test process

  ```c
  // Test: Ring3 user process + syscall round-trip
  // Target: 100% stable execution in QEMU
  proc_t *test_user = proc_create_user_process("ring3-test", 
                                               test_code, size, 
                                               PROC_IMAGE_FLAT);
  ```
  - _Requirements: Stable Ring3 process creation_

- [x] **1.5.2.2** Implement syscall round-trip test






  ```c
  // Syscall test: int 0x80 → kernel → return to Ring3
  // Test all current syscalls: read/write/open/close/exit
  ```
  - Validate INT 0x80 mechanism works reliably
  - Test syscall parameter passing and return values
  - Ensure Ring3→Ring0→Ring3 transitions are stable
  - _Requirements: 100% syscall round-trip success_

- [x] **1.5.2.3** QEMU integration testing





  - Create automated Ring3 validation script
  - Test user process execution through QEMU automation
  - Generate comprehensive test reports
  - _Requirements: Automated validation pipeline_

### Task 1.5.3: Code Cleanup and Consistency
- [x] **1.5.3.1** Remove unused switch_to_user_mode function




  - Analyze usage of `switch_to_user_mode` helper
  - Remove if unused or fix selector constants (CS=0x23, SS=0x1B)
  - Ensure scheduler path is the single source of Ring3 transitions
  - _Requirements: Clean, consistent Ring3 transition code_

- [x] **1.5.3.2** Verify GDT constant consistency





  - Confirm GDT selector constants across codebase (0x23/0x1B)
  - Ensure assembly and C header definitions match
  - Eliminate any build warnings related to constants
  - _Requirements: Zero build warnings, consistent constants_

- [x] **1.5.3.3** Update documentation consistency





  - Align PROJECT_STATUS_REPORT with current state
  - Update README to reflect "Phase 1 complete, testing validated"
  - Document Phase 1.5 completion status
  - _Requirements: Accurate documentation_

### Task 1.5.4: Phase 1.5 Validation and Sign-off
- [x] **1.5.4.1** Execute complete validation suite




  - Run all toolchain validation scripts
  - Execute Ring3 round-trip tests
  - Generate comprehensive Phase 1.5 completion report
  - _Requirements: All tests passing_

- [x] **1.5.4.2** Phase 1.5 completion documentation
  - Document all completed Phase 1 objectives
  - Verify alignment with Phase 1 documentation requirements
  - Create Phase 1.5 sign-off report
  - _Requirements: Phase 1 officially complete_

---

## 🚀 Phase 2.1 - Ring0 Syscall Redesign (Priority 2)

**Objective:** Implement execution-centric syscall interface
**Timeline:** 2-3 weeks
**Status:** 🚀 READY TO START (Phase 1.5 Complete)

### Task 2.1.1: New Syscall Interface Design
- [x] **2.1.1.1** Define execution-centric syscall interface
  ```c
  // kernel/sys/syscall_v2.h - New interface
  #define SYS_V2_MAP_MEMORY        0
  #define SYS_V2_UNMAP_MEMORY      1  
  #define SYS_V2_SWITCH_CONTEXT    2
  #define SYS_V2_SUBMIT_EXECUTION  3
  #define SYS_V2_WAIT_RESULT       4
  #define SYS_V2_INTERRUPT_RETURN  5
  #define SYS_V2_TIME_QUERY        6
  #define SYS_V2_CAPABILITY_BIND   7
  #define SYS_V2_CAPABILITY_REVOKE 8
  #define SYS_V2_EXIT              9
  ```
  - _Requirements: 10 syscalls exactly as per Phase 2 documentation_

- [x] **2.1.1.2** Implement syscall function signatures
  ```c
  // Implementation prototypes
  uint64_t sys_v2_map_memory(uint64_t virt, uint64_t phys, uint64_t flags);
  uint64_t sys_v2_submit_execution(void *bcib_graph, uint64_t size);
  uint64_t sys_v2_wait_result(uint64_t execution_id, uint64_t timeout);
  ```
  - _Requirements: Complete function signatures for all 10 syscalls_

### Task 2.1.2: Capability System Implementation
- [x] **2.1.2.1** Design capability token system
  ```c
  // kernel/include/capability.h - New system
  typedef struct capability_token {
      uint64_t id;
      uint32_t permissions;
      uint32_t resource_type;
  } capability_token_t;
  ```
  - _Requirements: Capability system design_

- [x] **2.1.2.2** Implement capability syscalls
  ```c
  uint64_t sys_v2_capability_bind(uint64_t execution_ctx, capability_token_t *token);
  uint64_t sys_v2_capability_revoke(uint64_t token_id);
  ```
  - _Requirements: Working capability bind/revoke mechanism_

### Task 2.1.3: Dual Syscall Support (Transition Period)
- [x] **2.1.3.1** Implement hybrid syscall dispatcher with number plan



  
  **Syscall Numbering Plan:**
  - **0-99 range:** Reserved for legacy POSIX-like (v1) syscalls
  - **1000-1009 range:** New execution-centric (v2) syscalls
  - **Deprecation Plan:** v1 syscalls will be completely removed in Phase 2.5
  
  ```c
  // kernel/sys/syscall.c - Hybrid approach with clear numbering
  uint64_t syscall_handler(uint64_t syscall_num, uint64_t arg1, ...) {
      // Route based on Syscall Numbering Plan
      if (syscall_num >= 1000 && syscall_num <= 1009) {
          // New execution-centric syscalls (v2)
          return syscall_v2_handler(syscall_num - 1000, arg1, ...);
      } else if (syscall_num >= 0 && syscall_num <= 99) {
          // Legacy POSIX-like syscalls (v1 - backward compatibility)
          return syscall_v1_handler(syscall_num, arg1, ...);
      } else {
          // Invalid syscall number
          return -ENOSYS; 
      }
  }
  ```
  - _Requirements: Backward compatibility maintained, clear number plan_

- [x] **2.1.3.2** Create syscall transition documentation



  - Document migration path from v1 to v2 syscalls
  - Provide examples for both interfaces
  - Create developer migration guide
  - _Requirements: Clear migration documentation_

---

## 🚀 Phase 2.2 - Ring3 Runtime Development (Priority 3)

**Objective:** Move VFS, DevFS, Scheduler policy to Ring3
**Timeline:** 3-4 weeks  
**Status:** Blocked until Phase 2.1 complete

### Task 2.2.1: Ring3 VFS Library
- [x] **2.2.1.1** Design Ring3 VFS interface (Step A: API Design)



  ```c
  // userspace/libayken/vfs.h - Ring3 VFS library
  typedef struct userspace_vfs {
      int (*open)(const char *path, int flags);
      int (*read)(int fd, void *buf, size_t count);
      int (*write)(int fd, const void *buf, size_t count);
      int (*close)(int fd);
  } userspace_vfs_t;
  ```
  - _Requirements: Ring3 VFS API design_

- [x] **2.2.1.2** Convert kernel VFS to Ring3 proxy (Step B: Kernel Stub Conversion)



  ```c
  // Convert kernel/fs/vfs.c functions to stubs that call Ring3 library
  // Remove internal VFS logic from kernel
  // Make vfs_read, vfs_open etc. call Ring3 VFS library functions
  int vfs_read(vfs_file_t *file, void *buffer, uint64_t size) {
      // Stub: redirect to Ring3 VFS library
      return userspace_vfs_read(file, buffer, size);
  }
  ```
  - _Requirements: Kernel VFS becomes proxy to Ring3 implementation_

- [x] **2.2.1.3** Implement Ring3 VFS using new syscalls (Step C: Full Implementation)



  ```c
  // Implementation using sys_v2_map_memory for file access
  int userspace_open(const char *path, int flags) {
      // Map file via sys_v2_map_memory
      // Return userspace file descriptor
  }
  ```
  - _Requirements: VFS operations via Ring0 mechanism only_
  - _Note: Step C (kernel stub removal) will be completed in Phase 2.5_

### Task 2.2.2: Ring3 Scheduler Policy
- [x] **2.2.2.1** Design Ring3 scheduler policy interface (Step A: API Design)


  ```c
  // userspace/libayken/scheduler.h - Ring3 scheduler policy
  typedef struct scheduler_policy {
      proc_t* (*select_next)(proc_t *ready_queue);
      void (*enqueue_ready)(proc_t *proc);
      void (*handle_block)(proc_t *proc, void *wait_obj);
  } scheduler_policy_t;
  ```
  - _Requirements: Policy in Ring3, mechanism in Ring0_

- [x] **2.2.2.2** Convert kernel scheduler to Ring3 proxy (Step B: Kernel Stub Conversion)


  ```c
  // Convert kernel/sched/sched.c policy functions to stubs
  // Remove scheduling policy logic from kernel
  // Make scheduler policy decisions call Ring3 library
  proc_t* sched_select_next(void) {
      // Stub: redirect to Ring3 scheduler policy
      return userspace_scheduler_select_next();
  }
  ```
  - _Requirements: Kernel scheduler becomes proxy to Ring3 policy_

- [x] **2.2.2.3** Implement Ring0 mechanism-only scheduler (Step C: Full Implementation)


  ```c
  // Ring0 provides only mechanism
  uint64_t sys_v2_switch_context(uint64_t old_ctx_id, uint64_t new_ctx_id);
  ```
  - _Requirements: Ring0 scheduler contains no policy decisions_
  - _Note: Step C (kernel stub removal) will be completed in Phase 2.5_

### Task 2.2.3: Ring3 DevFS Proxy
- [x] **2.2.3.1** Design Ring3 device proxy (Step A: API Design)



  ```c
  // userspace/libayken/devfs.h - Ring3 device proxy
  typedef struct device_proxy {
      int (*device_read)(const char *device_path, void *buf, size_t count);
      int (*device_write)(const char *device_path, const void *buf, size_t count);
  } device_proxy_t;
  ```
  - _Requirements: Device access via capability tokens_

- [x] **2.2.3.2** Convert kernel DevFS to Ring3 proxy (Step B: Kernel Stub Conversion)









  ```c
  // Convert kernel/fs/devfs.c functions to stubs
  // Remove device management logic from kernel
  // Make devfs operations call Ring3 device proxy
  int devfs_read(const char *device_path, void *buf, size_t count) {
      // Stub: redirect to Ring3 device proxy
      return userspace_device_read(device_path, buf, count);
  }
  ```
  - _Requirements: Kernel DevFS becomes proxy to Ring3 implementation_

- [x] **2.2.3.3** Implement capability-based device access (Step C: Full Implementation)



  ```c
  // Implementation using capability tokens
  int device_read(const char *device_path, void *buf, size_t count) {
      capability_token_t token = get_device_capability(device_path);
      return sys_v2_capability_bind(current_execution_ctx(), &token);
  }
  ```
  - _Requirements: Secure device access via capabilities_
  - _Note: Step C (kernel stub removal) will be completed in Phase 2.5_

---

## 🚀 Phase 2.3 - BCIB Execution Engine (Priority 4)

**Objective:** Implement Ring3 BCIB runtime
**Timeline:** 2-3 weeks
**Status:** Blocked until Phase 2.2 complete

### Task 2.3.1: BCIB Executor in Ring3
- [x] **2.3.1.1** Design BCIB executor architecture





  ```rust
  // userspace/bcib-runtime/src/executor.rs
  pub struct BcibExecutor {
      execution_contexts: HashMap<u64, ExecutionContext>,
      capability_manager: CapabilityManager,
  }
  ```
  - _Requirements: Ring3 BCIB execution engine_

- [x] **2.3.1.2** Implement execution submission





  ```rust
  impl BcibExecutor {
      pub fn submit_execution(&mut self, graph: &BcibGraph) -> Result<u64, ExecutionError> {
          let execution_id = self.allocate_execution_id();
          
          // Submit to Ring0 (mechanism only)
          unsafe {
              syscall_v2(SYS_V2_SUBMIT_EXECUTION,
                         graph.as_ptr() as u64,
                         graph.len() as u64,
                         execution_id, 0)
          }
      }
  }
  ```
  - _Requirements: BCIB graph submission via syscalls_

### Task 2.3.2: DSL Parser Implementation
- [x] **2.3.2.1** Implement hierarchical DSL parser





  ```rust
  // userspace/dsl-parser/src/parser.rs - Per Phase 2 documentation
  pub struct DslParser {
      context: ExecutionContext,
  }
  
  impl DslParser {
      pub fn parse_command(&mut self, input: &str) -> Result<DispatchRequest, ParseError> {
          match input {
              cmd if cmd.starts_with(">>") => self.parse_context_command(cmd),
              cmd if cmd.starts_with(">") => self.parse_simple_command(cmd),
              cmd if cmd.starts_with(">[ ]") => self.parse_batch_command(cmd),
              _ => Err(ParseError::InvalidSyntax)
          }
      }
  }
  ```
  - _Requirements: DSL grammar per Phase 2 documentation_

---

## 🚀 Phase 2.4 - AI Runtime Migration (Priority 5)

**Objective:** Move AI inference to Ring3
**Timeline:** 2-3 weeks
**Status:** Blocked until Phase 2.3 complete

### Task 2.4.1: AI Runtime Extraction
- [x] **2.4.1.1** Design Ring3 AI runtime interface (Step A: API Design)




  ```c
  // userspace/ai-runtime/lm_runtime.h - Ring3 AI runtime interface
  typedef struct ai_runtime {
      lm_model_t *model;
      float *workspace;
      capability_token_t gpu_access;
  } ai_runtime_t;
  ```
  - _Requirements: Ring3 AI runtime API design_

- [x] **2.4.1.2** Convert kernel AI runtime to Ring3 proxy (Step B: Kernel Stub Conversion)





  ```c
  // Convert kernel/ai/lm_runtime.c functions to stubs
  // Remove AI inference logic from kernel
  // Make AI functions call Ring3 AI runtime
  int lm_infer(const char *prompt, char *out, int max_out) {
      // Stub: redirect to Ring3 AI runtime
      return userspace_ai_infer(prompt, out, max_out);
  }
  ```
  - _Requirements: Kernel AI runtime becomes proxy to Ring3 implementation_

- [x] **2.4.1.3** Implement capability-based AI access (Step C: Full Implementation)




  ```c
  // userspace/ai-runtime/lm_runtime.c - Moved from kernel/ai/
  // All AI inference logic operates in Ring3
  int ai_runtime_init(ai_runtime_t *runtime) {
      // Get GPU capability token
      runtime->gpu_access = request_gpu_capability();
      
      // Memory map model weights
      sys_v2_map_memory(MODEL_VIRT_ADDR, model_phys_addr, MAP_READ_ONLY);
      
      return 0;
  }
  ```
  - _Requirements: Complete AI runtime in Ring3 with capability system_
  - _Note: Step C (kernel stub removal) will be completed in Phase 2.5_

### Task 2.4.2: AI Stub Implementation
- [x] **2.4.2.1** Implement AI stub per Phase 2 documentation




  ```rust
  // userspace/ai-runtime/src/ai_stub.rs - Per Phase 2 documentation
  pub struct AiStub {
      logger: Logger,
  }
  
  impl AiStub {
      pub fn ask(&self, prompt: &str) -> Result<String, AiError> {
          // Phase 2: log only
          self.logger.log(&format!("AI query: {}", prompt));
          Ok("AI response placeholder".to_string())
      }
  }
  ```
  - _Requirements: AI stub implementation per documentation_

---

## 🎯 Phase 2.5 - Legacy Cleanup (Priority 6)

**Objective:** Remove legacy POSIX syscalls and Ring0 policy code
**Timeline:** 1 week
**Status:** Blocked until Phase 2.4 complete

### Task 2.5.1: Legacy Syscall Removal
- [x] **2.5.1.1** Remove POSIX syscalls (Complete Step C for all components)










  - Remove sys_read, sys_write, sys_open, sys_close from Ring0
  - Remove syscall number range 0-99 support from dispatcher
  - Update syscall dispatcher to use only v2 interface (1000-1009)
  - Remove backward compatibility code
  - _Requirements: Only 10 execution-centric syscalls remain_

### Task 2.5.2: Ring0 Policy Code Removal  
- [x] **2.5.2.1** Remove VFS/DevFS stubs from Ring0 (Complete Step C)





  - Remove kernel/fs/vfs.c stub functions
  - Remove kernel/fs/devfs.c stub functions
  - Keep only memory mapping mechanism in Ring0
  - _Requirements: No file system policy or stubs in Ring0_

- [x] **2.5.2.2** Remove AI runtime stubs from Ring0 (Complete Step C)





  - Remove kernel/ai/lm_runtime.c stub functions
  - Remove all AI inference code and stubs from Ring0
  - _Requirements: No AI code or stubs in Ring0_


- [x] **2.5.2.3** Remove scheduler policy stubs from Ring0 (Complete Step C)




  - Remove policy decision stubs from kernel/sched/sched.c
  - Keep only context switch mechanism
  - _Requirements: No scheduling policy or stubs in Ring0_

### Task 2.5.3: Final Validation
- [x] **2.5.3.1** Execute complete Phase 2 validation





  - Validate all 10 syscalls work correctly
  - Test Ring3 VFS/DevFS/AI runtime
  - Test BCIB execution engine
  - Verify capability system functionality
  - _Requirements: Complete Phase 2 validation_

- [x] **2.5.3.2** Generate Phase 2 completion report




  - Document architectural transformation completion
  - Verify alignment with Phase 2 documentation
  - Create final validation report
  - _Requirements: Phase 2 officially complete_

---

## 🏆 Success Validation Checklist

### Phase 1.5 Completion ✅
- [x] Ring3 user process 100% stable in QEMU
- [x] Syscall round-trip validated and documented  
- [x] Toolchain setup completed and automated
- [x] All build warnings eliminated
- [x] GDT constants consistent across codebase

### Phase 2 Completion ✅
- [x] Ring0 contains exactly 10 syscalls (no more, no less)





- [x] VFS operations work entirely in Ring3








- [x] DevFS operations work entirely in Ring3  



- [x] Scheduler policy operates entirely in Ring3



- [x] AI runtime operates entirely in Ring3
- [x] BCIB execution engine works in Ring3
- [x] Capability system enforces security
- [x] DSL parser handles hierarchical commands
- [x] Legacy POSIX syscalls completely removed
- [x] No policy code remains in Ring0

## Critical Success Factors

1. **Phase Completion**: Never proceed to next phase until current phase is 100% complete
2. **Documentation Adherence**: Follow Phase 1 and Phase 2 documentation strictly
3. **Backward Compatibility**: Maintain working system during transition
4. **Validation**: Test each component thoroughly before integration
5. **Rollback Capability**: Keep working Phase 1 implementation as safety net