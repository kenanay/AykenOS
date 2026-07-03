"""Phase-21 first bounded implementation static validator skeleton.

This module is intentionally non-executing and non-authoritative. It has no
CLI entrypoint, no child-process use, no filesystem mutation, no network
access, no package-load behavior, no AykenOS runtime import, and no
authoritative verdict.
"""

SKELETON_ID = "ayken.phase21.first_bounded.static_validator_skeleton.v1"

SKELETON_POSTURE = {
    "userspace_only": True,
    "non_executing": True,
    "static_validation_only": True,
    "authoritative_verdict": False,
}

DENIED_AUTHORITIES = (
    "runtime_implementation_procedure",
    "code_execution",
    "process_start",
    "runtime_state_creation",
    "package_installation",
    "package-load",
    "package_execution",
    "capability_issuance",
    "registry_publication",
    "trust_assignment",
    "source_acceptance",
    "source_merge_authority",
)


def describe_skeleton_contract():
    """Return static contract metadata for inspection only.

    The returned metadata is not a verdict, not proof, not evidence
    acceptance, not package acceptance, and not source merge authority.
    """

    return {
        "skeleton_id": SKELETON_ID,
        "posture": dict(SKELETON_POSTURE),
        "denied_authorities": tuple(DENIED_AUTHORITIES),
        "verdict": "not_authoritative",
    }
