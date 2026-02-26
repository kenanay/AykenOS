#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_TOOLS="${ROOT}/tools/ci"
source "${CI_TOOLS}/lib.sh"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_governance_policy.sh --evidence-dir evidence/run-<id>/gates/governance-policy
    [--kernel-profile validation]
    [--source-deny scripts/ci/constitutional-source-deny.regex]
    [--source-allow scripts/ci/constitutional-source-allow.regex]
    [--ahs-config _ayken/steering/AHS_CONFIG.toml]
    [--waiver-dir docs/waivers]

Exit codes:
  0: pass/skip
  2: governance policy violations detected
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
KERNEL_PROFILE="${KERNEL_PROFILE:-validation}"
SOURCE_DENY="${ROOT}/scripts/ci/constitutional-source-deny.regex"
SOURCE_ALLOW="${ROOT}/scripts/ci/constitutional-source-allow.regex"
AHS_CONFIG="${ROOT}/_ayken/steering/AHS_CONFIG.toml"
WAIVER_DIR="${ROOT}/docs/waivers"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --kernel-profile)
      KERNEL_PROFILE="$2"
      shift 2
      ;;
    --source-deny)
      SOURCE_DENY="$2"
      shift 2
      ;;
    --source-allow)
      SOURCE_ALLOW="$2"
      shift 2
      ;;
    --ahs-config)
      AHS_CONFIG="$2"
      shift 2
      ;;
    --waiver-dir)
      WAIVER_DIR="$2"
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

mkdir -p "${EVIDENCE_DIR}"
TRACKED_KERNEL="${EVIDENCE_DIR}/tracked-kernel-files.txt"
SOURCE_HITS="${EVIDENCE_DIR}/source-deny-hits.txt"
WAIVER_AUDIT="${EVIDENCE_DIR}/waiver-audit.txt"
AHS_CHECK_TXT="${EVIDENCE_DIR}/ahs-check.txt"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"
REPORT_JSON="${EVIDENCE_DIR}/report.json"

NOW="$(ci_now_utc)"
GIT_SHA="$(git -C "${ROOT}" rev-parse HEAD 2>/dev/null || echo NO_GIT)"

if [[ "${KERNEL_PROFILE}" != "validation" ]]; then
  cat > "${META_TXT}" <<META
kernel_profile=${KERNEL_PROFILE}
time_utc=${NOW}
git_sha=${GIT_SHA}
reason=requires_validation_profile
violations_count=0
META
  : > "${VIOLATIONS_TXT}"
  : > "${SOURCE_HITS}"
  : > "${AHS_CHECK_TXT}"
  : > "${WAIVER_AUDIT}"
  python3 - <<'PY' "${REPORT_JSON}" "${KERNEL_PROFILE}" "${NOW}"
import json
import sys

