"""AYKEN kernel-level proof validator for user-code reachability."""

VALIDATOR_ID = "AYK_KRN_L0_RING3_USER_CODE_REACHED"
ERROR_CODE = "AYK-E102"
DESCRIPTION = "Authoritative user marker must be reached under the same CR3."


def validate(payload: dict) -> dict:
    user_code_reached = bool(payload.get("user_code_reached"))
    if user_code_reached:
        return {
            "verdict": "PASS",
            "message": "user marker reached under authoritative runtime chain",
            "details": {
                "user_code_reached": True,
                "selected_root": payload.get("selected_root"),
            },
        }
    return {
        "verdict": "FAIL",
        "message": "user marker was not reached under authoritative runtime chain",
        "details": {
            "user_code_reached": False,
            "authoritative_chain": payload.get("authoritative_chain", []),
        },
    }
