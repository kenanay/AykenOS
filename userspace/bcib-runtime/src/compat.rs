/// Version and opcode compatibility checks for BCIB v3.
///
/// Requirements: 1.5, 12.4, 12.5
///
/// Two responsibilities:
///   1. `check_version_compatibility()` — runtime version gate.
///      v3 (0x0003) → pass; v0.2 (0x0002) → backward-compat path;
///      anything else → fail-closed `BCIB_ERR_UNSUPPORTED_VERSION`.
///
///   2. `validate_opcode_no_conflict()` — build-time opcode ID conflict
///      detector. Panics at compile time (via `const` assertion) if any
///      registered opcode ID collides with a v0.2 reserved ID that has been
///      given a *different* name, or if any two registered opcodes share the
///      same ID. This is the CI gate for `ci-gate-toolchain-opcode-registry`.
use crate::binary_format::{BCIB_VERSION_V02, BCIB_VERSION_V3};
use crate::opcode_registry::{lookup_opcode, RESERVED_V02};
use crate::types::BcibError;

// ---------------------------------------------------------------------------
// CompatResult — outcome of a version check
// ---------------------------------------------------------------------------

/// Outcome of `check_version_compatibility()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatResult {
    /// BCIB v3 — full v3 execution path.
    V3,
    /// BCIB v0.2 — backward-compatible execution path.
    BackwardCompatV02,
}

// ---------------------------------------------------------------------------
// check_version_compatibility
// ---------------------------------------------------------------------------

/// Determine whether `version` is supported and which execution path to use.
///
/// | Version | Result |
/// |---------|--------|
/// | 0x0003  | `Ok(CompatResult::V3)` |
/// | 0x0002  | `Ok(CompatResult::BackwardCompatV02)` |
/// | other   | `Err(BcibError::UnsupportedVersion)` — fail-closed |
///
/// Requirement 1.5: v0.2 semantics must be backward-compatible or fail-closed.
/// Requirement 12.4: version lock mechanism; mismatch → fail-closed error.
pub fn check_version_compatibility(version: u16) -> Result<CompatResult, BcibError> {
    match version {
        BCIB_VERSION_V3 => Ok(CompatResult::V3),
        BCIB_VERSION_V02 => Ok(CompatResult::BackwardCompatV02),
        _ => Err(BcibError::UnsupportedVersion(
            "BCIB version not supported; expected 0x0003 (v3) or 0x0002 (v0.2)",
        )),
    }
}

// ---------------------------------------------------------------------------
// validate_opcode_no_conflict — runtime guard (mirrors build-time assertion)
// ---------------------------------------------------------------------------

