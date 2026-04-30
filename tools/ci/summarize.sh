#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  tools/ci/summarize.sh --run-dir evidence/run-<id>
  tools/ci/summarize.sh --run-dir evidence/run-<id> --gate gate-name
  tools/ci/summarize.sh --run-dir evidence/run-<id> --require-kill-switch-completeness
  tools/ci/summarize.sh --run-dir evidence/run-<id> --show-kill-switch-summary

Options:
  --gate <name>                        Evaluate command exit status for only the named gate
                                      while still generating cumulative summary artifacts.
  --require-kill-switch-completeness  Fail if kill-switch gates are not all discovered.
                                      Also enables kill-switch summary output.
  --show-kill-switch-summary          Print kill-switch summary to stdout (without failing
                                      on incomplete coverage). Used by ci-kill-switch-phase13.
EOF
}

RUN_DIR=""
SUMMARY_GATE=""
REQUIRE_KILL_SWITCH_COMPLETENESS=0
SHOW_KILL_SWITCH_SUMMARY=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --run-dir)
      RUN_DIR="$2"
      shift 2
      ;;
    --gate)
      SUMMARY_GATE="$2"
      shift 2
      ;;
    --require-kill-switch-completeness)
      REQUIRE_KILL_SWITCH_COMPLETENESS=1
      SHOW_KILL_SWITCH_SUMMARY=1
      shift 1
      ;;
    --show-kill-switch-summary)
      SHOW_KILL_SWITCH_SUMMARY=1
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
if [[ -n "${SUMMARY_GATE}" ]]; then
  cmd+=(--gate "${SUMMARY_GATE}")
fi
if [[ "${REQUIRE_KILL_SWITCH_COMPLETENESS}" == "1" ]]; then
  cmd+=(--require-kill-switch-completeness)
fi
"${cmd[@]}"

echo "summary: ${RUN_DIR}/reports/summary.json"
echo "kill_switch_summary: ${RUN_DIR}/reports/kill_switch_summary.json"
if [[ "${SHOW_KILL_SWITCH_SUMMARY}" == "1" ]] && [[ -s "${RUN_DIR}/reports/kill_switch_summary.txt" ]]; then
  cat "${RUN_DIR}/reports/kill_switch_summary.txt"
fi
