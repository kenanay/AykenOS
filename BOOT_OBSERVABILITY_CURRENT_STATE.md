# Boot Observability Pipeline - Current State Documentation

**Date:** 2026-04-12  
**Status:** DEMO-READY (NOT PRODUCTION-READY)  
**Author:** Kenan AY - Architectural Steward

## Executive Summary

The boot observability evidence pipeline is **functionally working** in sequential execution contexts but **NOT deterministic** across all execution scenarios. The system passes in isolated tests but fails under parallel execution and certain CI contexts.

## What Works (Sequential Execution)

### ✅ Successful Scenarios
1. **Direct harness execution:** `./scripts/qemu-boot-observability-harness.sh` → 5/5 PASS
2. **Make target execution:** `make ci-gate-boot-observability` → 3/3 PASS
3. **Manual QEMU commands:** Direct timeout + QEMU → Consistent PASS
4. **Post-warmup execution:** After first successful run, subsequent runs succeed

### ✅ Working Components
- QEMU debugcon capture (port 0xE9) → ~30-70KB output
- QEMU serial capture (COM1) → ~4.4KB output
- Boot markers present and in correct order:
  - `[B][UEFI_BOOT_START]` (bootloader)
  - `[[AYKEN_BOOT_OK]]` (kernel entry)
  - `[K][EARLY_BOOT_OK]` (kmain)
- CI gate validation (4 gates: channel integrity, forbidden ops, markers, order)
- Stdin redirection fix (`< /dev/null`) prevents SIGTTIN/SIGTTOU suspension

## What Fails (Nondeterministic Behavior)

### ❌ Failure Scenarios
1. **Parallel execution:** 3 simultaneous QEMU instances → ALL produced 0-byte debugcon
2. **Cold start (no warmup):** Fresh execution sometimes produces 0-byte logs
3. **CI hook context:** `agentStop` hook triggers `pre_ci_discipline.sh` → ABI gate fails (separate issue)
4. **Isolated test runs:** Same command, same environment → inconsistent results (0 bytes vs 64KB)

### ❌ Root Causes Identified

#### 1. **Timeout + Buffer Flush Problem**
```bash
timeout 45 qemu-system-x86_64 ... > /dev/null 2>&1
```
- `timeout` sends SIGTERM (exit code 124)
- QEMU terminates without flushing debugcon/serial buffers to disk
- Result: 0-byte files even though QEMU wrote to buffers

**Evidence:**
- `timeout 10` → 0 bytes
- `timeout 45` → 64KB (sometimes)
- Manual execution (no timeout) → Consistent success

#### 2. **OVMF NVRAM File Contention**
```bash
-drive if=pflash,format=raw,file="build/OVMF_VARS_RUN.fd"
```
- Shared `OVMF_VARS_RUN.fd` across parallel executions
- File locking / race condition
- Undefined behavior when multiple QEMU instances access same NVRAM

**Evidence:**
- Parallel test: All 3 runs → 0 bytes
- Sequential with separate NVRAM files → Still fails (other issues present)

#### 3. **File Descriptor / Creation Timing**
- QEMU creates log files late in boot sequence
- If QEMU is killed before file creation, no evidence exists
- No explicit sync/flush after QEMU termination

#### 4. **Execution Context Sensitivity**
- Interactive terminal (foreground) → Works
- Script/background → Sometimes fails
- CI pipeline (cold, isolated) → Fails
- Post-warmup → Works

## Current Harness Implementation

### Key Fix Applied: Stdin Redirection
```bash
timeout 45 qemu-system-x86_64 \
    -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
    -drive if=pflash,format=raw,file="$OVMF_VARS_RUN" \
    -drive format=raw,file="$EFI_IMAGE" \
    -boot order=c \
    -debugcon file:$DEBUGCON_LOG \
    -global isa-debugcon.iobase=0xe9 \
    -serial file:$SERIAL_LOG \
    -nographic \
    < /dev/null > /dev/null 2>&1
```

**What this fixes:**
- ✅ SIGTTIN/SIGTTOU terminal control signal suspension
- ✅ QEMU no longer attempts to read from stdin
- ✅ Works in script/background contexts

**What this does NOT fix:**
- ❌ Buffer flush on SIGTERM
- ❌ OVMF NVRAM contention
- ❌ File creation timing
- ❌ Parallel execution safety

## Test Results Summary

### Sequential Tests (PASS)
```
Run 1: debugcon=35680 bytes, serial=4474 bytes ✅
Run 2: debugcon=39068 bytes, serial=4474 bytes ✅
Run 3: debugcon=42088 bytes, serial=4474 bytes ✅
Run 4: debugcon=36214 bytes, serial=4474 bytes ✅
Run 5: debugcon=61846 bytes, serial=4474 bytes ✅
```

