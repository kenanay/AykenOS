#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_TOOLS="${ROOT}/tools/ci"
source "${CI_TOOLS}/lib.sh"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/check_naming_convention.sh --evidence-dir evidence/run-<id>/gates/naming-convention
    [--diff-range <git-range>]
    [--scope-file scripts/ci/naming-convention-scope.regex]
    [--deny-file scripts/ci/naming-convention-deny.regex]
    [--allow-file scripts/ci/naming-convention-legacy-allow.regex]

Exit codes:
  0: pass/skip
  2: naming convention violations detected
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
DIFF_RANGE="${NAMING_DIFF_RANGE:-}"
SCOPE_FILE="${ROOT}/scripts/ci/naming-convention-scope.regex"
DENY_FILE="${ROOT}/scripts/ci/naming-convention-deny.regex"
ALLOW_FILE="${ROOT}/scripts/ci/naming-convention-legacy-allow.regex"

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
    --scope-file)
      SCOPE_FILE="$2"
      shift 2
      ;;
    --deny-file)
      DENY_FILE="$2"
      shift 2
      ;;
    --allow-file)
      ALLOW_FILE="$2"
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
CHANGED_TXT="${EVIDENCE_DIR}/changed-files.txt"
SCOPED_TXT="${EVIDENCE_DIR}/scoped-files.txt"
ALLOWLISTED_TXT="${EVIDENCE_DIR}/allowlisted-files.txt"
HITS_TXT="${EVIDENCE_DIR}/hits.txt"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"
REPORT_JSON="${EVIDENCE_DIR}/report.json"
DIFF_TXT="${EVIDENCE_DIR}/diff.patch"

: > "${CHANGED_TXT}"
: > "${SCOPED_TXT}"
: > "${ALLOWLISTED_TXT}"
: > "${HITS_TXT}"
: > "${VIOLATIONS_TXT}"
: > "${DIFF_TXT}"

resolve_diff_range() {
  if [[ -n "${DIFF_RANGE}" ]]; then
    echo "${DIFF_RANGE}"
    return 0
  fi

  if [[ -n "${GITHUB_BASE_REF:-}" ]]; then
    local base_ref="origin/${GITHUB_BASE_REF}"
    if ! git -C "${ROOT}" rev-parse --verify "${base_ref}" >/dev/null 2>&1; then
      git -C "${ROOT}" fetch --no-tags --depth=200 origin "${GITHUB_BASE_REF}:${base_ref}" >/dev/null 2>&1 || true
      git -C "${ROOT}" fetch --no-tags --depth=200 origin "${GITHUB_BASE_REF}" >/dev/null 2>&1 || true
    fi
    if git -C "${ROOT}" rev-parse --verify "${base_ref}" >/dev/null 2>&1; then
      local merge_base
      merge_base="$(git -C "${ROOT}" merge-base "${base_ref}" HEAD 2>/dev/null || true)"
      if [[ -n "${merge_base}" ]]; then
        echo "${merge_base}...HEAD"
        return 0
      fi
    fi
  fi

  if git -C "${ROOT}" rev-parse --verify origin/main >/dev/null 2>&1; then
    local merge_base
    merge_base="$(git -C "${ROOT}" merge-base origin/main HEAD 2>/dev/null || true)"
    if [[ -n "${merge_base}" ]]; then
      echo "${merge_base}...HEAD"
      return 0
    fi
  fi

  if git -C "${ROOT}" rev-parse --verify HEAD~1 >/dev/null 2>&1; then
    echo "HEAD~1...HEAD"
    return 0
  fi

  echo "HEAD"
  return 0
}

RANGE="$(resolve_diff_range)"

