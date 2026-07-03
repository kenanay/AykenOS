from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
VALIDATOR = REPO_ROOT / "tools" / "phase21_first_bounded_validator" / "validator_skeleton.py"
BOUNDARY = REPO_ROOT / "docs" / "specs" / "phase21-first-bounded-implementation" / "PACKAGE_BOUNDARY.md"


def test_validator_skeleton_has_no_runtime_entrypoint():
    text = VALIDATOR.read_text(encoding="utf-8")

    denied_tokens = [
        "if __name__",
        "subprocess",
        "socket",
        "urllib",
        "requests",
        "open(",
        "write(",
        "Popen",
        "exec(",
        "eval(",
        "import ayken",
        "package_load",
        "load_package",
    ]

    for token in denied_tokens:
        assert token not in text


def test_package_boundary_preserves_denied_authorities():
    text = BOUNDARY.read_text(encoding="utf-8")

    required_denials = [
        "Runtime implementation procedure",
        "Code execution",
        "Process start",
        "Runtime state creation",
        "Package installation, loading, or execution",
        "Capability issuance",
        "Registry publication",
        "Trust assignment",
        "Source merge authority",
    ]

    for denial in required_denials:
        assert denial in text
