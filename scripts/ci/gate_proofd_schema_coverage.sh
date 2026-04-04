#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/gate_proofd_schema_coverage.sh \
    --evidence-dir evidence/run-<id>/gates/proofd-schema-coverage \
    [--source-gate-dir evidence/run-<id>/gates/proofd-service]

Exit codes:
  0: pass
  2: proofd schema coverage failure
  3: usage/tooling error
USAGE
}

EVIDENCE_DIR=""
SOURCE_GATE_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --evidence-dir)
      EVIDENCE_DIR="$2"
      shift 2
      ;;
    --source-gate-dir)
      SOURCE_GATE_DIR="$2"
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
if ! command -v python3 >/dev/null 2>&1; then
  echo "ERROR: missing required tool: python3" >&2
  exit 3
fi

mkdir -p "${EVIDENCE_DIR}"

if [[ -z "${SOURCE_GATE_DIR}" ]]; then
  SOURCE_GATE_DIR="$(cd "$(dirname "${EVIDENCE_DIR}")" && pwd)/proofd-service"
fi

REPORT_JSON="${EVIDENCE_DIR}/report.json"
DETAIL_REPORT_JSON="${EVIDENCE_DIR}/proofd_schema_coverage_report.json"
VIOLATIONS_TXT="${EVIDENCE_DIR}/violations.txt"
META_TXT="${EVIDENCE_DIR}/meta.txt"
SOURCE_CONTRACT_JSON="${SOURCE_GATE_DIR}/proofd_endpoint_contract.json"

if [[ ! -f "${SOURCE_CONTRACT_JSON}" ]]; then
  echo "ERROR: missing source contract artifact: ${SOURCE_CONTRACT_JSON}" >&2
  exit 3
fi

set +e
python3 - "${SOURCE_CONTRACT_JSON}" "${REPORT_JSON}" "${DETAIL_REPORT_JSON}" "${VIOLATIONS_TXT}" <<'PY'
import json
import sys
from pathlib import Path

source_path = Path(sys.argv[1])
report_path = Path(sys.argv[2])
detail_path = Path(sys.argv[3])
violations_path = Path(sys.argv[4])

payload = json.loads(source_path.read_text(encoding="utf-8"))
schema_contracts = payload.get("schema_contracts")
violations = []
allowed_coverage = {"none", "root_only", "full"}
allowed_root_kinds = {"object", "array", "string", "number", "boolean"}
allowed_response_modes = {
    "computed",
    "artifact_filtered",
    "artifact_json_passthrough",
    "artifact_file_passthrough",
}
coverage_counts = {"none": 0, "root_only": 0, "full": 0}
entries = []
seen_paths = set()

if not isinstance(schema_contracts, list):
    violations.append("schema_contracts_missing")
    schema_contracts = []
elif len(schema_contracts) == 0:
    violations.append("schema_contracts_empty")

for entry in schema_contracts:
    if not isinstance(entry, dict):
        violations.append("schema_contract_entry_not_object")
        continue

    path = entry.get("path_template")
    coverage = entry.get("coverage")
    scope = entry.get("scope")
    artifact_backed = entry.get("artifact_backed")
    response_mode = entry.get("response_mode")
    schema_present = entry.get("schema_present")
    enforcement_active = entry.get("schema_enforcement_active")
    root_kind = entry.get("root_kind")
    required_fields = entry.get("required_fields")
    optional_fields = entry.get("optional_fields")

    if not isinstance(path, str) or not path:
        violations.append("schema_contract_path_missing")
        continue
    if path in seen_paths:
        violations.append(f"schema_contract_duplicate_path:{path}")
        continue
    seen_paths.add(path)

    if coverage not in allowed_coverage:
        violations.append(f"schema_contract_invalid_coverage:{path}:{coverage}")
        continue
    coverage_counts[coverage] += 1

    if scope not in {"root", "run"}:
        violations.append(f"schema_contract_invalid_scope:{path}:{scope}")
    if response_mode not in allowed_response_modes:
        violations.append(f"schema_contract_invalid_response_mode:{path}:{response_mode}")

    if not isinstance(artifact_backed, bool):
        violations.append(f"schema_contract_artifact_backed_missing:{path}")
        artifact_backed = False
    if not isinstance(schema_present, bool):
        violations.append(f"schema_contract_schema_present_missing:{path}")
        schema_present = False
    if not isinstance(enforcement_active, bool):
        violations.append(f"schema_contract_enforcement_active_missing:{path}")
        enforcement_active = False

    if coverage == "none":
        if schema_present:
            violations.append(f"schema_contract_none_with_schema:{path}")
        if enforcement_active:
            violations.append(f"schema_contract_none_with_enforcement:{path}")
        if root_kind is not None:
            violations.append(f"schema_contract_none_with_root_kind:{path}")
        if response_mode in {"computed", "artifact_filtered"}:
            violations.append(f"schema_contract_service_owned_missing_coverage:{path}")
    else:
        if not schema_present:
            violations.append(f"schema_contract_missing_schema:{path}")
        if not enforcement_active:
            violations.append(f"schema_contract_enforcement_inactive:{path}")
        if root_kind not in allowed_root_kinds:
            violations.append(f"schema_contract_invalid_root_kind:{path}:{root_kind}")
        if response_mode in {"artifact_json_passthrough", "artifact_file_passthrough"}:
            violations.append(f"schema_contract_passthrough_not_none:{path}")
        if not isinstance(required_fields, list):
            violations.append(f"schema_contract_required_fields_missing:{path}")
            required_fields = []
        if not isinstance(optional_fields, list):
            violations.append(f"schema_contract_optional_fields_missing:{path}")
            optional_fields = []
        if coverage == "full" and len(required_fields) == 0:
            violations.append(f"schema_contract_full_without_required_fields:{path}")

    entries.append(
        {
            "path_template": path,
            "scope": scope,
            "artifact_backed": artifact_backed,
            "response_mode": response_mode,
            "coverage": coverage,
            "schema_present": schema_present,
            "schema_enforcement_active": enforcement_active,
            "required_field_count": len(required_fields)
            if isinstance(required_fields, list)
            else 0,
            "optional_field_count": len(optional_fields)
            if isinstance(optional_fields, list)
            else 0,
        }
    )

status = "PASS" if not violations else "FAIL"
detail = {
    "status": status,
    "gate": "proofd-schema-coverage",
    "schema_contract_count": len(entries),
    "coverage_counts": coverage_counts,
    "entries": entries,
    "violations": violations,
    "violations_count": len(violations),
    "source_contract_path": str(source_path),
}
report = {
    "gate": "proofd-schema-coverage",
    "mode": "phase14_proofd_schema_coverage",
    "verdict": status,
    "violations": violations,
    "violations_count": len(violations),
}

detail_path.write_text(json.dumps(detail, indent=2) + "\n", encoding="utf-8")
report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
violations_path.write_text(
    ("\n".join(violations) + "\n") if violations else "",
    encoding="utf-8",
)
sys.exit(0 if not violations else 2)
PY
VALIDATOR_RC=$?
set -e

{
  echo "time_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "validator_rc=${VALIDATOR_RC}"
  echo "source_gate_dir=${SOURCE_GATE_DIR}"
  echo "source_contract_json=${SOURCE_CONTRACT_JSON}"
  echo "evidence_dir=${EVIDENCE_DIR}"
} > "${META_TXT}"

if [[ "${VALIDATOR_RC}" -ne 0 ]]; then
  echo "proofd-schema-coverage: FAIL"
  exit 2
fi

echo "proofd-schema-coverage: PASS"
exit 0
