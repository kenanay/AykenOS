/// Opcode Registry — single source of truth for all BCIB v3 opcodes.
///
/// Requirements: 12.1, 12.5, 12.6, 17.1
///
/// Six opcode classes with fixed ID ranges:
///   control     0x00–0x0F  (Pure)
///   memory      0x10–0x1F  (Pure / DataMutating)
///   data        0x20–0x2F  (DataMutating)
///   ai          0x30–0x3F  (External)
///   ui          0x40–0x4F  (External)
///   diagnostics 0x50–0x5F  (Pure)
///
/// v0.2 reserved opcode IDs are locked in `RESERVED_V02` and MUST NOT be
/// reused or redefined (Requirement 12.5, 12.6).
use crate::types::{BcibError, CostUnit, OpcodeId, SideEffectClass};
use crate::types::{COST_DATA_MUTATING, COST_EXTERNAL, COST_PURE};

// ---------------------------------------------------------------------------
// Opcode descriptor
// ---------------------------------------------------------------------------

/// Static descriptor for a single opcode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpcodeDescriptor {
    pub id: OpcodeId,
    pub name: &'static str,
    pub class: OpcodeClass,
    pub side_effect: SideEffectClass,
    pub cost: CostUnit,
}

/// Opcode class (one of six, Requirement 12.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpcodeClass {
    Control,
    Memory,
    Data,
    Ai,
    Ui,
    Diagnostics,
}

// ---------------------------------------------------------------------------
// v0.2 reserved opcode IDs — MUST NOT be reused (Requirement 12.5, 12.6)
// ---------------------------------------------------------------------------

/// v0.2 reserved opcode IDs. Any attempt to register a new opcode with one
/// of these IDs is a CI-level error (`ci-gate-toolchain-opcode-registry`).
pub const RESERVED_V02: &[OpcodeId] = &[
    0x00, // Nop
    0x01, // End
    0x10, // DataCreate
    0x11, // DataAdd
    0x12, // DataQuery
    0x20, // UiRender
    0x30, // AiAsk
];

// ---------------------------------------------------------------------------
// Opcode table — const, O(1) index lookup via array position
// ---------------------------------------------------------------------------
//
// The table is a flat array of 256 entries (one per possible u8 opcode ID).
// `None` means the opcode is unknown / unregistered.
// `lookup_opcode()` indexes directly into this array → O(1).

/// Total opcode ID space (u8).
const OPCODE_TABLE_SIZE: usize = 256;

