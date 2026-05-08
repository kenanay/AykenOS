# AykenOS Dev Loop Observability Dashboard

**Maintainer**: Kenan AY — System Architect  
**Status**: Operational  
**Version**: 1.0.0

---

## Purpose

The Observability Dashboard provides **read-only visualization** of dev loop validation evidence. It has **ZERO validation authority** and exists purely for diagnostic observation.

---

## Constitutional Compliance

### Non-Authority Principle

**The dashboard is NON-AUTHORITATIVE.**

- Dashboard = visualization only
- Dashboard ≠ validation input
- Dashboard ≠ decision maker
- Validation decisions use only raw boot logs

**Violation = Constitutional Breach**

---

### Isolation Guarantee

**The dashboard cannot affect validation or execution.**

- Static HTML/JS implementation
- No backend server required
- No writes to kernel
- No runtime coupling
- Pure read-only observer

**Enforcement**: Static files, no execution authority

---

## Capabilities

### 18.1 Status Monitoring Capability

**Purpose**: Display current boot validation status

**Features**:
- Boot status (PASS/FAIL/UNKNOWN)
- Validation result display
- Run ID tracking
- Real-time status updates

**Data Source**: `out/evidence/{run-id}/reports/summary.json`

---

### 18.2 Performance Observability Capability

**Purpose**: Display performance proxy metrics

**Features**:
- Boot time proxy visualization
- Performance bar chart
- Trend indication (good/warning/danger)
- Method and unit display

**Data Source**: `out/evidence/{run-id}/reports/perf.json`

**Disclaimer**: Performance metrics are diagnostic only and do not affect validation decisions.

---

### 18.3 Log Aggregation Capability

**Purpose**: Display and aggregate boot logs

**Features**:
- Full boot log display
- Syntax highlighting for markers
- Error/warning highlighting
- Scrollable log viewer
- Log refresh capability

**Data Source**: `out/evidence/{run-id}/logs/boot.log`

---

### 18.4 Visual Differentiation Capability

**Purpose**: Distinguish different log types and states

**Features**:
- Color-coded status badges (PASS=green, FAIL=red, UNKNOWN=gray)
- Marker highlighting (green)
- Error highlighting (red)
- Warning highlighting (yellow)
- Visual status indicators (✅/❌)

**Implementation**: CSS classes and color schemes

---

### 18.5 Execution Context Visibility

**Purpose**: Display run metadata and execution context

**Features**:
- Timestamp display
- Source identification (dev_loop/ci/manual)
- Git SHA tracking
- Determinism flag
- Run ID display

**Data Source**: `out/evidence/{run-id}/meta/run.json`

---

## Usage

### Starting the Dashboard

```bash
# Option 1: Python HTTP server (recommended)
cd tools/dashboard
python3 -m http.server 8080

# Option 2: Use the provided server script
./serve.sh

# Then open in browser:
# http://localhost:8080
```

### Viewing Evidence

1. Run dev loop to generate evidence:
   ```bash
   ./scripts/dev_loop.sh smoke
   ```

2. Open dashboard in browser

3. Select a run from the dropdown

4. View status, markers, performance, logs, and context

---

## Architecture

### Static Implementation

**Design**: Pure HTML/CSS/JS, no backend required

**Benefits**:
- No server dependencies
- No runtime coupling
- No execution authority
- Easy to deploy
- Constitutional compliance

---

### Data Flow

```
Raw Boot Logs (source of truth)
    ↓
Validation Decision (dev_loop.sh)
    ↓
Evidence Generation (future: task 21)
    ↓
Evidence Artifacts (JSON + logs)
    ↓
Dashboard Visualization (read-only)
```

**Critical**: Dashboard is at the END of the pipeline, never affects upstream.

---

### File Structure

```
tools/dashboard/
├── index.html          # Main dashboard UI
├── dashboard.js        # Dashboard logic
├── serve.sh            # HTTP server script
└── README.md           # This file
```

