#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_TOOLS="${ROOT}/tools/ci"
source "${CI_TOOLS}/lib.sh"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_runtime_marker_contract.sh --evidence-dir evidence/run-<id>/gates/runtime-marker-contract [--diff-range <git-range>] [--enforce|--no-enforce]

Exit codes:
  0: pass/skip
  2: runtime marker contract violations detected
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
DIFF_RANGE="${ABI_DIFF_RANGE:-}"
ENFORCE="${RUNTIME_MARKER_CONTRACT_ENFORCE:-1}"

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
    --enforce)
      ENFORCE=1
      shift 1
      ;;
    --no-enforce)
      ENFORCE=0
      shift 1
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
if [[ "${ENFORCE}" != "0" && "${ENFORCE}" != "1" ]]; then
  echo "ERROR: ENFORCE must be 0 or 1" >&2
  exit 3
fi
if ! command -v git >/dev/null 2>&1 || ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: required tools missing (git/python3)" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

MARKER_REGISTRY="${ROOT}/constitution/runtime_markers.json"
SCHEMA_V1="${ROOT}/constitution/markers_schema_v1.json"
VERSION_FILE="${ROOT}/constitution/version.json"
CHANGED_TXT="${EVIDENCE_DIR}/changed-files.txt"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
MARKER_REGISTRY_SHA="${EVIDENCE_DIR}/marker_registry.sha256"
SCHEMA_V1_SHA="${EVIDENCE_DIR}/marker_schema_v1.sha256"

: > "${CHANGED_TXT}"
: > "${VIOLATIONS_TXT}"
: > "${META_TXT}"
: > "${MARKER_REGISTRY_SHA}"
: > "${SCHEMA_V1_SHA}"

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

if [[ -f "${MARKER_REGISTRY}" ]]; then
  sha256_file "${MARKER_REGISTRY}" > "${MARKER_REGISTRY_SHA}"
else
  echo "MISSING" > "${MARKER_REGISTRY_SHA}"
fi
if [[ -f "${SCHEMA_V1}" ]]; then
  sha256_file "${SCHEMA_V1}" > "${SCHEMA_V1_SHA}"
else
  echo "MISSING" > "${SCHEMA_V1_SHA}"
fi

NOW="$(ci_now_utc)"
GIT_SHA="$(git -C "${ROOT}" rev-parse HEAD 2>/dev/null || echo NO_GIT)"

if [[ "${ENFORCE}" == "0" ]]; then
  ROOT_ENV="${ROOT}" \
  CHANGED_TXT_ENV="${CHANGED_TXT}" \
  META_TXT_ENV="${META_TXT}" \
  REPORT_JSON_ENV="${REPORT_JSON}" \
  MARKER_REGISTRY_SHA_ENV="${MARKER_REGISTRY_SHA}" \
  SCHEMA_V1_SHA_ENV="${SCHEMA_V1_SHA}" \
  DIFF_RANGE_ENV="${DIFF_RANGE_VAL}" \
  BASE_REV_ENV="${BASE_REV}" \
  NOW_ENV="${NOW}" \
  GIT_SHA_ENV="${GIT_SHA}" \
  python3 - <<'PY'
import json
import os
from pathlib import Path