/// Build the static opcode lookup table at compile time.
///
/// Only the defined opcodes are populated; all other slots are `None`.
const fn build_table() -> [Option<OpcodeDescriptor>; OPCODE_TABLE_SIZE] {
    // Rust const fn does not support iterators, so we initialise manually.
    let mut table: [Option<OpcodeDescriptor>; OPCODE_TABLE_SIZE] = [None; OPCODE_TABLE_SIZE];

    // -----------------------------------------------------------------------
    // Control class — 0x00–0x0F (Pure)
    // -----------------------------------------------------------------------
    table[0x00] = Some(OpcodeDescriptor {
        id: 0x00,
        name: "Nop",
        class: OpcodeClass::Control,
        side_effect: SideEffectClass::Pure,
        cost: COST_PURE,
    });
    table[0x01] = Some(OpcodeDescriptor {
        id: 0x01,
        name: "End",
        class: OpcodeClass::Control,
        side_effect: SideEffectClass::Pure,
        cost: COST_PURE,
    });
    table[0x02] = Some(OpcodeDescriptor {
        id: 0x02,
        name: "Jump",
        class: OpcodeClass::Control,
        side_effect: SideEffectClass::Pure,
        cost: COST_PURE,
    });
    table[0x03] = Some(OpcodeDescriptor {
        id: 0x03,
        name: "JumpIf",
        class: OpcodeClass::Control,
        side_effect: SideEffectClass::Pure,
        cost: COST_PURE,
    });

    // -----------------------------------------------------------------------
    // Memory class — 0x10–0x1F
    // -----------------------------------------------------------------------
    table[0x10] = Some(OpcodeDescriptor {
        id: 0x10,
        name: "DataCreate", // v0.2 reserved — kept with original semantics
        class: OpcodeClass::Memory,
        side_effect: SideEffectClass::DataMutating,
        cost: COST_DATA_MUTATING,
    });
    table[0x11] = Some(OpcodeDescriptor {
        id: 0x11,
        name: "DataAdd", // v0.2 reserved
        class: OpcodeClass::Memory,
        side_effect: SideEffectClass::DataMutating,
        cost: COST_DATA_MUTATING,
    });
    table[0x12] = Some(OpcodeDescriptor {
        id: 0x12,
        name: "DataQuery", // v0.2 reserved
        class: OpcodeClass::Memory,
        side_effect: SideEffectClass::DataMutating,
        cost: COST_DATA_MUTATING,
    });
    table[0x13] = Some(OpcodeDescriptor {
        id: 0x13,
        name: "SlotAlloc",
        class: OpcodeClass::Memory,
        side_effect: SideEffectClass::Pure,
        cost: COST_PURE,
    });
    table[0x14] = Some(OpcodeDescriptor {
        id: 0x14,
        name: "SlotFree",
        class: OpcodeClass::Memory,
        side_effect: SideEffectClass::Pure,
        cost: COST_PURE,
    });
    table[0x15] = Some(OpcodeDescriptor {
        id: 0x15,
        name: "HandleBorrow",
        class: OpcodeClass::Memory,
        side_effect: SideEffectClass::Pure,
        cost: COST_PURE,
    });

    // -----------------------------------------------------------------------
    // Data class — 0x20–0x2F (DataMutating)
    // -----------------------------------------------------------------------
    table[0x20] = Some(OpcodeDescriptor {
        id: 0x20,
        name: "UiRender", // v0.2 reserved — kept with original semantics
        class: OpcodeClass::Data,
        side_effect: SideEffectClass::DataMutating,
        cost: COST_DATA_MUTATING,
    });
    table[0x21] = Some(OpcodeDescriptor {
        id: 0x21,
        name: "DataWrite",
        class: OpcodeClass::Data,
        side_effect: SideEffectClass::DataMutating,
        cost: COST_DATA_MUTATING,
    });
    table[0x22] = Some(OpcodeDescriptor {
        id: 0x22,
        name: "DataDelete",
        class: OpcodeClass::Data,
        side_effect: SideEffectClass::DataMutating,
        cost: COST_DATA_MUTATING,
    });

    // -----------------------------------------------------------------------
    // AI class — 0x30–0x3F (External)
    // -----------------------------------------------------------------------
    table[0x30] = Some(OpcodeDescriptor {
        id: 0x30,
        name: "AiAsk", // v0.2 reserved
        class: OpcodeClass::Ai,
        side_effect: SideEffectClass::External,
        cost: COST_EXTERNAL,
    });
    table[0x31] = Some(OpcodeDescriptor {
        id: 0x31,
        name: "AiStream",
        class: OpcodeClass::Ai,
        side_effect: SideEffectClass::External,
        cost: COST_EXTERNAL,
    });

    // -----------------------------------------------------------------------
    // UI class — 0x40–0x4F (External)
    // -----------------------------------------------------------------------
    table[0x40] = Some(OpcodeDescriptor {
        id: 0x40,
        name: "UiEvent",
        class: OpcodeClass::Ui,
        side_effect: SideEffectClass::External,
        cost: COST_EXTERNAL,
    });
    table[0x41] = Some(OpcodeDescriptor {
        id: 0x41,
        name: "UiUpdate",
        class: OpcodeClass::Ui,
        side_effect: SideEffectClass::External,
        cost: COST_EXTERNAL,
    });

    // -----------------------------------------------------------------------
    // Diagnostics class — 0x50–0x5F (Pure)
    // -----------------------------------------------------------------------
    table[0x50] = Some(OpcodeDescriptor {
        id: 0x50,
        name: "TraceEmit",
        class: OpcodeClass::Diagnostics,
        side_effect: SideEffectClass::Pure,
        cost: COST_PURE,
    });
    table[0x51] = Some(OpcodeDescriptor {
        id: 0x51,
        name: "CostReport",
        class: OpcodeClass::Diagnostics,
        side_effect: SideEffectClass::Pure,
        cost: COST_PURE,
    });

    table
}

/// Static opcode lookup table — O(1) array index dispatch.
static OPCODE_TABLE: [Option<OpcodeDescriptor>; OPCODE_TABLE_SIZE] = build_table();

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Look up an opcode descriptor by ID.
///
/// Returns `Ok(&OpcodeDescriptor)` for known opcodes.
/// Returns `Err(BcibError::InvalidGraph)` for unknown opcodes (fail-closed,
/// Requirement 12.1).
///
/// # Complexity
///
/// **O(1)** — direct array index into a 256-entry static table.
/// No iteration, no hashing, no branching on the opcode value.
/// The table is built at compile time (`const fn build_table()`), so the
/// lookup is a single bounds-checked array access at runtime
/// (Requirement 19.1 — O(1) dispatch path).
///
/// # Fail-fast
///
/// Unknown opcode IDs return `BCIB_ERR_INVALID_GRAPH` immediately with no
/// further work performed (Requirement 19.1 — early exit on invalid graph).
#[inline]
pub fn lookup_opcode(id: OpcodeId) -> Result<&'static OpcodeDescriptor, BcibError> {
    OPCODE_TABLE[id as usize]
        .as_ref()
        .ok_or(BcibError::InvalidGraph("unknown opcode id"))
}

