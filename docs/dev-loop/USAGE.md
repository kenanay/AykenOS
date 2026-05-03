# Dev Loop Usage Guide

**Audience**: Developers using the dev loop system  
**Status**: Usage Reference

---

## Daily Development Workflow

### During Coding (Every Small Change)
```bash
./scripts/dev_loop.sh smoke
```
**Time**: 5-10 seconds  
**Purpose**: Quick boot validation

---

### Feature Complete (Within Task)
```bash
./scripts/dev_loop.sh contract
```
**Time**: ~1-2 minutes  
**Purpose**: Runtime contract validation

---

### Before Commit
```bash
./scripts/dev_loop.sh full
```
**Time**: ~2-3 minutes  
**Purpose**: Comprehensive validation

---

### Regression Detected
```bash
./scripts/find_regression.sh <last-good-commit>
```
**Purpose**: Automatically find first failing commit

---

## Status Check Workflow

### After Any Dev Loop Run
```bash
./scripts/status.sh
```
**Purpose**: View system health

---

### Quick Health Check
```bash
./scripts/status.sh | grep "❌"
```
**Purpose**: Find failures quickly

---

### Performance Check
```bash
./scripts/status.sh | grep PERFORMANCE
```
**Purpose**: Check for performance regressions

---

### Full Diagnostic
```bash
./scripts/status.sh
```
**Purpose**: View last 20 log lines

---

## Evidence Pipeline Workflow

### View Latest Evidence
```bash
ls -la out/evidence/latest/reports/
```
**Purpose**: See structured reports

---

### Web Dashboard
```bash
cd tools/web && python3 -m http.server 8080
```
**URL**: `http://localhost:8080`  
**Purpose**: Live monitoring, run comparison, performance trending

---

### Dashboard Features
- **Live Status**: Auto-refreshes every 2 seconds
- **Run Comparison**: Select two runs, click "Compare"
- **Performance Graph**: Boot time trending
- **Marker Display**: Marker presence visualization

---

## Mental Model

- **smoke**: "Does it boot?"
- **contract**: "Does it work correctly?"
- **full**: "Is it provable?"
- **find_regression**: "Which commit broke it?"
- **status**: "What's the system health?"
- **evidence**: "What happened in this run?"
- **dashboard**: "Show me everything (live + history + diff + graph)"

---

## CI Integration

### Automatic Validation
- Every PR triggers: smoke → contract → full → isolation
- Failure triggers auto-bisect
- Branch protection blocks merge on failure

### Viewing CI Results
1. Go to GitHub Actions run
2. Scroll to "Artifacts" section
3. Download desired artifact
4. Extract and view logs

---

## Troubleshooting

### Smoke Failure
1. Download `smoke-logs` artifact
2. Check `boot_watch.log` for boot markers
3. Look for `[[AYKEN_BOOT_OK]]` marker
4. Check for build errors

### Contract Failure
1. Download `contract-logs` artifact
2. Check which contract test failed
3. Review test-specific logs
4. Check for VCP runtime issues

### Full Failure
1. Download `full-logs` artifact
2. Check evidence test logs
3. Review verification layer behavior

### Isolation Failure
1. Download `isolation-logs` artifact
2. Check baseline vs dev loop comparison
3. Look for marker set differences
4. Verify kernel behavior consistency

---

## References

- **Spec**: `.kiro/specs/dev-loop-boot-monitoring/`
- **Implementation Guide**: `docs/dev-loop/IMPLEMENTATION_GUIDE.md`
- **CI Integration**: `docs/dev-loop/CI_INTEGRATION.md`
- **Performance**: `docs/dev-loop/PERFORMANCE_INTEGRATION.md`

---

**Last Updated**: 2026-05-03  
**Maintainer**: Kenan AY