CHANGED_TXT = Path(os.environ["CHANGED_TXT_ENV"])
META_TXT = Path(os.environ["META_TXT_ENV"])
REPORT_JSON = Path(os.environ["REPORT_JSON_ENV"])
MARKER_REGISTRY_SHA = Path(os.environ["MARKER_REGISTRY_SHA_ENV"])
SCHEMA_V1_SHA = Path(os.environ["SCHEMA_V1_SHA_ENV"])
changed = [ln.strip() for ln in CHANGED_TXT.read_text(encoding="utf-8", errors="replace").splitlines() if ln.strip()] if CHANGED_TXT.exists() else []
sha_val = MARKER_REGISTRY_SHA.read_text(encoding="utf-8", errors="replace").strip() if MARKER_REGISTRY_SHA.exists() else "MISSING"
schema_sha_val = SCHEMA_V1_SHA.read_text(encoding="utf-8", errors="replace").strip() if SCHEMA_V1_SHA.exists() else "MISSING"
meta = {
    "time_utc": os.environ["NOW_ENV"],
    "git_sha": os.environ["GIT_SHA_ENV"],
    "diff_range": os.environ["DIFF_RANGE_ENV"],
    "base_rev": os.environ["BASE_REV_ENV"],
    "enforced": "0",
    "scope": "tier-2-phase-scoped",
    "marker_registry_sha256": sha_val if sha_val else "MISSING",
    "marker_schema_v1_sha256": schema_sha_val if schema_sha_val else "MISSING",
    "changed_files_count": len(changed),
    "violations_count": 0,
}
META_TXT.write_text("".join(f"{k}={v}\n" for k, v in meta.items()), encoding="utf-8")
report = {
    "gate": "runtime-marker-contract",
    "tier": "tier-2-phase-scoped",
    "verdict": "SKIP",
    "reason": "runtime marker contract enforcement disabled",
    "violations_count": 0,
    "meta": meta,
    "checks": {
        "marker_registry_contract": "SKIP",
        "marker_schema_v1_contract": "SKIP",
        "marker_schema_v1_breaking_guard": "SKIP",
        "schema_v2_major_bump_guard": "SKIP",
        "source_marker_anchor": "SKIP",
        "versioning_policy": "SKIP",
    },
    "changed_files": changed,
    "violations": [],
}
REPORT_JSON.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
  echo "runtime-marker-contract: SKIP (enforcement disabled)"
  exit 0
fi

ROOT_ENV="${ROOT}" \
MARKER_REGISTRY_ENV="${MARKER_REGISTRY}" \
SCHEMA_V1_ENV="${SCHEMA_V1}" \
VERSION_FILE_ENV="${VERSION_FILE}" \
CHANGED_TXT_ENV="${CHANGED_TXT}" \
VIOLATIONS_TXT_ENV="${VIOLATIONS_TXT}" \
META_TXT_ENV="${META_TXT}" \
REPORT_JSON_ENV="${REPORT_JSON}" \
MARKER_REGISTRY_SHA_ENV="${MARKER_REGISTRY_SHA}" \
SCHEMA_V1_SHA_ENV="${SCHEMA_V1_SHA}" \
DIFF_RANGE_ENV="${DIFF_RANGE_VAL}" \
BASE_REV_ENV="${BASE_REV}" \
NOW_ENV="${NOW}" \
GIT_SHA_ENV="${GIT_SHA}" \
python3 - <<'PY'
import json
import os
import re
import subprocess
from pathlib import Path

ROOT = Path(os.environ["ROOT_ENV"])
MARKER_REGISTRY = Path(os.environ["MARKER_REGISTRY_ENV"])
SCHEMA_V1 = Path(os.environ["SCHEMA_V1_ENV"])
VERSION_FILE = Path(os.environ["VERSION_FILE_ENV"])
CHANGED_TXT = Path(os.environ["CHANGED_TXT_ENV"])
VIOLATIONS_TXT = Path(os.environ["VIOLATIONS_TXT_ENV"])
META_TXT = Path(os.environ["META_TXT_ENV"])
REPORT_JSON = Path(os.environ["REPORT_JSON_ENV"])
MARKER_REGISTRY_SHA = Path(os.environ["MARKER_REGISTRY_SHA_ENV"])
SCHEMA_V1_SHA = Path(os.environ["SCHEMA_V1_SHA_ENV"])
DIFF_RANGE = os.environ["DIFF_RANGE_ENV"]
BASE_REV = os.environ["BASE_REV_ENV"]
NOW = os.environ["NOW_ENV"]
GIT_SHA = os.environ["GIT_SHA_ENV"]

MARKER_REL = "constitution/runtime_markers.json"
SCHEMA_V1_REL = "constitution/markers_schema_v1.json"
SCHEMA_V2_REL = "constitution/markers_schema_v2.json"
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

