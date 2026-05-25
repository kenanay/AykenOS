#!/usr/bin/env bash
set -euo pipefail

echo "=== Determinism 100-run Test ==="
echo "Running reference trace..."

cargo run --quiet > /tmp/ayken_trace_ref.txt

echo "Running 100 iterations..."

for i in $(seq 1 100); do
  cargo run --quiet > "/tmp/ayken_trace_$i.txt"
  if ! diff -u /tmp/ayken_trace_ref.txt "/tmp/ayken_trace_$i.txt" >/dev/null; then
    echo "❌ FAIL at iteration $i"
    diff -u /tmp/ayken_trace_ref.txt "/tmp/ayken_trace_$i.txt"
    exit 1
  fi
  
  if [ $((i % 10)) -eq 0 ]; then
    echo "  ✓ $i runs passed"
  fi
done

echo
echo "✅ Determinism PASS - 100 runs byte-identical"
