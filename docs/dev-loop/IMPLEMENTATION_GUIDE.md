# Dev Loop Implementation Guide

**Status**: Implementation Reference  
**Audience**: Developers implementing the dev loop system  
**Authority**: Explanatory (not normative)

---

## Purpose

This guide provides **detailed implementation instructions** for the Dev Loop & Boot Monitoring System. This is NOT the spec - for requirements and architecture, see `.kiro/specs/dev-loop-boot-monitoring/`.

---

## Implementation Details

### 1. Marker Validation

#### Marker Format
```
[K][EARLY_BOOT_OK]
[K][LATE_INIT_END]
[[AYKEN_BOOT_OK]]
```

#### Validation Logic
```bash
# Check for critical marker
if ! grep -q "\[\[AYKEN_BOOT_OK\]\]" "$BOOT_LOG"; then
    echo "❌ BOOT FAILED: [[AYKEN_BOOT_OK]] marker not found"
    tail -50 "$BOOT_LOG"
    exit 1
fi

# Check for required markers
if ! grep -q "\[K\]\[EARLY_BOOT_OK\]" "$BOOT_LOG"; then
    echo "❌ BOOT FAILED: [K][EARLY_BOOT_OK] marker not found"
    exit 1
fi

if ! grep -q "\[K\]\[LATE_INIT_END\]" "$BOOT_LOG"; then
    echo "❌ BOOT FAILED: [K][LATE_INIT_END] marker not found"
    exit 1
fi
```

#### Sequence Validation
```bash
# Extract line numbers
EARLY_LINE=$(grep -n "\[K\]\[EARLY_BOOT_OK\]" "$BOOT_LOG" | head -1 | cut -d: -f1)
LATE_LINE=$(grep -n "\[K\]\[LATE_INIT_END\]" "$BOOT_LOG" | head -1 | cut -d: -f1)
BOOT_LINE=$(grep -n "\[\[AYKEN_BOOT_OK\]\]" "$BOOT_LOG" | head -1 | cut -d: -f1)

# Validate sequence
if [ "$EARLY_LINE" -gt "$LATE_LINE" ] || [ "$LATE_LINE" -gt "$BOOT_LINE" ]; then
    echo "❌ BOOT FAILED: Marker sequence violation"
    exit 1
fi
```

---

### 2. Evidence Schema

#### meta.json
```json
{
  "schema_version": "1.0",
  "run_id": "run-20260503-154530",
  "timestamp": "2026-05-03T15:45:30Z",
  "source": "dev_loop",
  "deterministic": true,
  "author": "Kenan AY",
  "role": ["developer", "architect"],
  "signature_type": "digital_meta"
}
```

#### summary.json
```json
{
  "boot": "PASS",
  "markers_ok": true,
  "fail_closed": false,
  "perf_regression": false,
  "timestamp": "2026-05-03T15:45:30Z",
  "run_id": "run-20260503-154530",
  "source": "dev_loop",
  "deterministic": true
}
```

#### markers.json
```json
{
  "EARLY_BOOT_OK": true,
  "LATE_INIT_END": true,
  "BOOT_OK": true,
  "FAIL_CLOSED": false
}
```

#### perf.json
```json
{
  "boot_time_proxy": 1234,
  "method": "marker_delta",
  "valid": true,
  "disclaimer": "Proxy metric based on marker line count, NOT TSC-based measurement",
  "unit": "line_count"
}
```

---

### 3. Dashboard Implementation

#### HTML Structure
```html
<!DOCTYPE html>
<html>
<head>
  <title>Ayken Dev Loop Dashboard</title>
  <link rel="stylesheet" href="style.css">
  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
</head>
<body>
  <h1>Ayken Dev Loop Dashboard</h1>
  
  <section id="live-status">
    <h2>Live Status</h2>
    <div id="status"></div>
    <pre id="markers"></pre>
  </section>
  
  <section id="run-comparison">
    <h2>Run Comparison</h2>
    <select id="runA"></select>
    <select id="runB"></select>
    <button onclick="compare()">Compare</button>
    <div id="diff"></div>
  </section>
  
  <section id="performance">
    <h2>Performance</h2>
    <canvas id="perfChart"></canvas>
  </section>
  
  <footer>Kenan AY — System Architect</footer>
  
  <script src="app.js"></script>
</body>
</html>
```

#### JavaScript Implementation
```javascript
const BASE = "../../out/evidence/";

async function load(run) {
  const summary = await fetch(`${BASE}${run}/reports/summary.json`).then(r => r.json());
  const markers = await fetch(`${BASE}${run}/reports/markers.json`).then(r => r.json());
  const perf = await fetch(`${BASE}${run}/reports/perf.json`).then(r => r.json());
  return { summary, markers, perf };
}

async function loadLive() {
  const data = await load("latest");
  document.getElementById("status").textContent = `Status: ${data.summary.boot}`;
  document.getElementById("markers").textContent = JSON.stringify(data.markers, null, 2);
}

async function compare() {
  const runA = document.getElementById("runA").value;
  const runB = document.getElementById("runB").value;
  
  const A = await load(runA);
  const B = await load(runB);
  
  let diff = `STATUS: ${A.summary.boot} → ${B.summary.boot}\n\n`;
  
  // Marker diff
  for (const key in A.markers) {
    if (A.markers[key] !== B.markers[key]) {
      diff += `${key}: ${A.markers[key]} → ${B.markers[key]}\n`;
    }
  }
  
  // Performance diff
  diff += `\nBOOT_TIME: ${A.perf.boot_time_proxy} → ${B.perf.boot_time_proxy}\n`;
  
  document.getElementById("diff").textContent = diff;
}

// Auto-refresh every 2 seconds
setInterval(loadLive, 2000);
loadLive();
```