def marker_map(doc):
    out = {}
    if not isinstance(doc, dict):
        return out
    rows = doc.get("runtime_markers")
    if not isinstance(rows, list):
        return out
    for row in rows:
        if not isinstance(row, dict):
            continue
        name = row.get("name")
        if isinstance(name, str) and name:
            out[name] = {"marker": row.get("marker"), "pattern": row.get("pattern"), "fields": row.get("fields", {})}
    return out

def classify_delta(old_doc, new_doc):
    old_map = marker_map(old_doc)
    new_map = marker_map(new_doc)
    if not old_map or not new_map:
        return "none"
    old_names = set(old_map.keys())
    new_names = set(new_map.keys())
    if old_names - new_names:
        return "major"
    for name in sorted(old_names & new_names):
        if old_map[name] != new_map[name]:
            return "major"
    if new_names - old_names:
        return "minor"
    return "none"

def normalize_profile_contract(profile):
    if not isinstance(profile, dict):
        return None
    markers = profile.get("markers")
    if not isinstance(markers, dict):
        return None
    norm_markers = {}
    for name, row in markers.items():
        if not isinstance(name, str) or not name or not isinstance(row, dict):
            return None
        token = row.get("token")
        pattern = row.get("pattern")
        required_count = row.get("required_count")
        if not isinstance(required_count, int) or required_count < 0:
            return None
        if token is None and pattern is None:
            return None
        if token is not None and not isinstance(token, str):
            return None
        if pattern is not None and not isinstance(pattern, str):
            return None
        norm_markers[name] = {
            "token": token,
            "pattern": pattern,
            "required_count": required_count,
        }

    ordering = profile.get("ordering", [])
    if not isinstance(ordering, list):
        return None
    norm_ordering = []
    for pair in ordering:
        if not isinstance(pair, list) or len(pair) != 2:
            return None
        left, right = pair
        if not isinstance(left, str) or not isinstance(right, str):
            return None
        norm_ordering.append((left, right))

    flags = profile.get("profile_flags", {})
    if flags is None:
        flags = {}
    if not isinstance(flags, dict):
        return None

    return {
        "markers": norm_markers,
        "ordering": sorted(norm_ordering),
        "profile_flags": {k: flags[k] for k in sorted(flags.keys())},
    }

def normalize_schema_contract(doc):
    if not isinstance(doc, dict):
        return None
    profiles = doc.get("profiles")
    if not isinstance(profiles, dict):
        return None
    out = {}
    for profile_name, profile in profiles.items():
        if not isinstance(profile_name, str) or not profile_name:
            return None
        norm = normalize_profile_contract(profile)
        if norm is None:
            return None
        out[profile_name] = norm
    return out

def classify_schema_v1_delta(old_doc, new_doc):
    old_norm = normalize_schema_contract(old_doc)
    new_norm = normalize_schema_contract(new_doc)
    if old_norm is None or new_norm is None:
        return "none", []

    change = "none"
    reasons = []

    old_profiles = set(old_norm.keys())
    new_profiles = set(new_norm.keys())
    removed_profiles = sorted(old_profiles - new_profiles)
    added_profiles = sorted(new_profiles - old_profiles)

    if removed_profiles:
        return "breaking", [f"removed_profiles:{','.join(removed_profiles)}"]
    if added_profiles:
        change = "compatible"
        reasons.append(f"added_profiles:{','.join(added_profiles)}")

    for profile_name in sorted(old_profiles & new_profiles):
        old_profile = old_norm[profile_name]
        new_profile = new_norm[profile_name]

        if old_profile["ordering"] != new_profile["ordering"]:
            return "breaking", [f"ordering_changed:{profile_name}"]

        old_flags = old_profile["profile_flags"]
        new_flags = new_profile["profile_flags"]
        for key, old_value in old_flags.items():
            if key not in new_flags or new_flags[key] != old_value:
                return "breaking", [f"profile_flag_changed:{profile_name}:{key}"]
        added_flag_keys = sorted(set(new_flags.keys()) - set(old_flags.keys()))
        if added_flag_keys:
            if change == "none":
                change = "compatible"
            reasons.append(f"added_profile_flags:{profile_name}:{','.join(added_flag_keys)}")

        old_markers = old_profile["markers"]
        new_markers = new_profile["markers"]

        removed_markers = sorted(set(old_markers.keys()) - set(new_markers.keys()))
        if removed_markers:
            return "breaking", [f"removed_markers:{profile_name}:{','.join(removed_markers)}"]

        for marker_name in sorted(set(old_markers.keys()) & set(new_markers.keys())):
            if old_markers[marker_name] != new_markers[marker_name]:
                return "breaking", [f"marker_contract_changed:{profile_name}:{marker_name}"]

        added_markers = sorted(set(new_markers.keys()) - set(old_markers.keys()))
        if added_markers:
            for marker_name in added_markers:
                req = new_markers[marker_name].get("required_count")
                if req == 0:
                    if change == "none":
                        change = "compatible"
                    reasons.append(f"added_marker_optional:{profile_name}:{marker_name}")
                else:
                    return "breaking", [f"added_marker_required:{profile_name}:{marker_name}"]

    return change, reasons

