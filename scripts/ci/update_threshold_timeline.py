#!/usr/bin/env python3
import argparse
import json
from pathlib import Path


METRICS = [
    "entry_latency_ticks",
    "syscall_gate_return_latency_ticks",
    "syscall_latency_ticks_pure",
]


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def load_timeline(path: Path) -> list:
    if not path.exists():
        return []
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Append threshold recommendation snapshot to timeline JSON."
    )
    parser.add_argument("--threshold-policy", required=True)
    parser.add_argument("--timeline", required=True)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--git-sha", required=True)
    parser.add_argument("--created-at-utc", required=True)
    args = parser.parse_args()

    threshold_policy = load_json(Path(args.threshold_policy))
    timeline_path = Path(args.timeline)
    timeline = load_timeline(timeline_path)

    entry = {
        "run_id": args.run_id,
        "git_sha": args.git_sha,
        "created_at_utc": args.created_at_utc,
        "authority": threshold_policy.get("source", {}).get("authority"),
        "metrics": {},
    }

    recommendations = threshold_policy.get("recommendations", {})
    for metric in METRICS:
        payload = recommendations.get(metric)
        if not isinstance(payload, dict):
            continue
        entry["metrics"][metric] = {
            "sample_count": payload.get("sample_count"),
            "median": payload.get("median"),
            "median_abs_deviation": payload.get("median_abs_deviation"),
            "p95": payload.get("p95"),
            "recommended_threshold_ticks": payload.get("recommended_threshold_ticks"),
            "variance_ratio": payload.get("variance_ratio"),
            "enforcement": payload.get("enforcement"),
            "status": payload.get("status"),
        }

    timeline.append(entry)
    timeline_path.parent.mkdir(parents=True, exist_ok=True)
    timeline_path.write_text(json.dumps(timeline, indent=2) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
