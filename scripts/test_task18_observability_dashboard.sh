#!/usr/bin/env bash
# Test Task 18: Observability Status Dashboard
#
# Purpose: Verify all 5 dashboard capabilities
# Requirements: R10 (Diagnostic Output and Logging)
#
# Maintainer: Kenan AY — System Architect

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
DASHBOARD_DIR="${PROJECT_ROOT}/tools/dashboard"

PASS=0
FAIL=0

echo "=========================================="
echo "Task 18: Observability Dashboard Tests"
echo "=========================================="
echo ""

# Test 18.1: Status Monitoring Capability
echo "Test 18.1: Status Monitoring Capability"
echo "=========================================="
echo ""

echo "Checking dashboard HTML structure..."
if [ -f "${DASHBOARD_DIR}/index.html" ]; then
    echo "✅ Dashboard HTML exists"
    
    # Check for status monitoring elements
    if grep -q "id=\"bootStatus\"" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ Boot status element present"
        PASS=$((PASS + 1))
    else
        echo "❌ Boot status element missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "id=\"validationResult\"" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ Validation result element present"
        PASS=$((PASS + 1))
    else
        echo "❌ Validation result element missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "id=\"runId\"" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ Run ID element present"
        PASS=$((PASS + 1))
    else
        echo "❌ Run ID element missing"
        FAIL=$((FAIL + 1))
    fi
else
    echo "❌ Dashboard HTML not found"
    FAIL=$((FAIL + 3))
fi

echo ""

# Test 18.2: Performance Observability Capability
echo "Test 18.2: Performance Observability Capability"
echo "=========================================="
echo ""

echo "Checking performance visualization elements..."
if [ -f "${DASHBOARD_DIR}/index.html" ]; then
    if grep -q "id=\"perfStatus\"" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ Performance status element present"
        PASS=$((PASS + 1))
    else
        echo "❌ Performance status element missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "id=\"perfValue\"" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ Performance value element present"
        PASS=$((PASS + 1))
    else
        echo "❌ Performance value element missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "perf-bar" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ Performance bar chart present"
        PASS=$((PASS + 1))
    else
        echo "❌ Performance bar chart missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "Diagnostic Only" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ Performance disclaimer present"
        PASS=$((PASS + 1))
    else
        echo "❌ Performance disclaimer missing"
        FAIL=$((FAIL + 1))
    fi
else
    FAIL=$((FAIL + 4))
fi

echo ""

# Test 18.3: Log Aggregation Capability
echo "Test 18.3: Log Aggregation Capability"
echo "=========================================="
echo ""

echo "Checking log aggregation elements..."
if [ -f "${DASHBOARD_DIR}/index.html" ]; then
    if grep -q "id=\"logViewer\"" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ Log viewer element present"
        PASS=$((PASS + 1))
    else
        echo "❌ Log viewer element missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "log-viewer" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ Log viewer styling present"
        PASS=$((PASS + 1))
    else
        echo "❌ Log viewer styling missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "id=\"refreshBtn\"" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ Refresh button present"
        PASS=$((PASS + 1))
    else
        echo "❌ Refresh button missing"
        FAIL=$((FAIL + 1))
    fi
else
    FAIL=$((FAIL + 3))
fi

echo ""

# Check JavaScript log aggregation logic
if [ -f "${DASHBOARD_DIR}/dashboard.js" ]; then
    if grep -q "loadLogs" "${DASHBOARD_DIR}/dashboard.js"; then
        echo "✅ Log loading function present"
        PASS=$((PASS + 1))
    else
        echo "❌ Log loading function missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "updateLogViewer" "${DASHBOARD_DIR}/dashboard.js"; then
        echo "✅ Log viewer update function present"
        PASS=$((PASS + 1))
    else
        echo "❌ Log viewer update function missing"
        FAIL=$((FAIL + 1))
    fi
else
    echo "❌ Dashboard JavaScript not found"
    FAIL=$((FAIL + 2))
fi

echo ""

# Test 18.4: Visual Differentiation Capability
echo "Test 18.4: Visual Differentiation Capability"
echo "=========================================="
echo ""

