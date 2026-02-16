#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  tools/ci/summarize.sh --run-dir evidence/run-<id>
EOF
}

RUN_DIR=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --run-dir)
      RUN_DIR="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown arg: $1" >&2
      usage
      exit 2
      ;;
  esac
done

if [[ -z "${RUN_DIR}" ]]; then
  usage
  exit 2
fi

mkdir -p "${RUN_DIR}/reports"

RUN_DIR_ENV="${RUN_DIR}" python3 - <<'PY'
import json
import os
from pathlib import Path

run_dir = Path(os.environ["RUN_DIR_ENV"])
reports_dir = run_dir / "reports"
reports_dir.mkdir(parents=True, exist_ok=True)

def load_json(path: Path, default):
    if not path.exists():
        return default, None
    try:
        with path.open("r", encoding="utf-8", errors="replace") as fh:
            return json.load(fh), None
    except Exception as exc:
        return default, f"{type(exc).__name__}: {exc}"

def load_text(path: Path, default=""):
    if not path.exists():
        return default
    return path.read_text(encoding="utf-8", errors="replace").strip()

run_meta, run_meta_err = load_json(run_dir / "meta" / "run.json", {})
git_sha = load_text(run_dir / "meta" / "git.txt", "UNKNOWN")
gates = {}
gates_dir = run_dir / "gates"
parse_errors = []

if run_meta_err:
    parse_errors.append({"path": str(run_dir / "meta" / "run.json"), "error": run_meta_err})

for report_path in sorted(gates_dir.glob("*/report.json")):
    report, report_err = load_json(report_path, {})
    gate_name = str((report or {}).get("gate") or report_path.parent.name)
    verdict = str((report or {}).get("verdict", "UNKNOWN"))

    if report_err:
        gates[gate_name] = {
            "verdict": "FAIL",
            "report_path": str(report_path),
            "parse_error": report_err,
        }
        parse_errors.append({"path": str(report_path), "error": report_err})
        continue

    gate_entry = {"verdict": verdict}
    if "violations_count" in report:
        try:
            gate_entry["violations_count"] = int(report.get("violations_count", 0))
        except (TypeError, ValueError):
            gate_entry["violations_count"] = 0

    gates[gate_name] = gate_entry

overall_verdict = "PASS" if gates else "FAIL"
for gate in gates.values():
    verdict = gate.get("verdict")
    # SKIP and WARN are acceptable in provisional mode
    if verdict not in ("PASS", "SKIP", "WARN"):
        overall_verdict = "FAIL"
        break
if parse_errors:
    overall_verdict = "FAIL"

summary = {
    "run_id": run_meta.get("run_id", run_dir.name),
    "time_utc": run_meta.get("time_utc", ""),
    "git_sha": git_sha,
    "verdict": overall_verdict,
    "gates_discovered": len(gates),
    "parse_errors_count": len(parse_errors),
    "parse_errors": parse_errors,
    "gates": gates,
}

with (reports_dir / "summary.json").open("w", encoding="utf-8") as fh:
    json.dump(summary, fh, indent=2, sort_keys=True)
    fh.write("\n")

if overall_verdict != "PASS":
    raise SystemExit(2)
PY

echo "summary: ${RUN_DIR}/reports/summary.json"