if git -C "${ROOT}" diff --name-only --diff-filter=ACMRDT "${RANGE}" > "${CHANGED_TXT}" 2>/dev/null; then
  git -C "${ROOT}" diff --no-color --unified=0 "${RANGE}" > "${DIFF_TXT}" 2>/dev/null || true
  if [[ ! -s "${CHANGED_TXT}" && -f "${ROOT}/.git/HEAD" ]]; then
    git -C "${ROOT}" show --pretty="" --name-only HEAD > "${CHANGED_TXT}" 2>/dev/null || true
    git -C "${ROOT}" show --pretty="" --no-color --unified=0 HEAD > "${DIFF_TXT}" 2>/dev/null || true
  fi
else
  git -C "${ROOT}" show --pretty="" --name-only HEAD > "${CHANGED_TXT}" 2>/dev/null || true
  git -C "${ROOT}" show --pretty="" --no-color --unified=0 HEAD > "${DIFF_TXT}" 2>/dev/null || true
fi

ROOT_ENV="${ROOT}" \
SCOPE_FILE_ENV="${SCOPE_FILE}" \
DENY_FILE_ENV="${DENY_FILE}" \
ALLOW_FILE_ENV="${ALLOW_FILE}" \
CHANGED_TXT_ENV="${CHANGED_TXT}" \
SCOPED_TXT_ENV="${SCOPED_TXT}" \
ALLOWLISTED_TXT_ENV="${ALLOWLISTED_TXT}" \
HITS_TXT_ENV="${HITS_TXT}" \
VIOLATIONS_TXT_ENV="${VIOLATIONS_TXT}" \
META_TXT_ENV="${META_TXT}" \
REPORT_JSON_ENV="${REPORT_JSON}" \
DIFF_TXT_ENV="${DIFF_TXT}" \
NOW_ENV="$(ci_now_utc)" \
GIT_SHA_ENV="$(git -C "${ROOT}" rev-parse HEAD 2>/dev/null || echo NO_GIT)" \
DIFF_RANGE_ENV="${RANGE}" \
python3 - <<'PY'
import json
import os
import re
from pathlib import Path

ROOT = Path(os.environ["ROOT_ENV"])
SCOPE_FILE = Path(os.environ["SCOPE_FILE_ENV"])
DENY_FILE = Path(os.environ["DENY_FILE_ENV"])
ALLOW_FILE = Path(os.environ["ALLOW_FILE_ENV"])
CHANGED_TXT = Path(os.environ["CHANGED_TXT_ENV"])
SCOPED_TXT = Path(os.environ["SCOPED_TXT_ENV"])
ALLOWLISTED_TXT = Path(os.environ["ALLOWLISTED_TXT_ENV"])
HITS_TXT = Path(os.environ["HITS_TXT_ENV"])
VIOLATIONS_TXT = Path(os.environ["VIOLATIONS_TXT_ENV"])
META_TXT = Path(os.environ["META_TXT_ENV"])
REPORT_JSON = Path(os.environ["REPORT_JSON_ENV"])
DIFF_TXT = Path(os.environ["DIFF_TXT_ENV"])
NOW = os.environ["NOW_ENV"]
GIT_SHA = os.environ["GIT_SHA_ENV"]
DIFF_RANGE = os.environ["DIFF_RANGE_ENV"]


def read_regex_lines(path: Path):
    if not path.exists():
        return None
    rows = []
    for raw in path.read_text(encoding="utf-8", errors="replace").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        rows.append(line)
    return rows


def path_matches(regex_rows, rel_path: str) -> bool:
    return any(re.search(pat, rel_path) for pat in regex_rows)


scope_rows = read_regex_lines(SCOPE_FILE)
deny_rows = read_regex_lines(DENY_FILE)
allow_rows = read_regex_lines(ALLOW_FILE)

violations = []
hits = []
allowlisted = []

changed_files = [
    line.strip()
    for line in CHANGED_TXT.read_text(encoding="utf-8", errors="replace").splitlines()
    if line.strip()
]

if scope_rows is None:
    violations.append(f"missing_file:{SCOPE_FILE}")
    scope_rows = []
if deny_rows is None:
    violations.append(f"missing_file:{DENY_FILE}")
    deny_rows = []
if allow_rows is None:
    violations.append(f"missing_file:{ALLOW_FILE}")
    allow_rows = []