echo "Checking visual differentiation elements..."
if [ -f "${DASHBOARD_DIR}/index.html" ]; then
    if grep -q "status-pass" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ PASS status styling present"
        PASS=$((PASS + 1))
    else
        echo "❌ PASS status styling missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "status-fail" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ FAIL status styling present"
        PASS=$((PASS + 1))
    else
        echo "❌ FAIL status styling missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "status-unknown" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ UNKNOWN status styling present"
        PASS=$((PASS + 1))
    else
        echo "❌ UNKNOWN status styling missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "status-warning" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ WARNING status styling present"
        PASS=$((PASS + 1))
    else
        echo "❌ WARNING status styling missing"
        FAIL=$((FAIL + 1))
    fi
else
    FAIL=$((FAIL + 4))
fi

echo ""

# Check JavaScript visual differentiation logic
if [ -f "${DASHBOARD_DIR}/dashboard.js" ]; then
    if grep -q "log-line marker" "${DASHBOARD_DIR}/dashboard.js"; then
        echo "✅ Marker highlighting logic present"
        PASS=$((PASS + 1))
    else
        echo "❌ Marker highlighting logic missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "log-line error" "${DASHBOARD_DIR}/dashboard.js"; then
        echo "✅ Error highlighting logic present"
        PASS=$((PASS + 1))
    else
        echo "❌ Error highlighting logic missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "log-line warning" "${DASHBOARD_DIR}/dashboard.js"; then
        echo "✅ Warning highlighting logic present"
        PASS=$((PASS + 1))
    else
        echo "❌ Warning highlighting logic missing"
        FAIL=$((FAIL + 1))
    fi
else
    FAIL=$((FAIL + 3))
fi

echo ""

# Test 18.5: Execution Context Visibility
echo "Test 18.5: Execution Context Visibility"
echo "=========================================="
echo ""

echo "Checking execution context elements..."
if [ -f "${DASHBOARD_DIR}/index.html" ]; then
    if grep -q "id=\"contextTimestamp\"" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ Timestamp element present"
        PASS=$((PASS + 1))
    else
        echo "❌ Timestamp element missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "id=\"contextSource\"" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ Source element present"
        PASS=$((PASS + 1))
    else
        echo "❌ Source element missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "id=\"contextGitSha\"" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ Git SHA element present"
        PASS=$((PASS + 1))
    else
        echo "❌ Git SHA element missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "id=\"contextDeterministic\"" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ Deterministic flag element present"
        PASS=$((PASS + 1))
    else
        echo "❌ Deterministic flag element missing"
        FAIL=$((FAIL + 1))
    fi
else
    FAIL=$((FAIL + 4))
fi

echo ""

# Check JavaScript context update logic
if [ -f "${DASHBOARD_DIR}/dashboard.js" ]; then
    if grep -q "updateContextCard" "${DASHBOARD_DIR}/dashboard.js"; then
        echo "✅ Context update function present"
        PASS=$((PASS + 1))
    else
        echo "❌ Context update function missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "loadMetadata" "${DASHBOARD_DIR}/dashboard.js"; then
        echo "✅ Metadata loading function present"
        PASS=$((PASS + 1))
    else
        echo "❌ Metadata loading function missing"
        FAIL=$((FAIL + 1))
    fi
else
    FAIL=$((FAIL + 2))
fi

echo ""

# Test Constitutional Compliance
echo "Constitutional Compliance Checks"
echo "=========================================="
echo ""

echo "Checking read-only observer guarantee..."
if [ -f "${DASHBOARD_DIR}/index.html" ]; then
    if grep -q "Read-Only" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ Read-only declaration present"
        PASS=$((PASS + 1))
    else
        echo "❌ Read-only declaration missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "No Decision Authority" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ Non-authority declaration present"
        PASS=$((PASS + 1))
    else
        echo "❌ Non-authority declaration missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "Constitutional Compliance" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ Constitutional compliance section present"
        PASS=$((PASS + 1))
    else
        echo "❌ Constitutional compliance section missing"
        FAIL=$((FAIL + 1))
    fi
else
    FAIL=$((FAIL + 3))
fi

echo ""

# Check developer attribution
echo "Checking developer attribution..."
if [ -f "${DASHBOARD_DIR}/index.html" ]; then
    if grep -q "Kenan AY" "${DASHBOARD_DIR}/index.html"; then
        echo "✅ Developer attribution present in HTML"
        PASS=$((PASS + 1))
    else
        echo "❌ Developer attribution missing in HTML"
        FAIL=$((FAIL + 1))
    fi
else
    FAIL=$((FAIL + 1))
fi

