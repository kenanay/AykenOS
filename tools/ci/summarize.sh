#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  tools/ci/summarize.sh --run-dir evidence/run-<id>
  tools/ci/summarize.sh --run-dir evidence/run-<id> --require-kill-switch-completeness
EOF
}

RUN_DIR=""
REQUIRE_KILL_SWITCH_COMPLETENESS=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --run-dir)
      RUN_DIR="$2"
      shift 2
      ;;
    --require-kill-switch-completeness)
      REQUIRE_KILL_SWITCH_COMPLETENESS=1
      shift 1
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

cmd=(python3 ./tools/ci/summarize_ci_run.py --run-dir "${RUN_DIR}")
if [[ "${REQUIRE_KILL_SWITCH_COMPLETENESS}" == "1" ]]; then
  cmd+=(--require-kill-switch-completeness)
fi
"${cmd[@]}"

echo "summary: ${RUN_DIR}/reports/summary.json"
echo "kill_switch_summary: ${RUN_DIR}/reports/kill_switch_summary.json"
if [[ -s "${RUN_DIR}/reports/kill_switch_summary.txt" ]]; then
  cat "${RUN_DIR}/reports/kill_switch_summary.txt"
fi