scoped_files = [path for path in changed_files if path_matches(scope_rows, path)]
SCOPED_TXT.write_text(
    "\n".join(scoped_files) + ("\n" if scoped_files else ""),
    encoding="utf-8",
)

diff_lines = DIFF_TXT.read_text(encoding="utf-8", errors="replace").splitlines()
added_lines = {}
current_file = None
current_new_line = None

for raw in diff_lines:
    if raw.startswith("+++ b/"):
        current_file = raw[6:]
        current_new_line = None
        continue
    if raw.startswith("+++ "):
        current_file = None
        current_new_line = None
        continue
    if raw.startswith("@@"):
        match = re.search(r"\+([0-9]+)(?:,([0-9]+))?", raw)
        current_new_line = int(match.group(1)) if match else None
        continue
    if current_file is None or current_file not in scoped_files:
        continue
    if raw.startswith("+") and not raw.startswith("+++"):
        line_no = current_new_line if current_new_line is not None else 0
        content = raw[1:]
        added_lines.setdefault(current_file, []).append((line_no, content))
        if current_new_line is not None:
            current_new_line += 1
        continue
    if raw.startswith(" "):
        if current_new_line is not None:
            current_new_line += 1
        continue

deny_patterns = [re.compile(pat) for pat in deny_rows]

for rel_path in scoped_files:
    if path_matches(allow_rows, rel_path):
        allowlisted.append(rel_path)
        continue

    for line_no, content in added_lines.get(rel_path, []):
        stripped = content.lstrip()
        if re.match(r"^(//|/\*|\*)\s*LEGACY:", stripped):
            continue
        for pat in deny_patterns:
            if pat.search(content):
                snippet = content.strip()
                if len(snippet) > 160:
                    snippet = snippet[:157] + "..."
                hit = f"{rel_path}:{line_no}:pattern={pat.pattern}:line={snippet}"
                hits.append(hit)
                violations.append(f"forbidden_term:{hit}")
                break

ALLOWLISTED_TXT.write_text(
    "\n".join(allowlisted) + ("\n" if allowlisted else ""),
    encoding="utf-8",
)
HITS_TXT.write_text(
    "\n".join(hits) + ("\n" if hits else ""),
    encoding="utf-8",
)

skipped = not scoped_files
verdict = "PASS" if not violations else "FAIL"

meta = {
    "time_utc": NOW,
    "git_sha": GIT_SHA,
    "diff_range": DIFF_RANGE,
    "changed_files_count": len(changed_files),
    "scoped_files_count": len(scoped_files),
    "allowlisted_files_count": len(allowlisted),
    "violations_count": len(violations),
    "skipped": 1 if skipped else 0,
}

META_TXT.write_text(
    "".join(f"{key}={value}\n" for key, value in meta.items()),
    encoding="utf-8",
)
VIOLATIONS_TXT.write_text(
    "\n".join(violations) + ("\n" if violations else ""),
    encoding="utf-8",
)

report = {
    "gate": "naming-convention",
    "verdict": verdict,
    "skipped": skipped,
    "skip_reason": "no_scoped_changes" if skipped else "",
    "meta": meta,
    "changed_files": changed_files,
    "scoped_files": scoped_files,
    "allowlisted_files": allowlisted,
    "hits": hits,
    "violations": violations,
}
REPORT_JSON.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

raise SystemExit(0 if not violations else 2)
PY

VIOLATIONS_COUNT="$(grep -c . "${VIOLATIONS_TXT}" 2>/dev/null || true)"
if [[ "${VIOLATIONS_COUNT}" -gt 0 ]]; then
  echo "naming-convention: FAIL (${VIOLATIONS_COUNT} violations)"
  echo "See: ${VIOLATIONS_TXT}"
  exit 2
fi

if [[ -s "${SCOPED_TXT}" ]]; then
  echo "naming-convention: PASS"
else
  echo "naming-convention: PASS (SKIP no scoped changes)"
fi

exit 0
