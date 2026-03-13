#!/usr/bin/env python3
"""Black-box tests for Phase-12A gate harness modes."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class Phase12AGateSuiteTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.script = self.repo_root / "scripts/ci/gate_phase12_harness.sh"

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def run_gate(self, mode: str) -> Path:
        evidence_dir = self.root / mode
        proc = subprocess.run(
            [
                "bash",
                str(self.script),
                "--mode",
                mode,
                "--evidence-dir",
                str(evidence_dir),
            ],
            cwd=self.repo_root,
            check=False,
        )
        self.assertEqual(proc.returncode, 0, msg=f"{mode} gate returned {proc.returncode}")
        self.assertTrue((evidence_dir / "violations.txt").is_file())
        self.assertEqual((evidence_dir / "violations.txt").read_text(encoding="utf-8"), "")
        report = json.loads((evidence_dir / "report.json").read_text(encoding="utf-8"))
        self.assertEqual(report.get("verdict"), "PASS", msg=f"{mode} gate verdict not PASS")
        self.assertEqual(report.get("violations_count"), 0, msg=f"{mode} gate has violations")
        return evidence_dir

    def test_phase12a_gates_pass_and_export_required_artifacts(self) -> None:
        with self.subTest("producer-schema"):
            evidence_dir = self.run_gate("producer-schema")
            schema = json.loads(
                (evidence_dir / "producer_schema_report.json").read_text(encoding="utf-8")
            )
            examples = json.loads(
                (evidence_dir / "producer_identity_examples.json").read_text(encoding="utf-8")
            )
            self.assertEqual(schema.get("status"), "PASS")
            self.assertTrue(schema.get("bundle_id_stable_under_producer_rotation"))
            self.assertEqual(examples["current_example"]["producer_id"], "ayken-ci")
            self.assertEqual(examples["rotated_example"]["producer_id"], "ayken-ci")

        with self.subTest("signature-envelope"):
            evidence_dir = self.run_gate("signature-envelope")
            envelope = json.loads(
                (evidence_dir / "signature_envelope_report.json").read_text(encoding="utf-8")
            )
            identity = json.loads(
                (evidence_dir / "identity_stability_report.json").read_text(encoding="utf-8")
            )
            self.assertEqual(envelope.get("status"), "PASS")
            self.assertEqual(envelope.get("bundle_id_algorithm"), "sha256")
            self.assertTrue(identity.get("bundle_id_stable_under_envelope_mutation"))

        with self.subTest("bundle-v2-schema"):
            evidence_dir = self.run_gate("bundle-v2-schema")
            schema = json.loads(
                (evidence_dir / "bundle_schema_report.json").read_text(encoding="utf-8")
            )
            self.assertEqual(schema.get("status"), "PASS")
            self.assertEqual(schema.get("bundle_version"), 2)
            self.assertEqual(schema.get("mode_value"), "portable_proof_bundle_v2")
            self.assertEqual(schema.get("compatibility_mode"), "phase11-portable-core")

        with self.subTest("bundle-v2-compat"):
            evidence_dir = self.run_gate("bundle-v2-compat")
            compat = json.loads(
                (evidence_dir / "compatibility_report.json").read_text(encoding="utf-8")
            )
            self.assertEqual(compat.get("status"), "PASS")
            self.assertEqual(compat.get("compatibility_mode"), "phase11-portable-core")
            self.assertTrue(compat.get("portable_core_paths_present"))
            self.assertTrue(compat.get("overlay_is_external"))

        with self.subTest("signature-verify"):
            evidence_dir = self.run_gate("signature-verify")
            signature_verify = json.loads(
                (evidence_dir / "signature_verify.json").read_text(encoding="utf-8")
            )
            registry_resolution = json.loads(
                (evidence_dir / "registry_resolution_report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(signature_verify.get("status"), "PASS")
            self.assertEqual(signature_verify.get("bundle_id_algorithm"), "sha256")
            self.assertEqual(registry_resolution.get("status"), "PASS")
            self.assertEqual(registry_resolution.get("resolved_signer_count"), 1)

        with self.subTest("registry-resolution"):
            evidence_dir = self.run_gate("registry-resolution")
            matrix = json.loads(
                (evidence_dir / "registry_resolution_matrix.json").read_text(encoding="utf-8")
            )
            self.assertEqual(len(matrix), 4)
            self.assertEqual(matrix[0].get("primary_signer_status"), "ACTIVE")
            self.assertIn("PV0405", matrix[1].get("error_codes", []))
            self.assertIn("PV0404", matrix[2].get("error_codes", []))
            self.assertIn("PV0406", matrix[3].get("error_codes", []))
            self.assertIn("PV0408", matrix[3].get("error_codes", []))

        with self.subTest("key-rotation"):
            evidence_dir = self.run_gate("key-rotation")
            rotation = json.loads(
                (evidence_dir / "rotation_matrix.json").read_text(encoding="utf-8")
            )
            revocation = json.loads(
                (evidence_dir / "revocation_matrix.json").read_text(encoding="utf-8")
            )
            self.assertEqual(len(rotation), 2)
            self.assertEqual(rotation[0].get("primary_signer_status"), "ACTIVE")
            self.assertEqual(rotation[1].get("primary_signer_status"), "SUPERSEDED")
            self.assertEqual(rotation[1].get("signature_status"), "PASS")
            self.assertEqual(len(revocation), 1)
            self.assertEqual(revocation[0].get("primary_signer_status"), "REVOKED")
            self.assertIn("PV0403", revocation[0].get("resolution_error_codes", []))


if __name__ == "__main__":
    unittest.main()
