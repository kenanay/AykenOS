# Phase 2.3 Progress Update (BCIB Runtime & Syscall Mechanism)

**Date:** 2026-01-03  
**Scope:** Execution-centric syscalls & Ring3 runtime wiring (Phase 2.3)  
**Author:** Codex AI

## Kernel Mechanism Status
- `sys_v2_map_memory` / `sys_v2_unmap_memory` now call paging; page-by-page unmap implemented.  
- `sys_v2_submit_execution` validates BCIB header (magic/version/opcodes) and allocates a placeholder execution ID (Ring0 mechanism only).  
- `sys_v2_wait_result` reads placeholder results; real completion signal/timer still TODO.  
- Legacy POSIX syscalls and policy stubs remain; removal planned for Phase 2.5.

## Userspace Runtime Status
- New crates added: `userspace/bcib-runtime` (executor + syscall shim) and `userspace/dsl-parser` (hierarchical DSL parser).  
- Dispatcher binary (`bcib-runtime/src/bin/dispatcher.rs`) builds a sample BCIB from DSL and can invoke `submit_execution`/`wait_result` when `RUN_IN_QEMU=1`.  
- Rust workspace set under `userspace/Cargo.toml`; Makefile target `userspace-runtime` builds dispatcher.

## Open Items (to stay aligned with Phase 2)
- Implement real execution completion/timeout path in `wait_result` and hook to Ring3 executor once available.  
- Integrate dispatcher/runtime into QEMU image and run an end-to-end submit/wait test.  
- Extend ABDF meta/types for UI/GPU per Phase 2 docs and wire runtime container registry to ABDF views.  
- Proceed to Phase 2.4 AI migration only after the above mechanism is stable; defer Phase 2.5 cleanup until then.
