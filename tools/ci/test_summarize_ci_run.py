#!/usr/bin/env python3
"""Black-box tests for tools/ci/summarize.sh kill-switch reduction."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class SummarizeCiRunTest(unittest.TestCase):
    ALL_EXPECTED_GATES = (
        "observability-routing-separation",
        "proofd-observability-boundary",
        "diagnostics-consumer-non-authoritative-contract",
        "diagnostics-callsite-correlation",
        "convergence-non-election-boundary",
        "graph-non-authoritative-contract",
        "cross-node-parity",
        "proof-verdict-binding",
        "proof-bundle",
        "proof-receipt",
        "proofd-service",
        "verifier-authority-resolution",
        "verifier-reputation-prohibition",
    )

    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.script = self.repo_root / "tools" / "ci" / "summarize.sh"
        self.run_dir = self.root / "evidence" / "run-test"
        (self.run_dir / "meta").mkdir(parents=True)
        (self.run_dir / "gates").mkdir(parents=True)
        (self.run_dir / "meta" / "git.txt").write_text(
            "0123456789abcdef0123456789abcdef01234567\n",
            encoding="utf-8",
        )
        (self.run_dir / "meta" / "run.json").write_text(
            json.dumps(
                {
                    "run_id": "run-test",
                    "time_utc": "2026-03-13T12:00:00Z",
                },
                indent=2,
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_reports_passing_kill_switch_with_primary_and_supporting_gates(self) -> None:
        self._write_gate("observability-routing-separation", "PASS")
        self._write_gate("proofd-observability-boundary", "PASS")

        proc = self._run()
        self.assertEqual(proc.returncode, 0, proc.stderr)

        summary = self._load_report("summary.json")
        kill_switch_summary = self._load_report("kill_switch_summary.json")
        summary_text = (self.run_dir / "reports" / "kill_switch_summary.txt").read_text(
            encoding="utf-8"
        )

        self.assertEqual(summary.get("verdict"), "PASS")
        self.assertEqual(kill_switch_summary.get("overall_status"), "PARTIAL")

        item = self._find_kill_switch(kill_switch_summary, "observability-control-plane")
        self.assertEqual(item.get("status"), "PASS")
        self.assertEqual(item.get("failure_trigger"), "PRIMARY_GATE")
        self.assertEqual(item.get("primary_gate", {}).get("status"), "PASS")
        support_names = {
            gate.get("gate"): gate.get("status") for gate in item.get("supporting_gates", [])
        }
        self.assertEqual(support_names["proofd-observability-boundary"], "PASS")
        self.assertIn("PASS: observability -> control plane", summary_text)
        self.assertIn("trigger: PRIMARY_GATE", summary_text)
        self.assertIn(
            "primary: ci-gate-observability-routing-separation (PASS)",
            summary_text,
        )

    def test_reports_support_only_when_only_supporting_gate_is_present(self) -> None:
        self._write_gate("proofd-observability-boundary", "PASS")

        proc = self._run()
        self.assertEqual(proc.returncode, 0, proc.stderr)

        kill_switch_summary = self._load_report("kill_switch_summary.json")
        item = self._find_kill_switch(kill_switch_summary, "observability-control-plane")
        self.assertEqual(kill_switch_summary.get("overall_status"), "PARTIAL")
        self.assertEqual(item.get("status"), "SUPPORT_ONLY")
        self.assertEqual(item.get("failure_trigger"), "SUPPORTING_EVIDENCE_ONLY")
        self.assertEqual(item.get("primary_gate", {}).get("status"), "NOT_EXECUTED")
        self.assertEqual(
            item.get("primary_gate", {}).get("discovery_state"),
            "NOT_DISCOVERED",
        )

    def test_failing_primary_gate_fails_run_and_category(self) -> None:
        self._write_gate("convergence-non-election-boundary", "FAIL")
        self._write_gate("cross-node-parity", "PASS")

        proc = self._run()
        self.assertEqual(proc.returncode, 2)

        summary = self._load_report("summary.json")
        kill_switch_summary = self._load_report("kill_switch_summary.json")
        item = self._find_kill_switch(kill_switch_summary, "authority-election")

        self.assertEqual(summary.get("verdict"), "FAIL")
        self.assertEqual(kill_switch_summary.get("overall_status"), "FAIL")
        self.assertEqual(item.get("status"), "FAIL")
        self.assertEqual(item.get("failure_trigger"), "PRIMARY_GATE")
        self.assertEqual(item.get("primary_gate", {}).get("status"), "FAIL")

    def test_failing_supporting_gate_marks_category_as_supporting_failure(self) -> None:
        self._write_gate("convergence-non-election-boundary", "PASS")
        self._write_gate("cross-node-parity", "FAIL")

        proc = self._run()
        self.assertEqual(proc.returncode, 2)

        kill_switch_summary = self._load_report("kill_switch_summary.json")
        item = self._find_kill_switch(kill_switch_summary, "authority-election")
        self.assertEqual(item.get("status"), "FAIL")
        self.assertEqual(item.get("failure_trigger"), "SUPPORTING_GATE")
        failed_gates = {gate.get("gate") for gate in item.get("failed_gates", [])}
        self.assertEqual(failed_gates, {"cross-node-parity"})

    def test_skip_requires_reason(self) -> None:
        self._write_gate("observability-routing-separation", "SKIP")

        proc = self._run()
        self.assertEqual(proc.returncode, 2)

        summary = self._load_report("summary.json")
        kill_switch_summary = self._load_report("kill_switch_summary.json")
        item = self._find_kill_switch(kill_switch_summary, "observability-control-plane")
        self.assertEqual(summary.get("verdict"), "FAIL")
        self.assertEqual(item.get("status"), "FAIL")
        self.assertEqual(
            item.get("primary_gate", {}).get("summary_violation"),
            "skip_requires_reason",
        )

    def test_skip_with_reason_is_accepted(self) -> None:
        self._write_gate(
            "observability-routing-separation",
            "SKIP",
            extra={"skip_reason": "phase13_not_enabled"},
        )

        proc = self._run()
        self.assertEqual(proc.returncode, 0, proc.stderr)

        summary = self._load_report("summary.json")
        kill_switch_summary = self._load_report("kill_switch_summary.json")
        item = self._find_kill_switch(kill_switch_summary, "observability-control-plane")
        self.assertEqual(summary.get("verdict"), "PASS")
        self.assertEqual(kill_switch_summary.get("overall_status"), "PARTIAL")
        self.assertEqual(item.get("status"), "PASS")
        self.assertEqual(item.get("primary_gate", {}).get("verdict"), "SKIP")
        self.assertEqual(
            item.get("primary_gate", {}).get("skip_reason"),
            "phase13_not_enabled",
        )

    def test_strict_completeness_fails_when_expected_architectural_gate_is_missing(self) -> None:
        self._write_gate("observability-routing-separation", "PASS")

        proc = self._run(require_kill_switch_completeness=True)
        self.assertEqual(proc.returncode, 2)

        kill_switch_summary = self._load_report("kill_switch_summary.json")
        coverage = kill_switch_summary.get("coverage", {})
        self.assertTrue(kill_switch_summary.get("completeness_required"))
        self.assertEqual(coverage.get("coverage_status"), "INCOMPLETE")
        self.assertIn("proof-verdict-binding", coverage.get("missing_gates", []))

    def test_strict_completeness_passes_when_all_expected_architectural_gates_are_present(self) -> None:
        for gate in self.ALL_EXPECTED_GATES:
            self._write_gate(gate, "PASS")

        proc = self._run(require_kill_switch_completeness=True)
        self.assertEqual(proc.returncode, 0, proc.stderr)

        kill_switch_summary = self._load_report("kill_switch_summary.json")
        coverage = kill_switch_summary.get("coverage", {})
        self.assertTrue(kill_switch_summary.get("completeness_required"))
        self.assertEqual(coverage.get("coverage_status"), "COMPLETE")
        self.assertEqual(coverage.get("missing_gates"), [])
        self.assertEqual(kill_switch_summary.get("overall_status"), "PASS")

    def _run(
        self, *, require_kill_switch_completeness: bool = False
    ) -> subprocess.CompletedProcess[str]:
        cmd = ["bash", str(self.script), "--run-dir", str(self.run_dir)]
        if require_kill_switch_completeness:
            cmd.append("--require-kill-switch-completeness")
        return subprocess.run(
            cmd,
            cwd=self.repo_root,
            check=False,
            capture_output=True,
            text=True,
        )

    def _write_gate(self, gate: str, verdict: str, extra: dict | None = None) -> None:
        extra = extra or {}
        gate_dir = self.run_dir / "gates" / gate
        gate_dir.mkdir(parents=True, exist_ok=True)
        (gate_dir / "report.json").write_text(
            json.dumps(
                {
                    "gate": gate,
                    "verdict": verdict,
                    "violations_count": 0 if verdict == "PASS" else 1,
                    **extra,
                },
                indent=2,
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )

    def _load_report(self, name: str) -> dict:
        return json.loads((self.run_dir / "reports" / name).read_text(encoding="utf-8"))

    def _find_kill_switch(self, payload: dict, kill_switch_id: str) -> dict:
        for item in payload.get("kill_switches", []):
            if item.get("kill_switch_id") == kill_switch_id:
                return item
        self.fail(f"kill switch {kill_switch_id!r} not found")


if __name__ == "__main__":
    unittest.main()
