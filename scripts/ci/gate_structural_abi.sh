#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_TOOLS="${ROOT}/tools/ci"
source "${CI_TOOLS}/lib.sh"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_structural_abi.sh --evidence-dir evidence/run-<id>/gates/structural-abi [--diff-range <git-range>]

Exit codes:
  0: pass
  2: structural ABI violations detected
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
DIFF_RANGE="${ABI_DIFF_RANGE:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --diff-range)
      DIFF_RANGE="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown arg: $1" >&2
      usage
      exit 3
      ;;
  esac
done

if [[ -z "${EVIDENCE_DIR}" ]]; then
  usage
  exit 3
fi

if ! command -v git >/dev/null 2>&1 || ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: required tools missing (git/python3)" >&2
  exit 3
fi

CC_BIN=""
for c in clang cc gcc; do
  if command -v "${c}" >/dev/null 2>&1; then
    CC_BIN="${c}"
    break
  fi
done
if [[ -z "${CC_BIN}" ]]; then
  echo "ERROR: no C compiler found (clang/cc/gcc)" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

ABI_BASELINE="${ROOT}/constitution/abi_mailbox.json"
VERSION_FILE="${ROOT}/constitution/version.json"
DUMP_TOOL_C="${ROOT}/tools/dump_abi_layout.c"
DUMP_TOOL_BIN="${EVIDENCE_DIR}/dump_abi_layout"
ABI_ACTUAL_JSON="${EVIDENCE_DIR}/abi.actual.json"
BUILD_LOG="${EVIDENCE_DIR}/build.log"
CHANGED_TXT="${EVIDENCE_DIR}/changed-files.txt"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
ABI_BASELINE_SHA="${EVIDENCE_DIR}/abi_baseline.sha256"

: > "${BUILD_LOG}"
: > "${CHANGED_TXT}"
: > "${VIOLATIONS_TXT}"
: > "${META_TXT}"
: > "${ABI_BASELINE_SHA}"

resolve_diff_range() {
  if [[ -n "${DIFF_RANGE}" ]]; then
    echo "${DIFF_RANGE}"
    return 0
  fi
  if git -C "${ROOT}" rev-parse --verify origin/main >/dev/null 2>&1; then
    local mb
    mb="$(git -C "${ROOT}" merge-base origin/main HEAD 2>/dev/null || true)"
    if [[ -n "${mb}" ]]; then
      echo "${mb}...HEAD"
      return 0
    fi
  fi
  if git -C "${ROOT}" rev-parse --verify HEAD~1 >/dev/null 2>&1; then
    echo "HEAD~1...HEAD"
    return 0
  fi
  echo "HEAD"
}

DIFF_RANGE_VAL="$(resolve_diff_range)"
BASE_REV=""
if [[ "${DIFF_RANGE_VAL}" == *"..."* ]]; then
  BASE_REV="${DIFF_RANGE_VAL%%...*}"
elif [[ "${DIFF_RANGE_VAL}" == *".."* ]]; then
  BASE_REV="${DIFF_RANGE_VAL%%..*}"
fi

if ! git -C "${ROOT}" diff --name-only --diff-filter=ACMRDT "${DIFF_RANGE_VAL}" > "${CHANGED_TXT}" 2>/dev/null; then
  git -C "${ROOT}" show --pretty="" --name-only HEAD > "${CHANGED_TXT}" 2>/dev/null || true
fi

COMPILE_OK=0
DUMP_OK=0
if [[ -f "${DUMP_TOOL_C}" ]]; then
  if "${CC_BIN}" -std=c11 -Wall -Wextra -Werror "${DUMP_TOOL_C}" -o "${DUMP_TOOL_BIN}" >> "${BUILD_LOG}" 2>&1; then
    COMPILE_OK=1
    if "${DUMP_TOOL_BIN}" > "${ABI_ACTUAL_JSON}" 2>> "${BUILD_LOG}"; then
      DUMP_OK=1
    else
      echo "abi_dump_execute:failed" >> "${VIOLATIONS_TXT}"
    fi
  else
    echo "abi_dump_compile:failed" >> "${VIOLATIONS_TXT}"
  fi