marker_doc = load_json(MARKER_REGISTRY, "missing_file", "marker_registry_parse_error")
version_doc = load_json(VERSION_FILE, "missing_file", "version_parse_error")
marker_ok = True
schema_v1_contract_ok = True
schema_v1_breaking_guard_ok = True
source_ok = True
versioning_ok = True
report_schema_ok = True
marker_by_name = {}
compiled = {}

if isinstance(marker_doc, dict):
    schema = marker_doc.get("schema_version")
    if not isinstance(schema, str) or not schema:
        add("marker_schema_version_missing")
        marker_ok = False
    rules = marker_doc.get("rules")
    if not isinstance(rules, dict):
        add("marker_rules_missing")
        marker_ok = False
    else:
        for k in ("add_marker", "remove_marker", "modify_marker", "modify_marker_format"):
            if k not in rules:
                add(f"marker_rules_missing_key:{k}")
                marker_ok = False
    rows = marker_doc.get("runtime_markers")
    if not isinstance(rows, list):
        add("marker_registry_invalid:runtime_markers_must_be_array")
        marker_ok = False
        rows = []
    for i, row in enumerate(rows):
        if not isinstance(row, dict):
            add(f"marker_entry_invalid:index={i}")
            marker_ok = False
            continue
        name = row.get("name")
        marker = row.get("marker")
        pattern = row.get("pattern")
        fields = row.get("fields")
        if not isinstance(name, str) or not name:
            add(f"marker_entry_invalid_name:index={i}")
            marker_ok = False
            continue
        if name in marker_by_name:
            add(f"marker_duplicate_name:{name}")
            marker_ok = False
        marker_by_name[name] = row
        if not isinstance(marker, str) or not marker:
            add(f"marker_entry_invalid_marker:{name}")
            marker_ok = False
        if not isinstance(pattern, str) or not pattern:
            add(f"marker_entry_invalid_pattern:{name}")
            marker_ok = False
            continue
        if not isinstance(fields, dict):
            add(f"marker_entry_invalid_fields:{name}")
            marker_ok = False
        try:
            compiled[name] = re.compile(pattern)
        except re.error:
            add(f"marker_pattern_invalid_regex:{name}")
            marker_ok = False

    required = {
        "AYKEN_SCHED_MB_ACCEPT": r"^\[\[AYKEN_SCHED_MB_ACCEPT\]\] pid=[0-9]+ epoch=[0-9]+$",
        "AYKEN_SCHED_MB_REJECT": r"^\[\[AYKEN_SCHED_MB_REJECT\]\] reason=[0-9]+ epoch=[0-9]+ pid=[0-9]+$",
        "AYKEN_RING3_OK": r"^\[\[AYKEN_RING3_OK\]\]$",
        "R3OK_USER_TOKEN": r"^R3OK$",
        "AYKEN_SYSCALL_ENTER": r"^\[\[AYKEN_SYSCALL_ENTER\]\]$",
        "P10_SYSCALL_ENTER": r"^P10_SYSCALL_ENTER$",
        "AYKEN_SYSCALL_RETURN": r"^\[\[AYKEN_SYSCALL_RETURN\]\]$",
        "P10_SYSCALL_RETURN": r"^P10_SYSCALL_RETURN$",
        "P10_CAP_ENFORCED": r"^P10_CAP_ENFORCED$",
        "P10_RING3_USER_CODE": r"^P10_RING3_USER_CODE$",
    }
    for name, patt in required.items():
        row = marker_by_name.get(name)
        if row is None:
            add(f"marker_missing_required:{name}")
            marker_ok = False
            continue
        if row.get("pattern") != patt:
            add(f"marker_pattern_contract:{name}")
            marker_ok = False

    sample = {
        "AYKEN_SCHED_MB_ACCEPT": "[[AYKEN_SCHED_MB_ACCEPT]] pid=42 epoch=7",
        "AYKEN_SCHED_MB_REJECT": "[[AYKEN_SCHED_MB_REJECT]] reason=5 epoch=7 pid=42",
        "AYKEN_RING3_OK": "[[AYKEN_RING3_OK]]",
        "R3OK_USER_TOKEN": "R3OK",
        "AYKEN_SYSCALL_ENTER": "[[AYKEN_SYSCALL_ENTER]]",
        "P10_SYSCALL_ENTER": "P10_SYSCALL_ENTER",
        "AYKEN_SYSCALL_RETURN": "[[AYKEN_SYSCALL_RETURN]]",
        "P10_SYSCALL_RETURN": "P10_SYSCALL_RETURN",
        "P10_CAP_ENFORCED": "P10_CAP_ENFORCED",
        "P10_RING3_USER_CODE": "P10_RING3_USER_CODE",
    }
    for name, line in sample.items():
        rgx = compiled.get(name)
        if rgx and not rgx.fullmatch(line):
            add(f"marker_pattern_sample_mismatch:{name}")
            marker_ok = False

    anchors = [
        ("AYKEN_SCHED_MB_ACCEPT", ROOT / "kernel/sched/sched_mailbox.c", "[[AYKEN_SCHED_MB_ACCEPT]]"),
        ("AYKEN_SCHED_MB_REJECT", ROOT / "kernel/sched/sched_mailbox.c", "[[AYKEN_SCHED_MB_REJECT]]"),
        ("AYKEN_RING3_OK", ROOT / "kernel/sys/syscall_v2.c", "[[AYKEN_RING3_OK]]"),
        ("R3OK_USER_TOKEN", ROOT / "userspace/tests/gate3_ring3_sched_hint/main.c", "R3OK"),
        ("AYKEN_SYSCALL_ENTER", ROOT / "kernel/sys/syscall.c", "[[AYKEN_SYSCALL_ENTER]]"),
        ("P10_SYSCALL_ENTER", ROOT / "kernel/sys/syscall.c", "P10_SYSCALL_ENTER"),
        ("AYKEN_SYSCALL_RETURN", ROOT / "kernel/sys/syscall.c", "[[AYKEN_SYSCALL_RETURN]]"),
        ("P10_SYSCALL_RETURN", ROOT / "kernel/sys/syscall.c", "P10_SYSCALL_RETURN"),
        ("P10_CAP_ENFORCED", ROOT / "kernel/sys/syscall.c", "P10_CAP_ENFORCED"),
        ("P10_RING3_USER_CODE", ROOT / "kernel/arch/x86_64/interrupts.c", "P10_RING3_USER_CODE"),
    ]
    for name, path, token in anchors:
        if name not in marker_by_name:
            continue
        if not path.exists():
            add(f"source_marker_file_missing:{path}")
            source_ok = False
            continue
        txt = path.read_text(encoding="utf-8", errors="replace")
        if token not in txt:
            add(f"source_marker_token_missing:{name}:{path}")
            source_ok = False

    # Ring3 gate report contract fields must remain anchored in gate source.
    # Hardening: validate real REPORT_JSON writer blocks and Python assignments,
    # not raw global token search (comment spoof resistant).
    ring3_gate_script = ROOT / "scripts/ci/gate_ring3_execution_phase10a2.sh"
    required_report_fields = {
        "template_keys": [
            "gate",
            "verdict",
            "violations",
            "violations_count",
            "enforced_ayken_cr3_pcid",
            "observed_ayken_cr3_pcid",
        ],
        "python_assignments": [
            "boot_audit_exit_code",
            "qemu_timeout_seconds",
            "enforced_ayken_cr3_pcid",
            "observed_ayken_cr3_pcid",
        ],
    }
    if not ring3_gate_script.exists():
        add(f"report_schema_contract_file_missing:{ring3_gate_script}")
        report_schema_ok = False
    else:
        gate_text = ring3_gate_script.read_text(encoding="utf-8", errors="replace")

        # 1) REPORT_JSON heredoc templates.
        heredoc_rows = re.findall(
            r'cat\s+>\s*"\$\{REPORT_JSON\}"\s*<<EOF\s*\n(.*?)\nEOF',
            gate_text,
            flags=re.DOTALL,
        )
        if not heredoc_rows:
            add("report_schema_contract_missing_templates")
            report_schema_ok = False
        else:
            for idx, body in enumerate(heredoc_rows):
                # Make unexpanded shell vars JSON-friendly for parsing.
                normalized = re.sub(r"\$\{[^}]+\}", "0", body)
                try:
                    row = json.loads(normalized)
                except Exception:
                    add(f"report_schema_contract_template_invalid_json:index={idx}")
                    report_schema_ok = False
                    continue
                if not isinstance(row, dict):
                    add(f"report_schema_contract_template_invalid_type:index={idx}")
                    report_schema_ok = False
                    continue
                for field in required_report_fields["template_keys"]:
                    if field not in row:
                        add(f"report_schema_contract_missing_template_field:{field}:index={idx}")
                        report_schema_ok = False

        # 2) Final report augmentation assignments must exist in Python block.
        for field in required_report_fields["python_assignments"]:
            if not re.search(
                rf'^\s*row\["{re.escape(field)}"\]\s*=',
                gate_text,
                flags=re.MULTILINE,
            ):
                add(f"report_schema_contract_missing_assignment:{field}")
                report_schema_ok = False

    if isinstance(version_doc, dict):
        cv = version_doc.get("constitution_version")
        if isinstance(cv, str) and isinstance(schema, str) and schema != cv:
            add(f"marker_schema_version_mismatch:marker={schema}:constitution={cv}")
            marker_ok = False
