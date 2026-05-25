#!/usr/bin/env bash
set -euo pipefail

SPEC_DIR="${SPEC_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}"
BASELINE="${SPEC_DIR}/ORIGINAL_BASELINE.md"
METADATA="${SPEC_DIR}/.metadata.yml"

echo "== CI GATE SPEC VALIDATION =="
echo "spec_dir: ${SPEC_DIR}"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

hash_file() {
  local path="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$path" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$path" | awk '{print $1}'
  elif command -v openssl >/dev/null 2>&1; then
    openssl dgst -sha256 "$path" | awk '{print $NF}'
  else
    fail "no SHA256 tool available"
  fi
}

[ -f "${BASELINE}" ] || fail "missing ORIGINAL_BASELINE.md"
[ -f "${METADATA}" ] || fail "missing .metadata.yml"
grep -Eq '^validation_level:[[:space:]]*3\b' "${METADATA}" || fail "validation_level must be 3"

for file in requirements.md design.md tasks.md; do
  path="${SPEC_DIR}/${file}"
  [ -f "${path}" ] || fail "missing ${file}"
  expected="$(awk -v f="${file}" '$2 == f {print $1}' "${BASELINE}")"
  [ -n "${expected}" ] || fail "missing baseline hash for ${file}"
  actual="$(hash_file "${path}")"
  if [ "${actual}" != "${expected}" ]; then
    fail "baseline drift for ${file}: expected=${expected} actual=${actual}"
  fi
  echo "hash ok: ${file}"
done

echo "PASS: ayken-vcp-execution-binding spec baseline intact"