else
  echo "missing_file:${DUMP_TOOL_C}" >> "${VIOLATIONS_TXT}"
fi

if [[ -f "${ABI_BASELINE}" ]]; then
  sha256_file "${ABI_BASELINE}" > "${ABI_BASELINE_SHA}"
else
  echo "MISSING" > "${ABI_BASELINE_SHA}"
fi

NOW="$(ci_now_utc)"
GIT_SHA="$(git -C "${ROOT}" rev-parse HEAD 2>/dev/null || echo NO_GIT)"

ROOT_ENV="${ROOT}" \
ABI_BASELINE_ENV="${ABI_BASELINE}" \
VERSION_FILE_ENV="${VERSION_FILE}" \
ABI_ACTUAL_JSON_ENV="${ABI_ACTUAL_JSON}" \
CHANGED_TXT_ENV="${CHANGED_TXT}" \
VIOLATIONS_TXT_ENV="${VIOLATIONS_TXT}" \
META_TXT_ENV="${META_TXT}" \
REPORT_JSON_ENV="${REPORT_JSON}" \
ABI_BASELINE_SHA_ENV="${ABI_BASELINE_SHA}" \
DIFF_RANGE_ENV="${DIFF_RANGE_VAL}" \
BASE_REV_ENV="${BASE_REV}" \
CC_BIN_ENV="${CC_BIN}" \
COMPILE_OK_ENV="${COMPILE_OK}" \
DUMP_OK_ENV="${DUMP_OK}" \
NOW_ENV="${NOW}" \
GIT_SHA_ENV="${GIT_SHA}" \
python3 - <<'PY'
import json
import os
import re
import subprocess
from pathlib import Path

ROOT = Path(os.environ["ROOT_ENV"])
ABI_BASELINE = Path(os.environ["ABI_BASELINE_ENV"])
VERSION_FILE = Path(os.environ["VERSION_FILE_ENV"])
ABI_ACTUAL = Path(os.environ["ABI_ACTUAL_JSON_ENV"])
CHANGED_TXT = Path(os.environ["CHANGED_TXT_ENV"])
VIOLATIONS_TXT = Path(os.environ["VIOLATIONS_TXT_ENV"])
META_TXT = Path(os.environ["META_TXT_ENV"])
REPORT_JSON = Path(os.environ["REPORT_JSON_ENV"])
ABI_BASELINE_SHA = Path(os.environ["ABI_BASELINE_SHA_ENV"])
DIFF_RANGE = os.environ["DIFF_RANGE_ENV"]
BASE_REV = os.environ["BASE_REV_ENV"]
CC_BIN = os.environ["CC_BIN_ENV"]
COMPILE_OK = os.environ.get("COMPILE_OK_ENV", "0") == "1"
DUMP_OK = os.environ.get("DUMP_OK_ENV", "0") == "1"
NOW = os.environ["NOW_ENV"]
GIT_SHA = os.environ["GIT_SHA_ENV"]

ABI_REL = "constitution/abi_mailbox.json"
VERSION_REL = "constitution/version.json"
violations = []

def add(v):
    violations.append(v)

def parse_semver(raw):
    if not isinstance(raw, str):
        return None
    m = re.fullmatch(r"(\d+)\.(\d+)\.(\d+)", raw.strip())
    return tuple(int(x) for x in m.groups()) if m else None

def semver_bump(old, new):
    if old is None or new is None:
        return "none"
    if new < old:
        return "invalid"
    if new[0] > old[0]:
        return "major"
    if new[1] > old[1]:
        return "minor"
    if new[2] > old[2]:
        return "patch"
    return "none"

def bump_rank(name):
    return {"none": 0, "patch": 1, "minor": 2, "major": 3, "invalid": -1}.get(name, -1)