else:
    marker_ok = False
    source_ok = False
    report_schema_ok = False

schema_v1_doc = load_json(SCHEMA_V1, "missing_file", "marker_schema_v1_parse_error")
schema_v1_contract = normalize_schema_contract(schema_v1_doc)
if not isinstance(schema_v1_doc, dict):
    schema_v1_contract_ok = False
elif schema_v1_doc.get("version") != 1:
    add("marker_schema_v1_version_mismatch")
    schema_v1_contract_ok = False
if schema_v1_contract is None:
    add("marker_schema_v1_contract_invalid")
    schema_v1_contract_ok = False

changed = [ln.strip() for ln in CHANGED_TXT.read_text(encoding="utf-8", errors="replace").splitlines() if ln.strip()] if CHANGED_TXT.exists() else []
changed_set = set(changed)

current_version = parse_semver(version_doc.get("constitution_version")) if isinstance(version_doc, dict) else None
if current_version is None:
    add("constitution_version_invalid")
    versioning_ok = False

if MARKER_REL in changed_set and VERSION_REL not in changed_set:
    add("constitution_version_bump_required")
    versioning_ok = False

old_version_doc = git_show_json(BASE_REV, VERSION_REL)
old_marker_doc = git_show_json(BASE_REV, MARKER_REL)
old_schema_v1_doc = git_show_json(BASE_REV, SCHEMA_V1_REL)
old_schema_v2_doc = git_show_json(BASE_REV, SCHEMA_V2_REL)
old_version = parse_semver(old_version_doc.get("constitution_version")) if isinstance(old_version_doc, dict) else None