### Parallel Tests (FAIL)
```
Parallel run 1: 0 bytes ❌
Parallel run 2: 0 bytes ❌
Parallel run 3: 0 bytes ❌
```

### Make Tests (PASS)
```
Make run 1: PASS (debugcon=23932 bytes) ✅
Make run 2: PASS (debugcon=54370 bytes) ✅
Make run 3: PASS (debugcon=26602 bytes) ✅
```

## Why Hook Fails

The `agentStop` hook runs `scripts/ci/pre_ci_discipline.sh` which executes:
```bash
make ci-gate-abi  # ← This fails
```

**Two separate issues:**
1. **ABI baseline mismatch** (unrelated to boot observability)
   - `kernel/sys/syscall_v2.h` changed in previous commit
   - Baseline not updated: `scripts/ci/abi-baseline.lock.json`
   
2. **Boot observability nondeterminism** (if ABI were fixed)
   - Hook runs in cold/isolated context
   - May trigger 0-byte debugcon scenario
   - Would cause `ci-gate-boot-observability` to fail

## Critical Insight: Test Design Flaw

### Current Test Property (WRONG)
```
EXISTS run WHERE output > 0
```
"At least one execution produces output"

### Required Test Property (CORRECT)
```
FOR ALL runs: output > 0
```
"Every execution must produce output"

**Current test:** Checks if system CAN work  
**Required test:** Checks if system ALWAYS works

## Production-Ready Requirements (NOT MET)

To make this production-ready, the following must be implemented:

### 1. Graceful QEMU Termination
```bash
# Replace SIGTERM with SIGINT for graceful shutdown
timeout --signal=SIGINT 45 qemu-system-x86_64 ...

# Add explicit flush after QEMU
wait
sync
sleep 1
```

### 2. OVMF NVRAM Isolation
```bash
# Per-execution NVRAM to prevent contention
NVRAM_INSTANCE="build/OVMF_VARS_RUN_$$.fd"
cp -f "$OVMF_VARS_TEMPLATE" "$NVRAM_INSTANCE"
# Use $NVRAM_INSTANCE in QEMU command
# Clean up after: rm -f "$NVRAM_INSTANCE"
```

### 3. Parallel Execution Lock
```bash
# Prevent concurrent QEMU executions
flock /tmp/qemu-boot-harness.lock -c "
  # QEMU execution here
"
```

### 4. Robust Testing
```bash
# Test must verify determinism
for i in {1..10}; do
  run_harness
  if [[ $DEBUGCON_SIZE -eq 0 ]]; then
    FAIL "Nondeterministic: Run $i produced 0 bytes"
  fi
done
```

### 5. Explicit Flush Guarantees
```bash
# After QEMU terminates
sync
sleep 1
# Verify files exist and are non-zero
if [[ ! -s "$DEBUGCON_LOG" ]]; then
  FAIL "Debugcon log missing or empty"
fi
```

## Current Files

### Harness Script
- `scripts/qemu-boot-observability-harness.sh` (45s timeout, stdin redirect)

### CI Gate Script
- `scripts/ci-gate-boot-observability.sh` (4-gate validation)

### Evidence Output
- `evidence/boot-observability/qemu_debugcon.log` (primary channel)
- `evidence/boot-observability/qemu_serial.log` (secondary channel)
- `evidence/boot-observability/debugcon.trace` (preserved raw order)
- `evidence/boot-observability/serial.trace` (preserved raw order)
- `evidence/boot-observability/boot_observability_evidence.json` (gate results)

### Status Reports
- `BLOCK4_STATUS.md` (optimistic assessment - needs revision)
- `BOOT_OBSERVABILITY_CURRENT_STATE.md` (this document - realistic assessment)

## Recommendations

### Immediate Actions
1. **Document current state** ✅ (this document)
2. **Fix ABI baseline** (separate issue, blocks hook)
3. **Implement production fixes** (5 items above)
4. **Retest with robust test suite** (10+ runs, parallel, cold start)

### Strategic Decision
**Option A: Ship as-is (DEMO)**
- Works in sequential contexts
- Good enough for development/testing
- Risk: CI flakiness, production failures

**Option B: Fix properly (PRODUCTION)**
- Implement all 5 production requirements
- Verify determinism with robust tests
- Safe for CI and production use

## Conclusion

The boot observability pipeline is **functionally correct** but **operationally unreliable**. The system demonstrates the difference between:
- **"It works"** (can produce correct output)
- **"It's proven"** (always produces correct output)

This is a **critical lesson in execution truth engines**: environmental correctness is as important as algorithmic correctness.

**Current Status:** Demo-ready, not production-ready  
**Next Step:** Implement production fixes or accept demo-level reliability  
**Block 4:** ⚠️ INCOMPLETE (nondeterministic behavior unresolved)