def load_json(path, miss, bad):
    if not path.exists():
        add(f"{miss}:{path}")
        return None
    try:
        return json.loads(path.read_text(encoding="utf-8", errors="replace"))
    except Exception as exc:
        add(f"{bad}:{path}:{type(exc).__name__}")
        return None

def git_show_json(base_rev, rel):
    if not base_rev:
        return None
    p = subprocess.run(["git", "-C", str(ROOT), "show", f"{base_rev}:{rel}"], text=True, capture_output=True)
    if p.returncode != 0:
        return None
    try:
        return json.loads(p.stdout)
    except Exception:
        return None

def classify_abi_delta(old_doc, new_doc):
    if not isinstance(old_doc, dict) or not isinstance(new_doc, dict):
        return "none"
    old_mb = old_doc.get("mailbox")
    new_mb = new_doc.get("mailbox")
    if not isinstance(old_mb, dict) or not isinstance(new_mb, dict):
        return "major"
    if old_mb.get("size") != new_mb.get("size") or old_mb.get("alignment") != new_mb.get("alignment"):
        return "major"
    old_fields = old_mb.get("fields")
    new_fields = new_mb.get("fields")
    if not isinstance(old_fields, dict) or not isinstance(new_fields, dict):
        return "major"
    old_names = list(old_fields.keys())
    new_names = list(new_fields.keys())
    for name in old_names:
        if name not in new_fields:
            return "major"
        for key in ("type", "offset", "size"):
            if old_fields.get(name, {}).get(key) != new_fields.get(name, {}).get(key):
                return "major"
    if new_names[:len(old_names)] != old_names:
        return "major"
    if len(new_names) > len(old_names):
        return "minor"
    return "none"

if VIOLATIONS_TXT.exists():
    violations.extend([ln.strip() for ln in VIOLATIONS_TXT.read_text(encoding="utf-8", errors="replace").splitlines() if ln.strip()])

abi_baseline = load_json(ABI_BASELINE, "missing_file", "abi_baseline_parse_error")
version_doc = load_json(VERSION_FILE, "missing_file", "version_parse_error")
abi_actual = load_json(ABI_ACTUAL, "missing_file", "abi_actual_parse_error") if DUMP_OK else None

abi_contract_ok = True
abi_layout_ok = True
versioning_ok = True

if not COMPILE_OK:
    add("abi_dump_compile:failed")
if COMPILE_OK and not DUMP_OK:
    add("abi_dump_execute:failed")

if isinstance(abi_baseline, dict):
    rules = abi_baseline.get("rules")
    if not isinstance(rules, dict):
        add("abi_rules_missing")
        abi_contract_ok = False
    else:
        for key in ("size_change", "field_offset_change", "field_add", "field_remove"):
            if key not in rules:
                add(f"abi_rules_missing_key:{key}")
                abi_contract_ok = False
else:
    abi_contract_ok = False

if isinstance(abi_baseline, dict) and isinstance(abi_actual, dict):
    em = abi_baseline.get("mailbox")
    am = abi_actual.get("mailbox")
    if not isinstance(em, dict) or not isinstance(am, dict):
        add("abi_mailbox_missing")
        abi_layout_ok = False
    else:
        if em.get("size") != am.get("size"):
            add(f"abi_size_mismatch:expected={em.get('size')}:actual={am.get('size')}")
            abi_layout_ok = False
        if em.get("alignment") != am.get("alignment"):
            add(f"abi_alignment_mismatch:expected={em.get('alignment')}:actual={am.get('alignment')}")
            abi_layout_ok = False
        ef = em.get("fields")
        af = am.get("fields")
        if not isinstance(ef, dict) or not isinstance(af, dict):
            add("abi_fields_missing")
            abi_layout_ok = False
        else:
            for name in ef:
                if name not in af:
                    add(f"abi_field_missing:{name}")
                    abi_layout_ok = False
                    continue
                for key in ("type", "offset", "size"):
                    if ef[name].get(key) != af[name].get(key):
                        add(f"abi_field_mismatch:{name}.{key}")
                        abi_layout_ok = False
            for name in af:
                if name not in ef:
                    add(f"abi_field_unexpected:{name}")
                    abi_layout_ok = False
