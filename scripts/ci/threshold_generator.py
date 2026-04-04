#!/usr/bin/env python3
import argparse
import json
from pathlib import Path


METRIC_POLICY = {
    "entry_latency_ticks": {
        "mad_multiplier": 10.0,
        "enforcement": "soft",
        "minimum_sample_count": 5,
    },
    "syscall_gate_return_latency_ticks": {
        "mad_multiplier": 8.0,
        "enforcement": "medium",
        "minimum_sample_count": 5,
    },
    "syscall_latency_ticks_pure": {
        "mad_multiplier": 6.0,
        "enforcement": "hard",
        "minimum_sample_count": 10,
    },
}

VARIANCE_RATIO_GUARD = 0.10


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def build_metric_policy(metric_name: str, summary: dict) -> dict:
    policy = METRIC_POLICY[metric_name]
    sample_count = int(summary.get("sample_count", 0))
    median = float(summary.get("median", 0.0))
    mad = float(summary.get("median_abs_deviation", 0.0))
    p95 = float(summary.get("p95", median))
    variance_ratio = (mad / median) if median > 0 else 0.0
    recommended_threshold = max(p95, median + (policy["mad_multiplier"] * mad))

    status = "ready"
    enforcement = policy["enforcement"]
    notes = []

    if sample_count < policy["minimum_sample_count"]:
        status = "insufficient_samples"
        enforcement = "none"
        notes.append(
            f"sample_count {sample_count} is below minimum {policy['minimum_sample_count']}"
        )

    if variance_ratio > VARIANCE_RATIO_GUARD:
        status = "variance_guard_blocked"
        enforcement = "none"
        notes.append(
            f"variance_ratio {variance_ratio:.6f} exceeds guard {VARIANCE_RATIO_GUARD:.2f}"
        )

    return {
        "metric": metric_name,
        "sample_count": sample_count,
        "median": median,
        "median_abs_deviation": mad,
        "p95": p95,
        "recommended_threshold_ticks": recommended_threshold,
        "variance_ratio": variance_ratio,
        "mad_multiplier": policy["mad_multiplier"],
        "formula": f"max(p95, median + {policy['mad_multiplier']:.0f}*MAD)",
        "minimum_sample_count": policy["minimum_sample_count"],
        "variance_ratio_guard": VARIANCE_RATIO_GUARD,
        "enforcement": enforcement,
        "status": status,
        "notes": notes,
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Generate non-authoritative split-metric threshold policy candidates from "
            "performance-learning summary output."
        )
    )
    parser.add_argument(
        "--learning-summary",
        required=True,
        help="Path to gates/performance-learning/summary.json",
    )
    parser.add_argument(
        "--output",
        help="Optional output path for generated threshold policy JSON",
    )
    args = parser.parse_args()

    summary_path = Path(args.learning_summary)
    payload = load_json(summary_path)

    authority = payload.get("authority")
    source_payload = payload.get("source", {})
    metrics = payload.get("metrics", {})

    recommendations = {}
    for metric_name in METRIC_POLICY:
        summary = metrics.get(metric_name)
        if not isinstance(summary, dict):
            continue
        recommendations[metric_name] = build_metric_policy(metric_name, summary)

    output = {
        "schema_version": 1,
        "source": {
            "learning_summary_path": str(summary_path),
            "authority": authority,
            "eligible_run_count": source_payload.get("eligible_run_count"),
            "git_sha": source_payload.get("git_sha"),
            "git_sha_consistent": source_payload.get("git_sha_consistent"),
            "env_hash": source_payload.get("env_hash"),
            "env_hash_consistent": source_payload.get("env_hash_consistent"),
        },
        "policy_matrix": {
            metric_name: {
                "enforcement_target": policy["enforcement"],
                "mad_multiplier": policy["mad_multiplier"],
                "minimum_sample_count": policy["minimum_sample_count"],
            }
            for metric_name, policy in METRIC_POLICY.items()
        },
        "recommendations": recommendations,
    }

    rendered = json.dumps(output, indent=2, sort_keys=True) + "\n"
    if args.output:
        Path(args.output).write_text(rendered, encoding="utf-8")
    else:
        print(rendered, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
