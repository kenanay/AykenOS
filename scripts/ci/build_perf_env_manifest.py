#!/usr/bin/env python3
import argparse
import hashlib
import json
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Build the canonical performance environment manifest and runtime env hash."
    )
    parser.add_argument("--output", required=True)
    parser.add_argument("--kernel-profile", required=True)
    parser.add_argument("--qemu-timeout-seconds", required=True, type=int)
    parser.add_argument("--clang-version", required=True)
    parser.add_argument("--ld-version", required=True)
    parser.add_argument("--nasm-version", required=True)
    parser.add_argument("--qemu-version", required=True)
    parser.add_argument("--host-os", required=True)
    parser.add_argument("--host-arch", required=True)
    parser.add_argument("--baseline-authority", required=True)
    parser.add_argument("--ci-image-digest", required=True)
    parser.add_argument("--boot-ok-marker", required=True)
    parser.add_argument("--preempt-sw-count-pattern", required=True)
    parser.add_argument("--preempt-iret-count-pattern", required=True)
    parser.add_argument("--measurement-contract", required=True)
    parser.add_argument("--preempt-user-minimal-mode", required=True)
    parser.add_argument("--preempt-bootstrap-policy", required=True, type=int)
    parser.add_argument("--preempt-mb-selftest", required=True, type=int)
    parser.add_argument("--preempt-deterministic-exit", required=True, type=int)
    parser.add_argument("--preempt-ring3-entry-guard", required=True, type=int)
    parser.add_argument("--preempt-expected-qemu-exit-set", required=True)
    args = parser.parse_args()

    payload = {
        "kernel_profile": args.kernel_profile,
        "target_triple": "x86_64-elf",
        "qemu_timeout_seconds": args.qemu_timeout_seconds,
        "clang_version": args.clang_version,
        "ld_version": args.ld_version,
        "nasm_version": args.nasm_version,
        "qemu_version": args.qemu_version,
        "host_os": args.host_os,
        "host_arch": args.host_arch,
        "baseline_authority": args.baseline_authority,
        "ci_image_digest": args.ci_image_digest,
        "marker_contract": {
            "boot_ok_marker": args.boot_ok_marker,
            "preempt_sw_count_pattern": args.preempt_sw_count_pattern,
            "preempt_iret_count_pattern": args.preempt_iret_count_pattern,
            "measurement_contract": args.measurement_contract,
            "preempt_user_minimal_mode": args.preempt_user_minimal_mode,
            "preempt_bootstrap_policy": args.preempt_bootstrap_policy,
            "preempt_mb_selftest": args.preempt_mb_selftest,
            "preempt_deterministic_exit": args.preempt_deterministic_exit,
            "preempt_ring3_entry_guard": args.preempt_ring3_entry_guard,
            "preempt_expected_qemu_exit_set": args.preempt_expected_qemu_exit_set,
        },
    }

    hash_payload = dict(payload)
    if str(payload.get("baseline_authority", "")).startswith("github-hosted-"):
        # Keep digest for audit, but derive the hash from the stable authority surface.
        hash_payload.pop("ci_image_digest", None)
    canonical = json.dumps(hash_payload, sort_keys=True, separators=(",", ":"))
    payload["env_hash"] = hashlib.sha256(canonical.encode("utf-8")).hexdigest()

    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(payload["env_hash"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
