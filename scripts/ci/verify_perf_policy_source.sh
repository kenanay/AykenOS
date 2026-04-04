#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci/verify_perf_policy_source.sh \
    --policy-json path/to/threshold_policy.json \
    --env-json path/to/env.json \
    --expected-authority github-hosted-ubuntu-24.04-x64 \
    --expected-git-sha <git-sha> \
    --output-json path/to/policy-verification.json
USAGE
}

POLICY_JSON=""
ENV_JSON=""
EXPECTED_AUTHORITY=""
EXPECTED_GIT_SHA=""
OUTPUT_JSON=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --policy-json)
      POLICY_JSON="$2"
      shift 2
      ;;
    --env-json)
      ENV_JSON="$2"
      shift 2
      ;;
    --expected-authority)
      EXPECTED_AUTHORITY="$2"
      shift 2
      ;;
    --expected-git-sha)
      EXPECTED_GIT_SHA="$2"
      shift 2
      ;;
    --output-json)
      OUTPUT_JSON="$2"
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

if [[ -z "${POLICY_JSON}" || -z "${ENV_JSON}" || -z "${EXPECTED_AUTHORITY}" || -z "${EXPECTED_GIT_SHA}" || -z "${OUTPUT_JSON}" ]]; then
  usage
  exit 3
fi

python3 - <<'PY' "${POLICY_JSON}" "${ENV_JSON}" "${EXPECTED_AUTHORITY}" "${EXPECTED_GIT_SHA}" "${OUTPUT_JSON}"
import json
import sys
from pathlib import Path


def load_optional_json(path: Path):
    if not path.exists():
        return None
    return json.loads(path.read_text(encoding="utf-8"))


policy_path = Path(sys.argv[1])
env_path = Path(sys.argv[2])
expected_authority = sys.argv[3]
expected_git_sha = sys.argv[4]
output_path = Path(sys.argv[5])

policy_payload = load_optional_json(policy_path)
env_payload = load_optional_json(env_path)

expected_env_hash = None
if isinstance(env_payload, dict):
    expected_env_hash = env_payload.get("env_hash")

trusted = True
reason = "ok"
actual = {
    "authority": None,
    "git_sha": None,
    "env_hash": None,
    "git_sha_consistent": None,
    "env_hash_consistent": None,
}

if not isinstance(policy_payload, dict):
    trusted = False
    reason = "policy_missing"
else:
    source = policy_payload.get("source", {})
    actual["authority"] = source.get("authority")
    actual["git_sha"] = source.get("git_sha")
    actual["env_hash"] = source.get("env_hash")
    actual["git_sha_consistent"] = source.get("git_sha_consistent")
    actual["env_hash_consistent"] = source.get("env_hash_consistent")

    if actual["authority"] != expected_authority:
        trusted = False
        reason = "policy_authority_mismatch"
    elif actual["git_sha_consistent"] is not True:
        trusted = False
        reason = "policy_git_sha_inconsistent"
    elif actual["env_hash_consistent"] is not True:
        trusted = False
        reason = "policy_env_hash_inconsistent"
    elif actual["git_sha"] != expected_git_sha:
        trusted = False
        reason = "policy_git_sha_mismatch"
    elif not expected_env_hash:
        trusted = False
        reason = "env_hash_missing"
    elif actual["env_hash"] != expected_env_hash:
        trusted = False
        reason = "policy_env_hash_mismatch"

output = {
    "schema_version": 1,
    "trusted": trusted,
    "reason": reason,
    "policy_path": str(policy_path),
    "expected": {
        "authority": expected_authority,
        "git_sha": expected_git_sha,
        "env_hash": expected_env_hash,
    },
    "actual": actual,
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(output, indent=2, sort_keys=True) + "\n", encoding="utf-8")
raise SystemExit(0 if trusted else 1)
PY
