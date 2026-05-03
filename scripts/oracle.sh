#!/bin/bash
# Oracle Script: Deterministic Validation Check
# Returns: 0 (PASS) or non-zero (FAIL)
#
# Purpose: Provides a deterministic validation check for git bisect.
# This script is used by find_regression.sh to automatically identify
# which commit broke the system.
#
# Usage:
#   ./scripts/oracle.sh
#
# Exit Codes:
#   0 = PASS (system validates successfully)
#   1 = FAIL (validation failed)
#   2 = ERROR (script error, not validation failure)

set -euo pipefail

# Run smoke test (fast validation)
# For more thorough validation, change to: ./scripts/dev_loop.sh contract
./scripts/dev_loop.sh smoke >/dev/null 2>&1
