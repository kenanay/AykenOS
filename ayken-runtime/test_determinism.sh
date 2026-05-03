#!/usr/bin/env bash
set -euo pipefail

echo "=== Determinism Test ==="
echo "Running program twice and comparing traces..."
echo

cargo run --quiet > /tmp/ayken_trace_1.txt
cargo run --quiet > /tmp/ayken_trace_2.txt

if diff -u /tmp/ayken_trace_1.txt /tmp/ayken_trace_2.txt; then
    echo
    echo "✅ Determinism PASS - Traces are identical"
    exit 0
else
    echo
    echo "❌ Determinism FAIL - Traces differ"
    exit 1
fi
