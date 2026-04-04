#!/usr/bin/env python3
import argparse
import json
from pathlib import Path


ORDERED_METRICS = [
    "entry_latency_ticks",
    "syscall_gate_return_latency_ticks",
    "syscall_latency_ticks_pure",
]


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def fmt_num(value) -> str:
    if isinstance(value, int):
        return f"{value:,}"
    if isinstance(value, float):
        if value >= 1000:
            return f"{value:,.1f}"
        return f"{value:.4f}"
    return str(value)


def build_row(metric: str, payload: dict) -> str:
    return (
        f"| `{metric}`"
        f" | {payload.get('sample_count', '-')}"
        f" | {fmt_num(payload.get('median', '-'))}"
        f" | {fmt_num(payload.get('median_abs_deviation', '-'))}"
        f" | {fmt_num(payload.get('p95', '-'))}"
        f" | {fmt_num(payload.get('recommended_threshold_ticks', '-'))}"
        f" | {fmt_num(payload.get('variance_ratio', '-'))}"
        f" | `{payload.get('enforcement', '-')}`"
        f" | `{payload.get('status', '-')}` |"
    )


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Render markdown PR comment from threshold policy JSON."
    )
    parser.add_argument("--threshold-policy", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    policy = load_json(Path(args.threshold_policy))
    recommendations = policy.get("recommendations", {})
    authority = policy.get("source", {}).get("authority", "unknown")
    summary_path = policy.get("source", {}).get("learning_summary_path", "unknown")

    lines = [
        "## Performance Threshold Recommendation",
        "",
        f"- authority: `{authority}`",
        f"- source: `{summary_path}`",
        "- note: non-authoritative recommendation only; no baseline or threshold mutation",
        "",
        "| Metric | n | median | MAD | p95 | threshold | variance | enforcement | status |",
        "|---|---:|---:|---:|---:|---:|---:|---|---|",
    ]

    for metric in ORDERED_METRICS:
        payload = recommendations.get(metric)
        if isinstance(payload, dict):
            lines.append(build_row(metric, payload))

    note_lines = []
    for metric in ORDERED_METRICS:
        payload = recommendations.get(metric)
        if not isinstance(payload, dict):
            continue
        for note in payload.get("notes", []):
            note_lines.append(f"- `{metric}`: {note}")

    if note_lines:
        lines.extend(["", "### Notes"])
        lines.extend(note_lines)

    Path(args.output).write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
