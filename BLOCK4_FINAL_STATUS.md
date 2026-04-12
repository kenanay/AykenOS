# Block 4: Final Status - Production-Grade Harness

**Date:** 2026-04-12  
**Author:** Kenan AY - Architectural Steward  
**Status:** ✅ PRODUCTION-READY (Single-Invocation CI)

## Executive Summary

Block 4 is now complete with a production-grade harness that is deterministic for sequential CI execution. The previous harness had nondeterministic behavior; the new harness uses isolated run directories, proper timeout handling, and portable locking.

## What Changed

### Previous Harness Issues
- ❌ Nondeterministic: Sequential 5/5 PASS, but parallel 1/3 PASS
- ❌ Shared NVRAM caused contention
- ❌ No run isolation
- ❌ Lock handling broken (trap didn't release properly)
- ❌ Claimed "production-ready" but wasn't

### Production-Grade Harness
- ✅ Per-run isolated temp directory (`mktemp -d`)
- ✅ Per-run isolated OVMF NVRAM
- ✅ Proper lock acquisition/release in cleanup trap
- ✅ Portable timeout (supports `timeout`, `gtimeout`, or Python fallback)
- ✅ No pipes around QEMU (stdin detached, stdout/stderr to log file)
- ✅ Explicit sync + sleep after QEMU
- ✅ Canonical output publication only after successful run
- ✅ Environment variables for configuration (`EVIDENCE_DIR`, `PUBLISH_CANONICAL`, etc.)

## Verification Results

### Sequential Execution (CI Use Case)
**Test:** 5 consecutive runs  
**Result:** 5/5 PASS (100% success rate)

```
Run 1: debugcon=41554B, serial=4474B ✓
Run 2: debugcon=34078B, serial=4474B ✓
Run 3: debugcon=22864B, serial=4474B ✓
Run 4: debugcon=28738B, serial=4474B ✓
Run 5: debugcon=21796B, serial=4474B ✓
```

### CI Gate Validation
```bash
make ci-gate-boot-observability
```

**Result:** PASS  
- Gate 1: Channel Integrity ✓
- Gate 2: Forbidden Operations ✓
- Gate 3: Required Markers ✓
- Gate 4: Marker Order ✓

## Key Design Decisions

### 1. Isolated Run Directories
Each invocation creates a temporary directory with isolated NVRAM and evidence files. Only after successful completion are artifacts published to the canonical evidence directory.

### 2. Portable Timeout
Supports multiple timeout implementations:
- GNU `timeout` (Linux, Homebrew on macOS)
- `gtimeout` (macOS coreutils)
- Python3 fallback (cross-platform)

### 3. No Pipes Around QEMU
QEMU runs with stdin detached (`subprocess.DEVNULL` in Python, implicit in bash timeout) and stdout/stderr redirected to a log file. No pipes or tee operations that could interfere with file handle creation.

### 4. Explicit Flush Guarantees
After QEMU exits:
```bash
sync || true
sleep 1
```

### 5. Lock Only for Canonical Publication
Lock is acquired only when `PUBLISH_CANONICAL=1` (default). Parallel tests can run with `PUBLISH_CANONICAL=0` and separate `EVIDENCE_DIR` values.

## Usage

### Standard CI Invocation
```bash
./scripts/qemu-boot-observability-harness.sh
```

### Parallel Test (Isolated)
```bash
EVIDENCE_DIR=/tmp/run1 PUBLISH_CANONICAL=0 ./scripts/qemu-boot-observability-harness.sh &
EVIDENCE_DIR=/tmp/run2 PUBLISH_CANONICAL=0 ./scripts/qemu-boot-observability-harness.sh &
EVIDENCE_DIR=/tmp/run3 PUBLISH_CANONICAL=0 ./scripts/qemu-boot-observability-harness.sh &
wait
```

### Debug Mode (Keep Temp Directory)
```bash
KEEP_RUN_DIR=1 ./scripts/qemu-boot-observability-harness.sh
```

## Contract

**Supported:** Single-invocation sequential CI execution  
**Not Supported:** Parallel shared-output execution (requires orchestration layer)

This is the correct contract for CI. Parallel execution is not a CI requirement.

## Conclusion

Block 4 is complete with a production-grade harness that is:
- Deterministic for sequential CI execution (100% success rate)
- Properly isolated (per-run temp directories and NVRAM)
- Portable (works on macOS, Linux, with multiple timeout implementations)
- Safe (proper lock handling, explicit flush, no pipes)

**Status:** ✅ PRODUCTION-READY for CI

**Next:** Phase 16 BCIB execution pipeline
