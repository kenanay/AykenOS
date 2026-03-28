"""AYKEN kernel-level proof validator for executable-leaf source guard."""

VALIDATOR_ID = "AYK_KRN_L0_MMU_EXEC_LEAF_SOURCE_GUARD"
ERROR_CODE = "AYK-E104"
DESCRIPTION = "Executable leaf source guard must pass without violations."


def validate(payload: dict) -> dict:
    verdict = payload.get("source_guard_verdict")
    violations = payload.get("source_guard_violations", [])
    if verdict == "PASS" and not violations:
        return {
            "verdict": "PASS",
            "message": "executable leaf source guard passed",
            "details": {
                "source_guard_verdict": verdict,
            },
        }

    return {
        "verdict": "FAIL",
        "message": "executable leaf source guard failed",
        "details": {
            "source_guard_verdict": verdict,
            "source_guard_violations": violations,
        },
    }
