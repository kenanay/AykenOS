#!/usr/bin/env bash
set -euo pipefail

echo "=== Replay Integrity Test ==="
echo

cargo run --example test_replay --quiet

echo
echo "✅ Replay integrity PASS"
