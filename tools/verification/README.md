# AykenOS Verification Layer

**Status:** Production-Ready (Phase-17 Integration)  
**Version:** 1.0  
**Author:** Kenan AY  
**Date:** 2026-04-25  

## Overview

The AykenOS Verification Layer is a production-ready, evidence-driven verification system that validates AykenOS stability through non-invasive observation and deterministic proof generation. This system implements the core principle "No Evidence = No Truth" across all system components.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Verification Layer                       │
│                                                              │
│  ┌────────────┐      ┌──────────────┐      ┌─────────────┐ │
│  │  Manifest  │─────▶│ Orchestrator │─────▶│   Report    │ │
│  │   (JSON)   │      │ (run_all.sh) │      │   (JSON)    │ │
│  └────────────┘      └──────┬───────┘      └─────────────┘ │
│                             │                               │
│                             ▼                               │
│                    ┌─────────────────┐                      │
│                    │  Gate Executor  │                      │
│                    └────────┬────────┘                      │
│                             │                               │
│              ┌──────────────┼──────────────┐               │
│              ▼              ▼              ▼               │
│         ┌────────┐     ┌────────┐     ┌────────┐          │
│         │ Gate 1 │     │ Gate 2 │     │ Gate N │          │
│         └───┬────┘     └───┬────┘     └───┬────┘          │
│             │              │              │               │
│             ▼              ▼              ▼               │
│        ┌─────────┐   ┌─────────┐   ┌─────────┐           │
│        │Evidence │   │Evidence │   │Evidence │           │
│        │  (JSON) │   │  (JSON) │   │  (JSON) │           │
│        └─────────┘   └─────────┘   └─────────┘           │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

### Fast Verification (1 gate, ~10 seconds)
```bash
make verify-fast
```

### Standard Verification (3 gates, ~30 seconds)
```bash
make verify-system
```

### Heavy Verification (all gates, ~2 minutes)
```bash
make verify-heavy
```

### Shadow Mode (non-blocking)
```bash
make verify-shadow
```

## Current Gates (Phase-17)

| Gate ID | Tier | Determinism Level | Purpose |
|---------|------|------------------|---------|
| `boot_integrity` | fast | marker | Validates UEFI→kernel handoff |
| `ring3_runtime` | standard | marker | Validates Ring3 execution capability |
| `determinism_global_enforcement` | standard | marker | Validates constitutional determinism rules |

## Evidence Chain

Each verification run produces:
- **Run ID**: ISO 8601 timestamp (e.g., `2026-04-25T22:32:42Z`)
- **Evidence Files**: JSON reports for each gate
- **Canonical Hash**: SHA256 of sorted evidence chain
- **Report**: Aggregated verification results

Example evidence path:
```
out/evidence/verification/2026-04-25T22:32:42Z/
├── gates/
│   ├── boot_integrity/attempt-1/report.json
│   ├── ring3_runtime/attempt-1/report.json
│   └── determinism_global_enforcement/attempt-1/report.json
└── report.json
```

## Trust Chain Verification

The system enforces trust through:
1. **Evidence Integrity**: Each evidence file contains its own hash
2. **Canonical Hash**: Deterministic hash of all evidence files
3. **Command Fingerprint**: SHA256 of executed commands
4. **Run ID Coupling**: All evidence tied to single run
5. **Schema Validation**: Strict JSON schema enforcement

## Constitutional Enforcement

The verification layer enforces constitutional rules:
- **NON_OVERRIDABLE**: Memory safety, kernel safety, security boundaries
- **Phase Matrix**: Determinism, allocation, error handling rules
- **Fail-Closed**: System fails explicitly, never silently passes

## Adding New Gates