path, profile, now = sys.argv[1:4]
report = {
    "gate": "governance-policy",
    "kernel_profile": profile,
    "time_utc": now,
    "verdict": "SKIP",
    "reason": "requires_validation_profile",
    "violations_count": 0,
    "checks": {
        "source_deny_scan": "SKIP",
        "ahs_thresholds": "SKIP",
        "waiver_policy": "SKIP",
    },
    "violations": [],
}
with open(path, "w", encoding="utf-8") as fh:
    json.dump(report, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY
  echo "governance-policy: SKIP (kernel_profile=${KERNEL_PROFILE})"
  exit 0
fi

ROOT_ENV="${ROOT}" \
SOURCE_DENY_ENV="${SOURCE_DENY}" \
SOURCE_ALLOW_ENV="${SOURCE_ALLOW}" \
AHS_CONFIG_ENV="${AHS_CONFIG}" \
WAIVER_DIR_ENV="${WAIVER_DIR}" \
TRACKED_KERNEL_ENV="${TRACKED_KERNEL}" \
SOURCE_HITS_ENV="${SOURCE_HITS}" \
WAIVER_AUDIT_ENV="${WAIVER_AUDIT}" \
AHS_CHECK_TXT_ENV="${AHS_CHECK_TXT}" \
VIOLATIONS_TXT_ENV="${VIOLATIONS_TXT}" \
META_TXT_ENV="${META_TXT}" \
REPORT_JSON_ENV="${REPORT_JSON}" \
NOW_ENV="${NOW}" \
GIT_SHA_ENV="${GIT_SHA}" \
KERNEL_PROFILE_ENV="${KERNEL_PROFILE}" \
python3 - <<'PY'
import json
import os
import re
import subprocess
from datetime import date
from pathlib import Path

ROOT = Path(os.environ["ROOT_ENV"])
SOURCE_DENY = Path(os.environ["SOURCE_DENY_ENV"])
SOURCE_ALLOW = Path(os.environ["SOURCE_ALLOW_ENV"])
AHS_CONFIG = Path(os.environ["AHS_CONFIG_ENV"])
WAIVER_DIR = Path(os.environ["WAIVER_DIR_ENV"])

TRACKED_KERNEL = Path(os.environ["TRACKED_KERNEL_ENV"])
SOURCE_HITS = Path(os.environ["SOURCE_HITS_ENV"])
WAIVER_AUDIT = Path(os.environ["WAIVER_AUDIT_ENV"])
AHS_CHECK_TXT = Path(os.environ["AHS_CHECK_TXT_ENV"])
VIOLATIONS_TXT = Path(os.environ["VIOLATIONS_TXT_ENV"])
META_TXT = Path(os.environ["META_TXT_ENV"])
REPORT_JSON = Path(os.environ["REPORT_JSON_ENV"])

NOW = os.environ["NOW_ENV"]
GIT_SHA = os.environ["GIT_SHA_ENV"]
KERNEL_PROFILE = os.environ["KERNEL_PROFILE_ENV"]

for path in (TRACKED_KERNEL, SOURCE_HITS, WAIVER_AUDIT, AHS_CHECK_TXT, VIOLATIONS_TXT):
    path.write_text("", encoding="utf-8")

violations = []
source_hits = []
waiver_audit_rows = []
waiver_violations_count = 0
ahs_check_lines = []


def add_violation(kind: str, detail: str) -> None:
    violations.append(f"{kind}:{detail}")


def load_regex_lines(path: Path):
    if not path.exists():
        return None
    rows = []
    for raw in path.read_text(encoding="utf-8", errors="replace").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        rows.append(line)
    return rows


def parse_allow_rules(path: Path):
    rules = []
    rows = load_regex_lines(path)
    if rows is None:
        return None
    for line in rows:
        if ":" in line:
            file_rgx, line_rgx = line.split(":", 1)
            rules.append(("scoped", file_rgx.strip(), line_rgx.strip()))
        else:
            rules.append(("global", line))
    return rules


def allow_matches(allow_rules, rel_path: str, line: str):
    if not allow_rules:
        return False
    for rule in allow_rules:
        if rule[0] == "global":
            if re.search(rule[1], line):
                return True
        else:
            _, file_rgx, line_rgx = rule
            if re.search(file_rgx, rel_path) and re.search(line_rgx, line):
                return True
    return False


try:
    tracked_kernel = subprocess.check_output(
        ["git", "-C", str(ROOT), "ls-files", "kernel"], text=True
    ).splitlines()
except subprocess.CalledProcessError:
    add_violation("git_ls_files_failed", "kernel")
    tracked_kernel = []

TRACKED_KERNEL.write_text(
    "\n".join(tracked_kernel) + ("\n" if tracked_kernel else ""),
    encoding="utf-8",
)

# 1) Kernel source deny/allow scan.
deny_rows = load_regex_lines(SOURCE_DENY)
allow_rules = parse_allow_rules(SOURCE_ALLOW)
if deny_rows is None:
    add_violation("missing_file", str(SOURCE_DENY))
else:
    deny_patterns = [re.compile(pat) for pat in deny_rows]
    src_ext = {".c", ".h", ".S", ".s", ".asm", ".inc", ".ld"}
    for rel in tracked_kernel:
        if re.search(r"(?:^|/).*(?:_test|validation_test)\.c$", rel):
            continue
        p = ROOT / rel
        if p.suffix not in src_ext or not p.is_file():
            continue
        try:
            lines = p.read_text(encoding="utf-8", errors="replace").splitlines()
        except Exception as exc:
            add_violation("source_read_error", f"{rel}:{type(exc).__name__}")
            continue
        for idx, line in enumerate(lines, 1):
            stripped = line.strip()
            if (
                stripped.startswith("//")
                or stripped.startswith("/*")
                or stripped.startswith("*")
                or stripped.startswith(";")
            ):
                continue
            scan_line = re.sub(r"//.*$", "", line)
            scan_line = re.sub(r'"(?:\\.|[^"\\])*"', '""', scan_line)
            scan_line = re.sub(r"'(?:\\.|[^'\\])*'", "''", scan_line)
            for pat in deny_patterns:
                if pat.search(scan_line):
                    if allow_matches(allow_rules, rel, scan_line):
                        continue
                    snippet = line.strip()
                    if len(snippet) > 160:
                        snippet = snippet[:157] + "..."
                    hit = f"{rel}:{idx}:pattern={pat.pattern}:line={snippet}"
                    source_hits.append(hit)
                    add_violation("source_deny", hit)
                    break

if source_hits:
    SOURCE_HITS.write_text("\n".join(source_hits) + "\n", encoding="utf-8")

# 2) AHS threshold guard.
if not AHS_CONFIG.exists():
    add_violation("missing_file", str(AHS_CONFIG))
else:
    ahs_text = AHS_CONFIG.read_text(encoding="utf-8", errors="replace")
    checks = {
        "P5_minimum": 95,
        "P4_5_minimum": 90,
        "clean_threshold": 90,
    }
    for key, min_val in checks.items():
        m = re.search(rf"^\s*{re.escape(key)}\s*=\s*([0-9]+)\b", ahs_text, flags=re.MULTILINE)
        if not m:
            ahs_check_lines.append(f"{key}=MISSING")
            add_violation("ahs_config_missing", key)
            continue
        val = int(m.group(1))
        ahs_check_lines.append(f"{key}={val}")
        if val < min_val:
            add_violation("ahs_threshold", f"{key}:required>={min_val}:actual={val}")

AHS_CHECK_TXT.write_text(
    "\n".join(ahs_check_lines) + ("\n" if ahs_check_lines else ""),
    encoding="utf-8",
)

# 3) Waiver policy checks.
waiver_files = []
if not WAIVER_DIR.exists():
    add_violation("missing_dir", str(WAIVER_DIR))
else:
    for p in sorted(WAIVER_DIR.glob("*.md")):
        if p.name in {"README.md", "WAIVER_TEMPLATE.md"}:
            continue
        waiver_files.append(p)

today = date.today()
critical_tokens = [
    "KERNEL.RING0.POLICY",
    "KERNEL.CAPABILITY.BYPASS",
    "SECURITY.BOUNDARY.VIOLATION",
    "CONSTITUTIONAL.ENFORCEMENT.BYPASS",
    "NON_OVERRIDABLE",
]

for wf in waiver_files:
    text = wf.read_text(encoding="utf-8", errors="replace")
    rel = str(wf.relative_to(ROOT))

    exp_match = re.search(r"^\s*-\s*Expiry Date:\s*([0-9]{4}-[0-9]{2}-[0-9]{2})\s*$", text, flags=re.MULTILINE)
    date_match = re.search(r"^\s*-\s*Date:\s*([0-9]{4}-[0-9]{2}-[0-9]{2})\s*$", text, flags=re.MULTILINE)
    issue_match = re.search(r"^\s*-\s*Related Issue:\s*(.+)$", text, flags=re.MULTILINE)

    row = [rel]
    if not exp_match:
        waiver_violations_count += 1
        add_violation("waiver_missing_expiry", rel)
        row.append("expiry=MISSING")
        expiry_date = None
    else:
        row.append(f"expiry={exp_match.group(1)}")
        try:
            y, m, d = map(int, exp_match.group(1).split("-"))
            expiry_date = date(y, m, d)
            if expiry_date < today:
                waiver_violations_count += 1
                add_violation("waiver_expired", f"{rel}:{expiry_date.isoformat()}")
        except Exception:
            waiver_violations_count += 1
            add_violation("waiver_invalid_expiry", rel)
            expiry_date = None

    if not issue_match or not issue_match.group(1).strip():
        waiver_violations_count += 1
        add_violation("waiver_missing_issue", rel)
        row.append("issue=MISSING")
    else:
        row.append("issue=OK")

    if date_match and expiry_date:
        try:
            y, m, d = map(int, date_match.group(1).split("-"))
            start_date = date(y, m, d)
            duration = (expiry_date - start_date).days
            row.append(f"duration_days={duration}")
            if duration > 90:
                waiver_violations_count += 1
                add_violation("waiver_duration_exceeds_90", f"{rel}:{duration}")
        except Exception:
            waiver_violations_count += 1
            add_violation("waiver_invalid_date", rel)
            row.append("duration_days=INVALID")
    else:
        row.append("duration_days=UNKNOWN")

    for token in critical_tokens:
        if token.lower() in text.lower():
            waiver_violations_count += 1
            add_violation("waiver_non_overridable", f"{rel}:{token}")
            row.append("non_overridable_ref=YES")
            break

    waiver_audit_rows.append(" ".join(row))

if waiver_audit_rows:
    WAIVER_AUDIT.write_text("\n".join(waiver_audit_rows) + "\n", encoding="utf-8")

violations = sorted(set(v for v in violations if v))
VIOLATIONS_TXT.write_text(
    "\n".join(violations) + ("\n" if violations else ""),
    encoding="utf-8",
)

meta = {
    "time_utc": NOW,
    "git_sha": GIT_SHA,
    "kernel_profile": KERNEL_PROFILE,
    "source_deny_file": str(SOURCE_DENY),
    "source_allow_file": str(SOURCE_ALLOW),
    "ahs_config_file": str(AHS_CONFIG),
    "waiver_dir": str(WAIVER_DIR),
    "tracked_kernel_file_count": len(tracked_kernel),
    "source_deny_hits_count": len(source_hits),
    "waiver_file_count": len(waiver_files),
    "waiver_violations_count": waiver_violations_count,
    "violations_count": len(violations),
}

META_TXT.write_text(
    "".join(f"{k}={v}\n" for k, v in meta.items()),
    encoding="utf-8",
)

report = {
    "gate": "governance-policy",
    "kernel_profile": KERNEL_PROFILE,
    "time_utc": NOW,
    "verdict": "PASS" if not violations else "FAIL",
    "violations_count": len(violations),
    "meta": meta,
    "checks": {
        "source_deny_scan": "PASS" if not source_hits else "FAIL",
        "ahs_thresholds": "PASS" if not any(v.startswith("ahs_") for v in violations) else "FAIL",
        "waiver_policy": "PASS" if waiver_violations_count == 0 else "FAIL",
    },
    "source_deny_hits": source_hits,
    "waiver_audit": waiver_audit_rows,
    "violations": violations,
}

REPORT_JSON.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

raise SystemExit(0 if not violations else 2)
PY

VIOLATIONS_COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
if [[ "${VIOLATIONS_COUNT}" -gt 0 ]]; then
  echo "governance-policy: FAIL (${VIOLATIONS_COUNT} violations)"
  echo "See: ${VIOLATIONS_TXT}"
  exit 2
fi

echo "governance-policy: PASS"
exit 0
