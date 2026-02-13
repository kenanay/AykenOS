#!/usr/bin/env bash
set -euo pipefail

ci_now_utc() {
  date -u +"%Y-%m-%dT%H:%M:%SZ"
}

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    echo "NO_SHA_TOOL"
  fi
}