if [ -f "${DASHBOARD_DIR}/dashboard.js" ]; then
    if grep -q "Kenan AY" "${DASHBOARD_DIR}/dashboard.js"; then
        echo "✅ Developer attribution present in JS"
        PASS=$((PASS + 1))
    else
        echo "❌ Developer attribution missing in JS"
        FAIL=$((FAIL + 1))
    fi
else
    FAIL=$((FAIL + 1))
fi

if [ -f "${DASHBOARD_DIR}/README.md" ]; then
    if grep -q "Kenan AY" "${DASHBOARD_DIR}/README.md"; then
        echo "✅ Developer attribution present in README"
        PASS=$((PASS + 1))
    else
        echo "❌ Developer attribution missing in README"
        FAIL=$((FAIL + 1))
    fi
else
    FAIL=$((FAIL + 1))
fi

echo ""

# Test File Structure
echo "File Structure Checks"
echo "=========================================="
echo ""

echo "Checking required files..."
if [ -f "${DASHBOARD_DIR}/index.html" ]; then
    echo "✅ index.html exists"
    PASS=$((PASS + 1))
else
    echo "❌ index.html missing"
    FAIL=$((FAIL + 1))
fi

if [ -f "${DASHBOARD_DIR}/dashboard.js" ]; then
    echo "✅ dashboard.js exists"
    PASS=$((PASS + 1))
else
    echo "❌ dashboard.js missing"
    FAIL=$((FAIL + 1))
fi

if [ -f "${DASHBOARD_DIR}/README.md" ]; then
    echo "✅ README.md exists"
    PASS=$((PASS + 1))
else
    echo "❌ README.md missing"
    FAIL=$((FAIL + 1))
fi

if [ -f "${DASHBOARD_DIR}/serve.sh" ]; then
    echo "✅ serve.sh exists"
    PASS=$((PASS + 1))
else
    echo "❌ serve.sh missing"
    FAIL=$((FAIL + 1))
fi

if [ -x "${DASHBOARD_DIR}/serve.sh" ]; then
    echo "✅ serve.sh is executable"
    PASS=$((PASS + 1))
else
    echo "❌ serve.sh is not executable"
    FAIL=$((FAIL + 1))
fi

echo ""

# Test Evidence Schema Compliance
echo "Evidence Schema Compliance"
echo "=========================================="
echo ""

echo "Checking evidence artifact references..."
if [ -f "${DASHBOARD_DIR}/dashboard.js" ]; then
    if grep -q "summary.json" "${DASHBOARD_DIR}/dashboard.js"; then
        echo "✅ summary.json reference present"
        PASS=$((PASS + 1))
    else
        echo "❌ summary.json reference missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "markers.json" "${DASHBOARD_DIR}/dashboard.js"; then
        echo "✅ markers.json reference present"
        PASS=$((PASS + 1))
    else
        echo "❌ markers.json reference missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "perf.json" "${DASHBOARD_DIR}/dashboard.js"; then
        echo "✅ perf.json reference present"
        PASS=$((PASS + 1))
    else
        echo "❌ perf.json reference missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "run.json" "${DASHBOARD_DIR}/dashboard.js"; then
        echo "✅ meta/run.json reference present"
        PASS=$((PASS + 1))
    else
        echo "❌ meta/run.json reference missing"
        FAIL=$((FAIL + 1))
    fi
    
    if grep -q "boot.log" "${DASHBOARD_DIR}/dashboard.js"; then
        echo "✅ boot.log reference present"
        PASS=$((PASS + 1))
    else
        echo "❌ boot.log reference missing"
        FAIL=$((FAIL + 1))
    fi
else
    FAIL=$((FAIL + 5))
fi

echo ""

# Summary
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo ""
echo "PASS: ${PASS}"
echo "FAIL: ${FAIL}"
echo ""

if [ ${FAIL} -eq 0 ]; then
    echo "✅ All tests passed"
    echo ""
    echo "Task 18 Observability Dashboard: COMPLETE"
    echo ""
    echo "All 5 capabilities verified:"
    echo "  ✓ 18.1 Status monitoring capability"
    echo "  ✓ 18.2 Performance observability capability"
    echo "  ✓ 18.3 Log aggregation capability"
    echo "  ✓ 18.4 Visual differentiation capability"
    echo "  ✓ 18.5 Execution context visibility"
    echo ""
    exit 0
else
    echo "❌ Some tests failed"
    echo ""
    exit 1
fi
