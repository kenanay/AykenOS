#!/usr/bin/env bash
# AykenOS Dev Loop Observability Dashboard Server
#
# Purpose: Serve the observability dashboard via HTTP
# Authority: ZERO - purely serves static files
#
# Maintainer: Kenan AY — System Architect

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PORT="${1:-8080}"

echo "=========================================="
echo "AykenOS Observability Dashboard"
echo "=========================================="
echo ""
echo "Starting HTTP server on port ${PORT}..."
echo ""
echo "Dashboard URL: http://localhost:${PORT}"
echo ""
echo "Press Ctrl+C to stop"
echo ""
echo "=========================================="
echo ""

cd "${SCRIPT_DIR}"

# Check if Python 3 is available
if command -v python3 &> /dev/null; then
    python3 -m http.server "${PORT}"
elif command -v python &> /dev/null; then
    python -m http.server "${PORT}"
else
    echo "Error: Python not found"
    echo "Please install Python 3 to run the dashboard server"
    exit 1
fi