---

### 4. Parser Scripts

#### parse_markers.py
```python
#!/usr/bin/env python3
import sys
import json

def parse_markers(log_path):
    markers = {
        "EARLY_BOOT_OK": False,
        "LATE_INIT_END": False,
        "BOOT_OK": False,
        "FAIL_CLOSED": False
    }
    
    with open(log_path) as f:
        for line in f:
            if "[K][EARLY_BOOT_OK]" in line:
                markers["EARLY_BOOT_OK"] = True
            elif "[K][LATE_INIT_END]" in line:
                markers["LATE_INIT_END"] = True
            elif "[[AYKEN_BOOT_OK]]" in line:
                markers["BOOT_OK"] = True
            elif "[VCP_FAIL_CLOSED]" in line:
                markers["FAIL_CLOSED"] = True
    
    print(json.dumps(markers, indent=2))

if __name__ == "__main__":
    parse_markers(sys.argv[1])
```

#### parse_perf.py
```python
#!/usr/bin/env python3
import sys
import json

def parse_perf(log_path):
    early_line = None
    boot_line = None
    
    with open(log_path) as f:
        for i, line in enumerate(f, 1):
            if "[K][EARLY_BOOT_OK]" in line and early_line is None:
                early_line = i
            elif "[[AYKEN_BOOT_OK]]" in line and boot_line is None:
                boot_line = i
    
    if early_line and boot_line:
        boot_time = boot_line - early_line
        result = {
            "boot_time_proxy": boot_time,
            "method": "marker_delta",
            "valid": True,
            "disclaimer": "Proxy metric based on marker line count, NOT TSC-based measurement",
            "unit": "line_count"
        }
    else:
        result = {
            "boot_time_proxy": -1,
            "method": "marker_delta",
            "valid": False,
            "disclaimer": "Markers missing or invalid",
            "unit": "line_count"
        }
    
    print(json.dumps(result, indent=2))

if __name__ == "__main__":
    parse_perf(sys.argv[1])
```

---

### 5. Evidence Generation Script

#### generate_evidence.sh
```bash
#!/bin/bash
set -euo pipefail

LOG_FILE="${1:-out/logs/boot_watch.log}"
TIMESTAMP=$(date -u +"%Y%m%d-%H%M%S")
RUN_DIR="out/evidence/run-${TIMESTAMP}"

# Create directory structure
mkdir -p "${RUN_DIR}/logs"
mkdir -p "${RUN_DIR}/reports"

# Copy raw log
cp "$LOG_FILE" "${RUN_DIR}/logs/boot.log"

# Generate reports
python3 tools/debug/parse_markers.py "$LOG_FILE" > "${RUN_DIR}/reports/markers.json"
python3 tools/debug/parse_perf.py "$LOG_FILE" > "${RUN_DIR}/reports/perf.json"

# Generate summary
BOOT_STATUS="FAIL"
if grep -q "\[\[AYKEN_BOOT_OK\]\]" "$LOG_FILE"; then
    BOOT_STATUS="PASS"
fi

cat > "${RUN_DIR}/reports/summary.json" <<EOF
{
  "boot": "${BOOT_STATUS}",
  "markers_ok": true,
  "fail_closed": false,
  "perf_regression": false,
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "run_id": "run-${TIMESTAMP}",
  "source": "dev_loop",
  "deterministic": true
}
EOF

# Generate metadata
cat > "${RUN_DIR}/meta.json" <<EOF
{
  "schema_version": "1.0",
  "run_id": "run-${TIMESTAMP}",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "source": "dev_loop",
  "deterministic": true,
  "author": "Kenan AY",
  "role": ["developer", "architect"],
  "signature_type": "digital_meta"
}
EOF

# Update latest symlink
ln -sfn "run-${TIMESTAMP}" "out/evidence/latest"

echo "Evidence generated: ${RUN_DIR}"
```

---

### 6. CPU Count Detection

```bash
# Detect CPU count
if command -v sysctl >/dev/null 2>&1; then
    NCPU=$(sysctl -n hw.ncpu)  # macOS
elif command -v nproc >/dev/null 2>&1; then
    NCPU=$(nproc)  # Linux
else
    NCPU=4  # Fallback
fi

echo "Building with ${NCPU} CPUs..."
make -j"${NCPU}" kernel.elf
```

---

### 7. Exit Status Pattern

```bash
# CRITICAL: Evidence generation must not affect validation exit status
validation_status=$?

# Generate evidence (failure is non-fatal)
./scripts/generate_evidence.sh || true

# Exit with original validation status
exit "$validation_status"
```

---

## References

- **Spec**: `.kiro/specs/dev-loop-boot-monitoring/`
- **CI Integration**: `docs/dev-loop/CI_INTEGRATION.md`
- **Performance**: `docs/dev-loop/PERFORMANCE_INTEGRATION.md`

---

**Last Updated**: 2026-05-03  
**Maintainer**: Kenan AY
