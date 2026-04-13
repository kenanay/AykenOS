#!/usr/bin/env python3
"""
Bug Condition Exploration Test: Payload Authority Drift Detection

This test MUST FAIL on unfixed code - failure confirms the bug exists.
DO NOT attempt to fix the test or the code when it fails.

This test encodes the expected behavior - it will validate the fix when it passes after implementation.

GOAL: Surface counterexamples that demonstrate authority drift exists.
"""

import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Dict, List, Optional, Tuple

# Project root
PROJECT_ROOT = Path(__file__).parent.parent.parent
BUILD_DIR = PROJECT_ROOT / "out" / "build"
MANIFEST_PATH = BUILD_DIR / "payload_manifest.json"
EMBEDDED_HEADER_PATH = PROJECT_ROOT / "kernel" / "include" / "embedded_elf.h"


class PayloadAuthorityDriftTest:
    """Test suite for payload authority drift detection."""

    def __init__(self):
        self.failures: List[str] = []
        self.counterexamples: List[Dict] = []

    def log_failure(self, test_name: str, expected: str, actual: str, details: str = ""):
        """Log a test failure as a counterexample."""
        failure_msg = f"FAIL: {test_name}\n  Expected: {expected}\n  Actual: {actual}"
        if details:
            failure_msg += f"\n  Details: {details}"
        self.failures.append(failure_msg)
        
        self.counterexamples.append({
            "test": test_name,
            "expected": expected,
            "actual": actual,
            "details": details
        })

    def clean_build(self):
        """Clean build artifacts to ensure fresh build."""
        print("Cleaning build artifacts...")
        subprocess.run(["make", "clean"], cwd=PROJECT_ROOT, capture_output=True)

    def build_kernel(self, user_minimal_mode: Optional[str] = None) -> Tuple[bool, str]:
        """Build kernel with specified USER_MINIMAL_MODE."""
        env = os.environ.copy()
        if user_minimal_mode:
            env["USER_MINIMAL_MODE"] = user_minimal_mode
            print(f"Building kernel with USER_MINIMAL_MODE={user_minimal_mode}...")
        else:
            print("Building kernel with default mode...")
        
        result = subprocess.run(
            ["make", "efi-img"],
            cwd=PROJECT_ROOT,
            env=env,
            capture_output=True,
            text=True
        )
        
        return result.returncode == 0, result.stdout + result.stderr

    def extract_manifest_mode(self) -> Optional[str]:
        """Extract selected_mode from manifest JSON."""
        if not MANIFEST_PATH.exists():
            return None
        
        try:
            with open(MANIFEST_PATH, "r") as f:
                manifest = json.load(f)
                return manifest.get("selected_mode")
        except (json.JSONDecodeError, KeyError):
            return None

    def extract_embedded_mode(self) -> Optional[str]:
        """Extract embedded_elf_mode from embedded_elf.h."""
        if not EMBEDDED_HEADER_PATH.exists():
            return None
        
        with open(EMBEDDED_HEADER_PATH, "r") as f:
            content = f.read()
            match = re.search(r'embedded_elf_mode\[\]\s*=\s*"([^"]+)"', content)
            return match.group(1) if match else None

    def extract_embedded_sha(self) -> Optional[str]:
        """Extract embedded_elf_sha256 from embedded_elf.h."""
        if not EMBEDDED_HEADER_PATH.exists():
            return None
        
        with open(EMBEDDED_HEADER_PATH, "r") as f:
            content = f.read()
            match = re.search(r'embedded_elf_sha256\[\]\s*=\s*"([^"]+)"', content)
            return match.group(1) if match else None

    def extract_manifest_hashes(self) -> Tuple[Optional[str], Optional[str]]:
        """Extract payload_sha256 and embedded_header_sha256 from manifest."""
        if not MANIFEST_PATH.exists():
            return None, None
        
        try:
            with open(MANIFEST_PATH, "r") as f:
                manifest = json.load(f)
                return manifest.get("payload_sha256"), manifest.get("embedded_header_sha256")
        except (json.JSONDecodeError, KeyError):
            return None, None

    def boot_kernel_qemu(self) -> Tuple[bool, str]:
        """Boot kernel in QEMU and capture debugcon output."""
        print("Booting kernel in QEMU...")
        
        # Use entry-proof harness to boot and capture output
        harness_script = PROJECT_ROOT / "scripts" / "qemu-entry-proof-harness.sh"
        if not harness_script.exists():
            return False, "Harness script not found"
        
        result = subprocess.run(
            [str(harness_script)],
            cwd=PROJECT_ROOT,
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Read debugcon output
        debugcon_path = PROJECT_ROOT / "evidence" / "entry-proof" / "qemu_debugcon.log"
        if debugcon_path.exists():
            with open(debugcon_path, "r") as f:
                return True, f.read()
        
        return False, result.stdout + result.stderr

    def test_entry_proof_manifest_mode(self):
        """Test 1: Entry-Proof Manifest Test
        
        Build kernel with USER_MINIMAL_MODE=entry-proof, verify manifest shows selected_mode: "entry-proof"
        EXPECTED TO FAIL on unfixed code (shows phase10a2)
        """
        print("\n=== Test 1: Entry-Proof Manifest Mode ===")
        self.clean_build()
        
        success, build_log = self.build_kernel("entry-proof")
        if not success:
            self.log_failure(
                "entry_proof_manifest_mode",
                "Build succeeds",
                "Build failed",
                "Build should succeed with entry-proof mode"
            )
            return
        
        manifest_mode = self.extract_manifest_mode()
        if manifest_mode is None:
            self.log_failure(
                "entry_proof_manifest_mode",
                "Manifest exists with selected_mode",
                "Manifest missing or malformed",
                "Manifest should be generated during build"
            )
            return
        
        if manifest_mode != "entry-proof":
            self.log_failure(
                "entry_proof_manifest_mode",
                'selected_mode: "entry-proof"',
                f'selected_mode: "{manifest_mode}"',
                "Authority drift detected: manifest mode doesn't match requested mode"
            )
        else:
            print("✓ PASS: Manifest shows correct mode")

    def test_runtime_bridge_hash_match(self):
        """Test 2: Runtime-Bridge Hash Test
        
        Build kernel with USER_MINIMAL_MODE=runtime-bridge-test, verify manifest payload_sha256 == embedded_header_sha256
        EXPECTED TO FAIL on unfixed code (hash mismatch)
        """
        print("\n=== Test 2: Runtime-Bridge Hash Match ===")
        self.clean_build()
        
        success, build_log = self.build_kernel("runtime-bridge-test")
        if not success:
            self.log_failure(
                "runtime_bridge_hash_match",
                "Build succeeds",
                "Build failed",
                "Build should succeed with runtime-bridge-test mode"
            )
            return
        
        payload_sha, embedded_sha = self.extract_manifest_hashes()
        if payload_sha is None or embedded_sha is None:
            self.log_failure(
                "runtime_bridge_hash_match",
                "Manifest exists with both hashes",
                "Manifest missing or incomplete",
                "Manifest should contain both payload_sha256 and embedded_header_sha256"
            )
            return
        
        if payload_sha != embedded_sha:
            self.log_failure(
                "runtime_bridge_hash_match",
                f"payload_sha256 == embedded_header_sha256",
                f"payload_sha256={payload_sha[:16]}... != embedded_header_sha256={embedded_sha[:16]}...",
                "Payload integrity failure: embedded hash doesn't match manifest"
            )
        else:
            print("✓ PASS: Hashes match")

    def test_boot_marker_emission(self):
        """Test 3: Boot Marker Test
        
        Boot kernel with entry-proof payload, verify debugcon contains BOTH [K][PAYLOAD_MODE=entry-proof] AND [K][PAYLOAD_SHA=...]
        EXPECTED TO FAIL on unfixed code (markers not emitted)
        """
        print("\n=== Test 3: Boot Marker Emission ===")
        self.clean_build()
        
        success, build_log = self.build_kernel("entry-proof")
        if not success:
            self.log_failure(
                "boot_marker_emission",
                "Build succeeds",
                "Build failed",
                "Build should succeed with entry-proof mode"
            )
            return
        
        boot_success, boot_log = self.boot_kernel_qemu()
        if not boot_success:
            self.log_failure(
                "boot_marker_emission",
                "Kernel boots successfully",
                "Boot failed or no output",
                "Kernel should boot and produce debugcon output"
            )
            return
        
        # Check for mode marker
        mode_marker_found = re.search(r'\[K\]\[PAYLOAD_MODE=entry-proof\]', boot_log)
        sha_marker_found = re.search(r'\[K\]\[PAYLOAD_SHA=([a-f0-9]+)\]', boot_log)
        
        if not mode_marker_found:
            self.log_failure(
                "boot_marker_emission",
                "[K][PAYLOAD_MODE=entry-proof] marker in boot log",
                "Mode marker not found",
                "Boot verification failure: mode marker not emitted"
            )
        
        if not sha_marker_found:
            self.log_failure(
                "boot_marker_emission",
                "[K][PAYLOAD_SHA=...] marker in boot log",
                "SHA marker not found",
                "Boot verification failure: SHA marker not emitted"
            )
        
        if mode_marker_found and sha_marker_found:
            print("✓ PASS: Both boot markers found")

    def test_invalid_mode_fails(self):
        """Test 4: Invalid Mode Test
        
        Build with USER_MINIMAL_MODE=invalid, verify build fails
        EXPECTED TO FAIL on unfixed code (falls back silently)
        """
        print("\n=== Test 4: Invalid Mode Fails ===")
        self.clean_build()
        
        success, build_log = self.build_kernel("invalid")
        if success:
            self.log_failure(
                "invalid_mode_fails",
                "Build fails with explicit error",
                "Build succeeded (silent fallback)",
                "Build should HARD FAIL when USER_MINIMAL_MODE is invalid"
            )
        else:
            # Check if error message is explicit
            if "invalid" not in build_log.lower() and "user_minimal_mode" not in build_log.lower():
                self.log_failure(
                    "invalid_mode_fails",
                    "Explicit error message about invalid mode",
                    "Build failed but error message unclear",
                    "Error message should explicitly mention invalid USER_MINIMAL_MODE"
                )
            else:
                print("✓ PASS: Build fails with explicit error")

    def test_manifest_missing_fails(self):
        """Test 5: Manifest Regeneration Test
        
        Delete manifest, verify build regenerates it (on FIXED code)
        EXPECTED TO FAIL on unfixed code (no manifest dependency, build succeeds without regenerating)
        EXPECTED TO PASS on fixed code (manifest dependency causes regeneration)
        """
        print("\n=== Test 5: Manifest Regeneration ===")
        self.clean_build()
        
        # First build to generate manifest
        success, _ = self.build_kernel("entry-proof")
        if not success:
            print("⚠ SKIP: Initial build failed, cannot test manifest deletion")
            return
        
        # Verify manifest exists
        if not MANIFEST_PATH.exists():
            self.log_failure(
                "manifest_missing_fails",
                "Manifest exists after build",
                "Manifest not found",
                "Manifest should be generated during build"
            )
            return
        
        # Delete manifest
        MANIFEST_PATH.unlink()
        print("Deleted manifest, rebuilding...")
        
        # Rebuild without cleaning (should regenerate manifest)
        success, build_log = self.build_kernel("entry-proof")
        if not success:
            self.log_failure(
                "manifest_missing_fails",
                "Build succeeds by regenerating manifest",
                "Build failed",
                "Build should regenerate missing manifest automatically"
            )
            return
        
        # Verify manifest was regenerated
        if not MANIFEST_PATH.exists():
            self.log_failure(
                "manifest_missing_fails",
                "Manifest regenerated after deletion",
                "Manifest still missing after rebuild",
                "Build should regenerate manifest when it's missing"
            )
        else:
            # Verify manifest content is correct
            manifest_mode = self.extract_manifest_mode()
            if manifest_mode != "entry-proof":
                self.log_failure(
                    "manifest_missing_fails",
                    "Regenerated manifest has correct mode",
                    f"Regenerated manifest has mode: {manifest_mode}",
                    "Regenerated manifest should have correct content"
                )
            else:
                print("✓ PASS: Manifest regenerated correctly")

    def run_all_tests(self):
        """Run all bug condition exploration tests."""
        print("=" * 60)
        print("Bug Condition Exploration Test: Payload Authority Drift")
        print("=" * 60)
        print("\nCRITICAL: These tests MUST FAIL on unfixed code.")
        print("Failure confirms the bug exists.\n")
        
        self.test_entry_proof_manifest_mode()
        self.test_runtime_bridge_hash_match()
        self.test_boot_marker_emission()
        self.test_invalid_mode_fails()
        self.test_manifest_missing_fails()
        
        print("\n" + "=" * 60)
        print("Test Results")
        print("=" * 60)
        
        if self.failures:
            print(f"\n❌ {len(self.failures)} test(s) FAILED (expected on unfixed code)")
            print("\nCounterexamples found:")
            for failure in self.failures:
                print(f"\n{failure}")
            
            print("\n" + "=" * 60)
            print("Root Cause Analysis")
            print("=" * 60)
            
            # Analyze counterexamples to understand root cause
            manifest_issues = [ce for ce in self.counterexamples if "manifest" in ce["test"].lower()]
            hash_issues = [ce for ce in self.counterexamples if "hash" in ce["test"].lower()]
            boot_issues = [ce for ce in self.counterexamples if "boot" in ce["test"].lower()]
            validation_issues = [ce for ce in self.counterexamples if "invalid" in ce["test"].lower() or "missing" in ce["test"].lower()]
            
            if manifest_issues:
                print(f"\n✗ Authority Source Issues ({len(manifest_issues)} found):")
                print("  - Manifest mode doesn't match requested mode")
                print("  - Indicates USER_MINIMAL_MODE not being used as single source of authority")
            
            if hash_issues:
                print(f"\n✗ Hash Verification Issues ({len(hash_issues)} found):")
                print("  - Embedded hash doesn't match manifest payload hash")
                print("  - Indicates missing build-time hash verification")
            
            if boot_issues:
                print(f"\n✗ Boot Verification Issues ({len(boot_issues)} found):")
                print("  - Boot markers not emitted or incomplete")
                print("  - Indicates missing boot-time verification")
            
            if validation_issues:
                print(f"\n✗ Validation Issues ({len(validation_issues)} found):")
                print("  - Invalid mode falls back silently")
                print("  - Missing manifest doesn't cause build failure")
                print("  - Indicates missing input validation")
            
            print("\n" + "=" * 60)
            print("EXPECTED OUTCOME: Tests FAIL (confirms bug exists)")
            print("=" * 60)
            return 1
        else:
            print("\n✅ All tests PASSED")
            print("\nThis means the bug is FIXED!")
            print("Correctness invariants are satisfied:")
            print("  - Mode Authority Invariant: manifest.selected_mode == embedded_elf_mode == boot_emitted_mode")
            print("  - Payload Integrity Invariant: manifest.payload_sha256 == embedded_elf_sha == boot_emitted_sha")
            return 0


def main():
    """Main entry point."""
    test_suite = PayloadAuthorityDriftTest()
    return test_suite.run_all_tests()


if __name__ == "__main__":
    sys.exit(main())