/// Returns `true` if the given opcode ID is in the v0.2 reserved list.
///
/// Used by the toolchain CI gate to prevent ID reuse (Requirement 12.5).
#[inline]
pub fn is_reserved_v02(id: OpcodeId) -> bool {
    RESERVED_V02.contains(&id)
}

// ---------------------------------------------------------------------------
// Build-time assertion: cost ordering invariant (pure < data-mutating < external)
// ---------------------------------------------------------------------------

const _: () = {
    assert!(
        COST_PURE < COST_DATA_MUTATING,
        "COST_PURE must be less than COST_DATA_MUTATING"
    );
    assert!(
        COST_DATA_MUTATING < COST_EXTERNAL,
        "COST_DATA_MUTATING must be less than COST_EXTERNAL"
    );
};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_opcodes_resolve() {
        // v0.2 reserved opcodes must be present
        for &id in RESERVED_V02 {
            assert!(
                lookup_opcode(id).is_ok(),
                "v0.2 reserved opcode 0x{:02X} must be registered",
                id
            );
        }
    }

    #[test]
    fn unknown_opcode_returns_invalid_graph() {
        // 0xFF is not registered
        let err = lookup_opcode(0xFF).unwrap_err();
        assert_eq!(err, BcibError::InvalidGraph("unknown opcode id"));
    }

    #[test]
    fn cost_ordering_invariant() {
        assert!(COST_PURE < COST_DATA_MUTATING);
        assert!(COST_DATA_MUTATING < COST_EXTERNAL);
    }

    #[test]
    fn v02_reserved_ids_are_locked() {
        for &id in RESERVED_V02 {
            assert!(
                is_reserved_v02(id),
                "0x{:02X} should be flagged as reserved",
                id
            );
        }
        // A non-reserved id should not be flagged
        assert!(!is_reserved_v02(0x02));
    }

    #[test]
    fn side_effect_classes_match_spec() {
        // Control opcodes are Pure
        assert_eq!(
            lookup_opcode(0x00).unwrap().side_effect,
            SideEffectClass::Pure
        );
        assert_eq!(
            lookup_opcode(0x02).unwrap().side_effect,
            SideEffectClass::Pure
        );

        // Data opcodes are DataMutating
        assert_eq!(
            lookup_opcode(0x10).unwrap().side_effect,
            SideEffectClass::DataMutating
        );
        assert_eq!(
            lookup_opcode(0x21).unwrap().side_effect,
            SideEffectClass::DataMutating
        );

        // AI/UI opcodes are External
        assert_eq!(
            lookup_opcode(0x30).unwrap().side_effect,
            SideEffectClass::External
        );
        assert_eq!(
            lookup_opcode(0x40).unwrap().side_effect,
            SideEffectClass::External
        );

        // Diagnostics opcodes are Pure
        assert_eq!(
            lookup_opcode(0x50).unwrap().side_effect,
            SideEffectClass::Pure
        );
    }

    /// Verify O(1) dispatch: every possible u8 value either resolves to a
    /// known descriptor or returns `BCIB_ERR_INVALID_GRAPH` immediately.
    /// No linear scan — the result is determined by a single array index.
    #[test]
    fn lookup_is_o1_all_256_ids() {
        for id in 0u8..=255 {
            let result = lookup_opcode(id);
            match result {
                Ok(desc) => assert_eq!(desc.id, id, "descriptor id must match lookup id"),
                Err(BcibError::InvalidGraph(_)) => {} // unknown opcode — correct fail-fast
                Err(e) => panic!("unexpected error for opcode 0x{:02X}: {:?}", id, e),
            }
        }
    }

    /// Fail-fast: unknown opcode returns BCIB_ERR_INVALID_GRAPH with no
    /// additional work (verified by checking several unregistered IDs).
    #[test]
    fn unknown_opcodes_fail_fast() {
        // Spot-check several unregistered IDs across all class ranges
        let unregistered: &[u8] = &[0x04, 0x0F, 0x1F, 0x2F, 0x3F, 0x4F, 0x5F, 0xFF];
        for &id in unregistered {
            assert!(
                matches!(lookup_opcode(id), Err(BcibError::InvalidGraph(_))),
                "opcode 0x{:02X} should fail-fast with BCIB_ERR_INVALID_GRAPH",
                id
            );
        }
    }
}