required_bump = "none"
reasons = []
schema_v1_change_class = "none"
schema_v1_change_reasons = []
schema_v2_introduced = False

if MARKER_REL in changed_set and old_marker_doc is not None and isinstance(marker_doc, dict):
    delta = classify_delta(old_marker_doc, marker_doc)
    if bump_rank(delta) > bump_rank(required_bump):
        required_bump = delta
    if delta != "none":
        reasons.append(f"markers:{delta}")

if SCHEMA_V1_REL in changed_set and old_schema_v1_doc is not None and isinstance(schema_v1_doc, dict):
    schema_v1_change_class, schema_v1_change_reasons = classify_schema_v1_delta(old_schema_v1_doc, schema_v1_doc)
    if schema_v1_change_class == "breaking":
        old_v2_exists = old_schema_v2_doc is not None
        new_v2_exists = (ROOT / SCHEMA_V2_REL).exists()
        v2_introduced = new_v2_exists and not old_v2_exists
        if not v2_introduced:
            add("marker_schema_v1_breaking_requires_v2")
            schema_v1_breaking_guard_ok = False
        else:
            schema_v2_introduced = True

if SCHEMA_V2_REL in changed_set and old_schema_v2_doc is None and (ROOT / SCHEMA_V2_REL).exists():
    schema_v2_introduced = True

