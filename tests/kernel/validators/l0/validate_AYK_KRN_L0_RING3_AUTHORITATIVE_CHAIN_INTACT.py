"""AYKEN kernel-level proof validator for authoritative chain integrity."""

VALIDATOR_ID = "AYK_KRN_L0_RING3_AUTHORITATIVE_CHAIN_INTACT"
ERROR_CODE = "AYK-E103"
DESCRIPTION = "Authoritative marker chain must be complete and strictly ordered."

EXPECTED_CHAIN = [
    "P10_TEXT_FRAME_WITNESS",
    "P10_POST_CR3_TEXT_PROBE",
    "P10_RING3_USER_CODE",
]


def validate(payload: dict) -> dict:
    witness_line = payload.get("selected_witness_line")
    probe_line = payload.get("selected_probe_line")
    user_line = payload.get("selected_user_marker_line")
    counts = payload.get("observed_counts", {})
    runtime_rule_violations = payload.get("runtime_rule_violations", [])
    chain = payload.get("authoritative_chain", [])

    ok = (
        chain == EXPECTED_CHAIN
        and witness_line is not None
        and probe_line is not None
        and user_line is not None
        and int(counts.get("pre_dispatch_witness", 0)) >= 1
        and int(counts.get("post_cr3_probe", 0)) >= 1
        and int(counts.get("user_marker", 0)) >= 1
        and int(witness_line) < int(probe_line) < int(user_line)
        and not runtime_rule_violations
    )
    if ok:
        return {
            "verdict": "PASS",
            "message": "authoritative chain is complete and ordered",
            "details": {
                "authoritative_chain": chain,
                "selected_witness_line": witness_line,
                "selected_probe_line": probe_line,
                "selected_user_marker_line": user_line,
            },
        }

    return {
        "verdict": "FAIL",
        "message": "authoritative chain is incomplete, unordered, or internally violated",
        "details": {
            "authoritative_chain": chain,
            "selected_witness_line": witness_line,
            "selected_probe_line": probe_line,
            "selected_user_marker_line": user_line,
            "observed_counts": counts,
            "runtime_rule_violations": runtime_rule_violations,
        },
    }
