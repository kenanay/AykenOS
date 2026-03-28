"""AYKEN kernel-level proof validator for witness/probe equality."""

VALIDATOR_ID = "AYK_KRN_L0_RING3_WITNESS_EQ_PROBE"
ERROR_CODE = "AYK-E101"
DESCRIPTION = "Pre-dispatch witness qword must match post-CR3 probe qword."


def validate(payload: dict) -> dict:
    witness_q = payload.get("selected_witness_qword")
    probe_q = payload.get("selected_probe_qword")
    if witness_q == probe_q and witness_q is not None:
        return {
            "verdict": "PASS",
            "message": "witness qword matches post-CR3 probe qword",
            "details": {
                "selected_witness_qword": witness_q,
                "selected_probe_qword": probe_q,
            },
        }
    return {
        "verdict": "FAIL",
        "message": "witness qword does not match post-CR3 probe qword",
        "details": {
            "expected": witness_q,
            "actual": probe_q,
        },
    }