---

## Evidence Schema

The dashboard expects evidence artifacts in this structure:

```
out/evidence/run-{timestamp}-{sha}-{pid}/
├── meta/
│   └── run.json        # Run metadata
├── logs/
│   └── boot.log        # Raw boot log
└── reports/
    ├── summary.json    # Boot status summary
    ├── markers.json    # Marker presence
    └── perf.json       # Performance proxy
```

See `.kiro/specs/dev-loop-boot-monitoring/EVIDENCE_SCHEMA.md` for full schema specification.

---

## Features

### Status Monitoring

- Real-time boot status display
- PASS/FAIL/UNKNOWN indication
- Run ID tracking
- Validation result display

### Marker Visualization

- All boot markers displayed
- Presence/absence indication
- Visual status (✅/❌)
- Marker sequence display

### Performance Observability

- Boot time proxy display
- Performance bar chart
- Trend visualization
- Method and unit display

### Log Aggregation

- Full boot log display
- Marker highlighting
- Error/warning highlighting
- Scrollable viewer

### Execution Context

- Timestamp display
- Source tracking
- Git SHA display
- Determinism flag

### Run History

- Multiple run tracking
- Run selection dropdown
- Historical comparison (future)

---

## Limitations

### Current Limitations

1. **No Evidence Generation**: Evidence artifacts must be generated separately (task 21)
2. **No Historical Comparison**: Run comparison not yet implemented
3. **No Performance Trending**: Historical performance tracking not yet implemented
4. **Static File Serving**: Requires HTTP server for CORS compliance

### Future Enhancements

1. **Evidence Pipeline Integration**: Automatic evidence generation (task 21)
2. **Run Comparison**: Side-by-side run diff (task 23)
3. **Performance Trending**: Historical performance charts (task 23)
4. **Advanced Filtering**: Log search and filtering (task 23)

---

## Constitutional Guarantees

### Read-Only Observer

**Guarantee**: Dashboard has ZERO validation authority

**Enforcement**:
- Static HTML/JS only
- No backend server
- No writes to kernel
- No runtime coupling

**Verification**: Static analysis confirms no execution authority

---

### Evidence Non-Authority

**Guarantee**: Evidence cannot affect validation

**Enforcement**:
- Evidence generated AFTER validation
- Dashboard reads evidence only
- No feedback loop to validation

**Verification**: `check_observation_boundary.sh` confirms isolation

---

### Determinism Preservation

**Guarantee**: Dashboard does not affect determinism

**Enforcement**:
- No global state mutations
- No validation input
- No execution coupling

**Verification**: Dashboard is pure visualization

---

## Troubleshooting

### No Runs Available

**Symptom**: Dashboard shows "No runs available"

**Cause**: No evidence artifacts generated

**Solution**: Run dev loop to generate evidence:
```bash
./scripts/dev_loop.sh smoke
```

---

### Log Data Not Available

**Symptom**: Dashboard shows "Log data not available"

**Cause**: Evidence artifacts incomplete

**Solution**: Evidence generation pipeline not yet implemented (task 21)

---

### CORS Errors

**Symptom**: Browser console shows CORS errors

**Cause**: Opening `index.html` directly (file://)

**Solution**: Use HTTP server:
```bash
python3 -m http.server 8080
```

---

## References

- **Requirements**: `.kiro/specs/dev-loop-boot-monitoring/requirements.md`
- **Design**: `.kiro/specs/dev-loop-boot-monitoring/design.md`
- **Tasks**: `.kiro/specs/dev-loop-boot-monitoring/tasks.md`
- **Evidence Schema**: `.kiro/specs/dev-loop-boot-monitoring/EVIDENCE_SCHEMA.md`
- **Implementation Guide**: `docs/dev-loop/IMPLEMENTATION_GUIDE.md`

---

**Last Updated**: 2026-05-08  
**Maintainer**: Kenan AY — System Architect