else:
    abi_layout_ok = False

changed_files = []
if CHANGED_TXT.exists():
    changed_files = [ln.strip() for ln in CHANGED_TXT.read_text(encoding="utf-8", errors="replace").splitlines() if ln.strip()]
changed_set = set(changed_files)

current_version = parse_semver(version_doc.get("constitution_version")) if isinstance(version_doc, dict) else None
if current_version is None:
    add("constitution_version_invalid")
    versioning_ok = False

if ABI_REL in changed_set and VERSION_REL not in changed_set:
    add("constitution_version_bump_required")
    versioning_ok = False

old_version_doc = git_show_json(BASE_REV, VERSION_REL)
old_abi_doc = git_show_json(BASE_REV, ABI_REL)
old_version = parse_semver(old_version_doc.get("constitution_version")) if isinstance(old_version_doc, dict) else None

required_bump = "none"
reasons = []
if ABI_REL in changed_set and old_abi_doc is not None and isinstance(abi_baseline, dict):
    delta = classify_abi_delta(old_abi_doc, abi_baseline)
    if bump_rank(delta) > bump_rank(required_bump):
        required_bump = delta
    if delta != "none":
        reasons.append(f"abi:{delta}")

actual_bump = semver_bump(old_version, current_version)
if actual_bump == "invalid":
    add("constitution_version_downgrade")
    versioning_ok = False
if required_bump == "major" and actual_bump != "major":
    add(f"constitution_version_major_required:{','.join(reasons) or 'major'}")
    versioning_ok = False
elif required_bump == "minor" and actual_bump not in {"minor", "major"}:
    add(f"constitution_version_minor_required:{','.join(reasons) or 'minor'}")
    versioning_ok = False

violations = sorted(set(violations))
VIOLATIONS_TXT.write_text("\n".join(violations) + ("\n" if violations else ""), encoding="utf-8")

sha_val = ABI_BASELINE_SHA.read_text(encoding="utf-8", errors="replace").strip() if ABI_BASELINE_SHA.exists() else "MISSING"
meta = {
    "time_utc": NOW,
    "git_sha": GIT_SHA,
    "diff_range": DIFF_RANGE,
    "base_rev": BASE_REV,
    "compiler": CC_BIN,
    "compile_ok": "1" if COMPILE_OK else "0",
    "dump_ok": "1" if DUMP_OK else "0",
    "constitution_version": version_doc.get("constitution_version") if isinstance(version_doc, dict) else "UNKNOWN",
    "required_bump": required_bump,
    "actual_bump": actual_bump,
    "abi_baseline_sha256": sha_val if sha_val else "MISSING",
    "changed_files_count": len(changed_files),
    "violations_count": len(violations),
}
META_TXT.write_text("".join(f"{k}={v}\n" for k, v in meta.items()), encoding="utf-8")

report = {
    "gate": "structural-abi",
    "tier": "tier-1-permanent",
    "verdict": "PASS" if not violations else "FAIL",
    "violations_count": len(violations),
    "meta": meta,
    "checks": {
        "abi_contract_rules": "PASS" if abi_contract_ok else "FAIL",
        "abi_layout_lock": "PASS" if abi_layout_ok else "FAIL",
        "versioning_policy": "PASS" if versioning_ok else "FAIL",
    },
    "changed_files": changed_files,
    "violations": violations,
}
REPORT_JSON.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
raise SystemExit(0 if not violations else 2)
PY

VIOLATION_COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
if [[ "${VIOLATION_COUNT}" -gt 0 ]]; then
  echo "structural-abi: FAIL (${VIOLATION_COUNT} violations)"
  echo "See: ${VIOLATIONS_TXT}"
  exit 2
fi

echo "structural-abi: PASS"
exit 0