1. **Define in Manifest** (`manifest.json`):
```json
{
  "id": "new_gate",
  "command": "make ci-gate-new-feature",
  "evidence_path": "gates/new_gate/attempt-{attempt}/report.json",
  "required_verdict": "PASS",
  "blocking": true,
  "performance_tier": "standard",
  "determinism_level": "marker",
  "required_markers": ["[NEW_FEATURE_OK]"],
  "forbidden_markers": ["NEW_FEATURE_FAIL", "PANIC"]
}
```

2. **Implement Gate Command** (Makefile):
```makefile
ci-gate-new-feature: ci-evidence-dir
	@echo "== CI GATE NEW FEATURE =="
	@./tools/verification/adapters/make_gate_adapter.sh \
		"$(EVIDENCE_RUN_DIR)/gates/new_gate" \
		"new_feature" \
		"marker" \
		"make test-new-feature"
```

3. **Test Integration**:
```bash
make verify-system  # Should include new gate
```

## Troubleshooting

### Common Issues

**Gate Fails with "Evidence file not found"**
- Check `AYKEN_EVIDENCE_DIR` environment variable
- Verify gate command produces evidence at expected path
- Check adapter script execution

**Hash Mismatch Error**
- Evidence files modified after generation
- Run ID mismatch between orchestrator and validator
- Canonical hash computation difference

**Validation Failed**
- Evidence doesn't match JSON schema
- Required markers missing from output
- Forbidden markers present in output

### Debug Commands

```bash
# Verbose verification
./tools/verification/run_all.sh --tier standard --verbose

# Validate specific evidence
./tools/verification/validators/validate_evidence.py \
  --evidence-file out/evidence/verification/latest/gates/boot_integrity/attempt-1/report.json \
  --run-id 2026-04-25T22:32:42Z

# Check manifest validity
./tools/verification/validators/validate_manifest.py \
  --manifest tools/verification/manifest.json
```

## Integration with CI

The verification layer integrates with CI through:
- **Pre-CI Discipline**: Local fail-closed gates before CI
- **CI Hard Gates**: Blocking verification in CI pipeline
- **Shadow Mode**: Non-blocking verification for testing

### CI Integration Example
```yaml
# .github/workflows/ci.yml
- name: Verification Layer
  run: make verify-system
  # Fails CI if any blocking gate fails
```

## Performance Tiers

| Tier | Gates | Duration | Use Case |
|------|-------|----------|----------|
| `fast` | 1 gate | ~10s | Quick feedback loop |
| `standard` | 3 gates | ~30s | Standard CI verification |
| `heavy` | All gates | ~2min | Full system validation |

## Schema Reference

- **Manifest Schema**: `schemas/manifest.schema.json`
- **Evidence Schema**: `schemas/evidence.schema.json`  
- **Report Schema**: `schemas/report.schema.json`

## Files and Directories

```
tools/verification/
├── README.md                    # This file
├── run_all.sh                   # Main orchestrator
├── manifest.json                # Gate definitions
├── schemas/                     # JSON schemas
│   ├── manifest.schema.json
│   ├── evidence.schema.json
│   └── report.schema.json
├── validators/                  # Python validators
│   ├── validate_manifest.py
│   ├── validate_evidence.py
│   └── validate_report.py
└── adapters/                    # Gate adapters
    ├── README.md
    ├── evidence_adapter.py
    └── make_gate_adapter.sh
```

## Phase-17 Integration Status

✅ **MVP Complete**: Evidence chain, trust anchor, constitutional enforcement  
✅ **Production Ready**: 3 gates operational, fail-closed behavior verified  
✅ **CI Integration**: Pre-CI discipline and hard gate modes active  
🔄 **Phase-17 Pending**: Execution pipeline integration, real workload validation  

## Support

For issues or questions:
- Check troubleshooting section above
- Review evidence files in `out/evidence/verification/latest/`
- Consult specification documents in `.kiro/specs/tools-verification-layer/`

---

**Authority**: Kenan AY - Architectural Steward  
**Implementation**: Phase-16 MVP → Phase-17 Production Integration  
**Status**: Production-Ready for Phase-17 Execution Pipeline