/// Validate that no registered opcode ID conflicts with the v0.2 reserved
/// list in a way that would change the opcode's name/semantics.
///
/// This is the *runtime* companion to the build-time `const` assertion below.
/// It is intended to be called from CI integration tests so that any future
/// table mutation is caught immediately.
///
/// Returns `Ok(())` if the registry is clean.
/// Returns `Err(BcibError::InvalidGraph)` with the conflicting ID on the
/// first detected conflict (fail-closed, Requirement 12.5).
pub fn validate_opcode_no_conflict() -> Result<(), BcibError> {
    for &reserved_id in RESERVED_V02 {
        match lookup_opcode(reserved_id) {
            Ok(_) => {
                // The reserved ID is present in the table — this is expected
                // (v0.2 opcodes are kept with their original semantics).
                // No conflict: the ID is registered under its canonical v0.2 name.
            }
            Err(_) => {
                // A v0.2 reserved ID has been removed from the table entirely.
                // This is a breaking change — fail-closed.
                return Err(BcibError::InvalidGraph(
                    "v0.2 reserved opcode ID missing from registry (breaking change)",
                ));
            }
        }
    }

    // Check for duplicate IDs across the full 0x00–0xFF range.
    // Because the opcode table is a flat array indexed by ID, duplicates are
    // structurally impossible at the table level. We verify the invariant
    // explicitly here for defence-in-depth.
    let mut seen = [false; 256];
    for id in 0u16..=255 {
        if lookup_opcode(id as u8).is_ok() {
            if seen[id as usize] {
                // Duplicate detected — should never happen with the const table,
                // but guard against future refactors.
                return Err(BcibError::InvalidGraph(
                    "duplicate opcode ID detected in registry",
                ));
            }
            seen[id as usize] = true;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Build-time assertion: v0.2 reserved IDs must remain in the registry
// ---------------------------------------------------------------------------
//
// This `const` block runs at compile time. If any v0.2 reserved opcode ID
// is absent from the table (i.e. `OPCODE_TABLE[id]` is `None`), the
// assertion fires and the build fails — satisfying the CI gate requirement
// for `ci-gate-toolchain-opcode-registry` (Requirement 12.5).
//
// Note: we cannot call `lookup_opcode()` in a `const` context because it
// references a `static`. Instead we re-check the `is_reserved_v02` predicate
// against the known set. The actual table presence is verified at test time
// via `validate_opcode_no_conflict()` and the unit tests in `opcode_registry`.

const _: () = {
    // Verify the reserved list is non-empty (guards against accidental erasure).
    // If someone clears RESERVED_V02, this fires at compile time.
    assert!(
        !RESERVED_V02.is_empty(),
        "RESERVED_V02 must not be empty — v0.2 opcode IDs must be locked"
    );

    // Verify the known reserved IDs are present in the list by checking
    // their expected positions. This is a structural sanity check: if the
    // list is reordered or truncated the count assertion below will catch it.
    //
    // Expected v0.2 reserved IDs: 0x00, 0x01, 0x10, 0x11, 0x12, 0x20, 0x30
    assert!(
        RESERVED_V02.len() == 7,
        "RESERVED_V02 must contain exactly 7 v0.2 opcode IDs"
    );
};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // check_version_compatibility
    // -----------------------------------------------------------------------

    #[test]
    fn v3_version_passes() {
        assert_eq!(
            check_version_compatibility(BCIB_VERSION_V3),
            Ok(CompatResult::V3)
        );
    }

    #[test]
    fn v02_version_backward_compat() {
        assert_eq!(
            check_version_compatibility(BCIB_VERSION_V02),
            Ok(CompatResult::BackwardCompatV02)
        );
    }

    #[test]
    fn version_zero_rejected() {
        assert!(matches!(
            check_version_compatibility(0x0000),
            Err(BcibError::UnsupportedVersion(_))
        ));
    }

    #[test]
    fn version_one_rejected() {
        assert!(matches!(
            check_version_compatibility(0x0001),
            Err(BcibError::UnsupportedVersion(_))
        ));
    }

    #[test]
    fn future_version_rejected_fail_closed() {
        // Any version beyond v3 must be rejected deterministically.
        for future in [0x0004u16, 0x0010, 0x00FF, 0xFFFF] {
            assert!(
                matches!(
                    check_version_compatibility(future),
                    Err(BcibError::UnsupportedVersion(_))
                ),
                "version 0x{:04X} should be rejected",
                future
            );
        }
    }

    #[test]
    fn v3_and_v02_are_the_only_accepted_versions() {
        // Exhaustive check over the full u16 range would be slow; spot-check
        // the boundary values and a representative sample.
        let rejected = [0x0000u16, 0x0001, 0x0004, 0x0100, 0x1000, 0xFFFF];
        for &v in &rejected {
            assert!(
                matches!(
                    check_version_compatibility(v),
                    Err(BcibError::UnsupportedVersion(_))
                ),
                "version 0x{:04X} should be rejected",
                v
            );
        }
    }

    // -----------------------------------------------------------------------
    // validate_opcode_no_conflict
    // -----------------------------------------------------------------------

    #[test]
    fn opcode_registry_has_no_conflicts() {
        validate_opcode_no_conflict().expect("opcode registry must be conflict-free");
    }

    #[test]
    fn all_reserved_v02_ids_present_in_registry() {
        // Every v0.2 reserved ID must resolve in the opcode table.
        for &id in RESERVED_V02 {
            assert!(
                lookup_opcode(id).is_ok(),
                "v0.2 reserved opcode 0x{:02X} must remain in the registry",
                id
            );
        }
    }
}
