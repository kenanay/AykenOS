#!/usr/bin/env python3
"""Black-box tests for gate_proofd_service.sh."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


class ProofdServiceGateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.repo_root = Path(__file__).resolve().parents[2]
        self.evidence_dir = self.root / "proofd-service"
        self.script = self.repo_root / "scripts/ci/gate_proofd_service.sh"

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_gate_passes_and_exports_required_artifacts(self) -> None:
        proc = subprocess.run(
            ["bash", str(self.script), "--evidence-dir", str(self.evidence_dir)],
            cwd=self.repo_root,
            check=False,
        )
        self.assertEqual(proc.returncode, 0)

        report = json.loads((self.evidence_dir / "report.json").read_text(encoding="utf-8"))
        service = json.loads(
            (self.evidence_dir / "proofd_service_report.json").read_text(encoding="utf-8")
        )
        receipt = json.loads(
            (self.evidence_dir / "proofd_receipt_report.json").read_text(encoding="utf-8")
        )
        receipt_verification = json.loads(
            (self.evidence_dir / "proofd_receipt_verification_report.json").read_text(
                encoding="utf-8"
            )
        )
        repeated_execution = json.loads(
            (self.evidence_dir / "proofd_repeated_execution_report.json").read_text(
                encoding="utf-8"
            )
        )
        contract = json.loads(
            (self.evidence_dir / "proofd_endpoint_contract.json").read_text(
                encoding="utf-8"
            )
        )

        self.assertEqual(report.get("verdict"), "PASS")
        self.assertEqual(report.get("violations_count"), 0)
        self.assertEqual(service.get("status"), "PASS")
        self.assertEqual(
            service.get("service_mode"),
            "verification_execution_and_read_only_diagnostics",
        )
        self.assertEqual(service.get("receipt_mode"), "emit_signed")
        self.assertTrue(service.get("root_passthrough_ok") is True)
        self.assertTrue(service.get("run_scoped_passthrough_ok") is True)
        self.assertTrue(service.get("deterministic_repeated_read_ok") is True)
        self.assertTrue(service.get("deterministic_repeated_execution_ok") is True)
        self.assertTrue(service.get("verification_execution_active") is True)
        self.assertTrue(service.get("explicit_policy_binding_active") is True)
        self.assertTrue(service.get("explicit_registry_binding_active") is True)
        self.assertTrue(service.get("receipt_emission_active") is True)
        self.assertTrue(service.get("signed_receipt_execution_active") is True)
        self.assertTrue(service.get("signed_receipt_verified") is True)
        self.assertTrue(service.get("receipt_authority_binding_verified") is True)
        self.assertTrue(service.get("request_bound_timestamp_preserved") is True)
        self.assertTrue(service.get("repeated_receipt_bytes_equal") is True)
        self.assertTrue(service.get("repeated_run_manifest_equal") is True)
        self.assertTrue(service.get("diagnostics_artifacts_unchanged") is True)
        self.assertTrue(service.get("run_artifact_merge_detected") is False)
        self.assertTrue(service.get("closure_complete") is True)
        self.assertEqual(service.get("run_count"), 1)
        self.assertEqual(service.get("run_id"), "run-proofd-local-r1")
        self.assertEqual(
            service.get("endpoint_contract_path"), "proofd_endpoint_contract.json"
        )

        self.assertEqual(receipt.get("status"), "PASS")
        self.assertEqual(receipt.get("receipt_mode"), "emit_signed")
        self.assertTrue(receipt.get("receipt_boundary_preserved") is True)
        self.assertTrue(receipt.get("receipt_emission_active") is True)
        self.assertTrue(receipt.get("signed_receipt_verified") is True)
        self.assertTrue(receipt.get("receipt_authority_verified") is True)
        self.assertGreaterEqual(receipt.get("signed_receipt_findings_count", -1), 0)
        self.assertGreaterEqual(receipt.get("receipt_authority_findings_count", -1), 0)
        self.assertTrue(receipt.get("receipt_authority_chain_id") is not None)
        self.assertTrue(receipt.get("request_bound_timestamp_preserved") is True)
        self.assertTrue(receipt.get("receipt_endpoint_exposed") is False)
        self.assertTrue(receipt.get("proofd_recomputes_receipts") is False)
        self.assertTrue(receipt.get("proofd_reinterprets_receipts") is False)
        self.assertTrue(receipt.get("closure_complete") is True)
        self.assertEqual(
            receipt.get("receipt_path"), "receipts/verification_receipt.json"
        )
        self.assertEqual(
            receipt.get("reason"), "closure_ready_final_hardening_green"
        )

        self.assertEqual(receipt_verification.get("status"), "PASS")
        self.assertTrue(receipt_verification.get("signed_receipt_verified") is True)
        self.assertTrue(receipt_verification.get("receipt_authority_verified") is True)
        self.assertTrue(
            receipt_verification.get("request_bound_timestamp_preserved") is True
        )
        self.assertTrue(receipt_verification.get("receipt_boundary_preserved") is True)

        self.assertEqual(repeated_execution.get("status"), "PASS")
        self.assertTrue(repeated_execution.get("repeated_response_equal") is True)
        self.assertTrue(
            repeated_execution.get("repeated_receipt_bytes_equal") is True
        )
        self.assertTrue(
            repeated_execution.get("repeated_run_manifest_equal") is True
        )
        self.assertTrue(
            repeated_execution.get("diagnostics_artifacts_unchanged") is True
        )
        self.assertTrue(repeated_execution.get("run_artifact_merge_detected") is False)

        self.assertEqual(contract.get("status"), "PASS")
        self.assertEqual(contract.get("mode"), "phase12_proofd_service_gate_execution_slice")
        self.assertEqual(contract.get("run_id"), "run-proofd-local-r1")
        self.assertGreaterEqual(contract.get("endpoint_count", 0), 20)
        checks = contract.get("endpoint_checks", [])
        self.assertTrue(any(item.get("endpoint") == "/diagnostics/parity" for item in checks))
        self.assertTrue(any(item.get("endpoint") == "/verify/bundle" for item in checks))
        self.assertTrue(
            any(
                item.get("endpoint")
                == "/diagnostics/runs/run-proofd-local-r1/authority-topology"
                for item in checks
            )
        )
        self.assertTrue(
            any(
                item.get("endpoint") == "/diagnostics/runs/run-proofd-local-r1/drift"
                for item in checks
            )
        )
        self.assertTrue(
            any(
                item.get("endpoint")
                == "/diagnostics/runs/run-proofd-local-r1/convergence"
                for item in checks
            )
        )
        self.assertEqual(
            contract.get("verify_request_path"), "proofd_verify_request.json"
        )
        self.assertEqual(
            contract.get("verify_response_path"), "proofd_verify_response.json"
        )

        self.assertTrue((self.evidence_dir / "violations.txt").is_file())
        self.assertEqual(
            (self.evidence_dir / "violations.txt").read_text(encoding="utf-8"), ""
        )
        self.assertTrue((self.evidence_dir / "service-root").is_dir())
        self.assertTrue((self.evidence_dir / "proofd_verify_request.json").is_file())
        self.assertTrue((self.evidence_dir / "proofd_verify_response.json").is_file())
        self.assertTrue((self.evidence_dir / "proofd_run_manifest.json").is_file())
        self.assertTrue(
            (self.evidence_dir / "proofd_receipt_verification_report.json").is_file()
        )
        self.assertTrue(
            (self.evidence_dir / "proofd_repeated_execution_report.json").is_file()
        )
        verify_request = json.loads(
            (self.evidence_dir / "proofd_verify_request.json").read_text(encoding="utf-8")
        )
        self.assertEqual(verify_request.get("receipt_mode"), "emit_signed")
        self.assertTrue(isinstance(verify_request.get("receipt_signer"), dict))


if __name__ == "__main__":
    unittest.main()