actual_bump = semver_bump(old_version, current_version)
if schema_v2_introduced and actual_bump != "major":
    add("constitution_version_major_required:schema_v2_introduced")
    versioning_ok = False
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

sha_val = MARKER_REGISTRY_SHA.read_text(encoding="utf-8", errors="replace").strip() if MARKER_REGISTRY_SHA.exists() else "MISSING"
schema_sha_val = SCHEMA_V1_SHA.read_text(encoding="utf-8", errors="replace").strip() if SCHEMA_V1_SHA.exists() else "MISSING"
meta = {
    "time_utc": NOW,
    "git_sha": GIT_SHA,
    "diff_range": DIFF_RANGE,
    "base_rev": BASE_REV,
    "enforced": "1",
    "scope": "tier-2-phase-scoped",
    "constitution_version": version_doc.get("constitution_version") if isinstance(version_doc, dict) else "UNKNOWN",
    "required_bump": required_bump,
    "actual_bump": actual_bump,
    "marker_registry_sha256": sha_val if sha_val else "MISSING",
    "marker_schema_v1_sha256": schema_sha_val if schema_sha_val else "MISSING",
    "schema_v1_change_class": schema_v1_change_class,
    "schema_v1_change_reasons": ",".join(schema_v1_change_reasons),
    "schema_v2_introduced": int(schema_v2_introduced),
    "changed_files_count": len(changed),
    "violations_count": len(violations),
}
META_TXT.write_text("".join(f"{k}={v}\n" for k, v in meta.items()), encoding="utf-8")

report = {
    "gate": "runtime-marker-contract",
    "tier": "tier-2-phase-scoped",
    "verdict": "PASS" if not violations else "FAIL",
    "violations_count": len(violations),
    "meta": meta,
    "checks": {
        "marker_registry_contract": "PASS" if marker_ok else "FAIL",
        "marker_schema_v1_contract": "PASS" if schema_v1_contract_ok else "FAIL",
        "marker_schema_v1_breaking_guard": "PASS" if schema_v1_breaking_guard_ok else "FAIL",
        "schema_v2_major_bump_guard": "PASS" if (not schema_v2_introduced or actual_bump == "major") else "FAIL",
        "source_marker_anchor": "PASS" if source_ok else "FAIL",
        "ring3_report_schema_contract": "PASS" if report_schema_ok else "FAIL",
        "versioning_policy": "PASS" if versioning_ok else "FAIL",
    },
    "changed_files": changed,
    "violations": violations,
}
REPORT_JSON.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
raise SystemExit(0 if not violations else 2)
PY

VIOLATION_COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
if [[ "${VIOLATION_COUNT}" -gt 0 ]]; then
  echo "runtime-marker-contract: FAIL (${VIOLATION_COUNT} violations)"
  echo "See: ${VIOLATIONS_TXT}"
  exit 2
fi

echo "runtime-marker-contract: PASS"
exit 0
