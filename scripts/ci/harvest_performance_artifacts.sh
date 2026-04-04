#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT}"

if [[ "$#" -lt 2 ]]; then
  echo "usage: $0 <output-root> <run-id-1> [run-id-2 ...]" >&2
  exit 1
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "ERROR: required tool missing (gh)" >&2
  exit 1
fi

OUT_ROOT="$1"
shift
RUN_IDS=("$@")

BATCH_DIR="${OUT_ROOT}/learning-batch"
EVIDENCE_DIR="${OUT_ROOT}/evidence"
LEARNING_RUN_DIR="${EVIDENCE_DIR}/run-learning-batch/gates/performance-learning"

rm -rf "${BATCH_DIR}" "${LEARNING_RUN_DIR}"
mkdir -p "${BATCH_DIR}" "${LEARNING_RUN_DIR}"

for id in "${RUN_IDS[@]}"; do
  id="${id//[[:space:]]/}"
  if [[ -z "${id}" ]]; then
    continue
  fi
  artifact_name="$(gh api "repos/:owner/:repo/actions/runs/${id}/artifacts" --jq \
    ".artifacts[] | select(.name | startswith(\"performance-evidence-${id}-\")) | .name" | head -n1)"
  if [[ -z "${artifact_name}" ]]; then
    echo "ERROR: missing performance artifact for run ${id}" >&2
    exit 1
  fi
  mkdir -p "${BATCH_DIR}/${id}"
  gh run download "${id}" --name "${artifact_name}" --dir "${BATCH_DIR}/${id}"
done

scripts/ci/gate_performance_learning_review.sh \
  --evidence-dir "${LEARNING_RUN_DIR}" \
  --source-glob "${BATCH_DIR}/*/run-gh-*/gates/performance/report.json"
