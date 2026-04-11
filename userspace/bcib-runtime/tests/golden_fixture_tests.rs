use bcib_runtime::types::CapabilityTokenId;
/// Golden Fixture Integration Tests — BCIB v0.2 Corpus Validation
///
/// Requirements: 1.5, 12.3, 12.4
///
/// Each test loads a binary fixture from `tests/fixtures/`, runs the
/// parse → verify → plan cycle, and compares the result against the
/// expected outcome defined in `fixtures.json`.
///
/// Fixture mismatch → test FAIL → CI FAIL (Requirement 12.3).
///
/// The fixtures represent the v0.2 compatibility corpus (Requirement 12.4):
///   - nop_end.bcib            — minimal v0.2 program (Pure opcodes only)
///   - data_create_query.bcib  — DataCreate + DataQuery (DataMutating)
///   - data_add.bcib           — DataCreate + DataAdd (DataMutating)
///   - ui_render.bcib          — UiRender (DataMutating)
///   - ai_ask.bcib             — AiAsk (External)
///   - invalid_magic.bcib      — negative: bad magic → BCIB_ERR_INVALID_GRAPH
///   - unsupported_version.bcib — negative: version 0x0004 → BCIB_ERR_UNSUPPORTED_VERSION
use bcib_runtime::{
    BcibError, BcibVerifierPlanner, CapabilitySet, ResourceLimits, BCIB_VERSION_V02,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Load a fixture file from `tests/fixtures/`.
///
/// Panics with a clear message if the file is missing — this is a CI FAIL
/// condition (Requirement 12.3: fixture mismatch → CI FAIL).
fn load_fixture(name: &str) -> Vec<u8> {
    let path = format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "FIXTURE MISMATCH — CI FAIL: cannot load fixture '{}': {}\n\
             Run `cargo build` to regenerate fixtures from build.rs.",
            name, e
        )
    })
}

/// Build a `CapabilitySet` with a single token for testing.
///
/// The `NoopCapabilityManager` stub (Group 1, Task 1.4) accepts any token,
/// so we just need at least one token in the set for DataMutating/External
/// instructions to pass capability validation.
fn caps_with_token() -> CapabilitySet {
    CapabilitySet {
        token_ids: vec![1u64 as CapabilityTokenId],
    }
}

/// Empty capability set — used for Pure-only programs.
fn empty_caps() -> CapabilitySet {
    CapabilitySet::default()
}

/// Default resource limits — permissive enough for all fixture programs.
fn default_limits() -> ResourceLimits {
    ResourceLimits::default()
}

// ---------------------------------------------------------------------------
// Fixture 1: nop_end.bcib
// Minimal v0.2 program: Nop (0x00) + End (0x01)
// Expected: parse OK, plan OK
// Requirements: 1.5, 12.3, 12.4
// ---------------------------------------------------------------------------

#[test]
fn fixture_nop_end_parse_and_plan_ok() {
    let data = load_fixture("nop_end.bcib");
    let planner = BcibVerifierPlanner::new();

    // Parse: header must be valid v0.2
    let header = bcib_runtime::parse_header(&data)
        .expect("FIXTURE MISMATCH — nop_end.bcib: parse_header must succeed");
    assert_eq!(
        header.version, BCIB_VERSION_V02,
        "FIXTURE MISMATCH — nop_end.bcib: version must be 0x0002 (v0.2)"
    );
    assert_eq!(
        header.magic, *b"BCIB",
        "FIXTURE MISMATCH — nop_end.bcib: magic must be b\"BCIB\""
    );

    // Verify + Plan: Nop and End are Pure — no capability required
    let plan = planner
        .verify_and_plan(&data, &empty_caps(), &default_limits())
        .expect("FIXTURE MISMATCH — nop_end.bcib: verify_and_plan must succeed");

    // Validate plan content
    let instrs = plan.instructions();
    assert_eq!(
        instrs.len(),
        2,
        "FIXTURE MISMATCH — nop_end.bcib: expected 2 instructions (Nop + End)"
    );
    assert_eq!(
        instrs[0].opcode, 0x00,
        "FIXTURE MISMATCH — nop_end.bcib: instruction[0] must be Nop (0x00)"
    );
    assert_eq!(
        instrs[1].opcode, 0x01,
        "FIXTURE MISMATCH — nop_end.bcib: instruction[1] must be End (0x01)"
    );

    // Canonical hash must be stable (DETERMINISM.GLOBAL)
    let h1 = plan.canonical_hash();
    let h2 = plan.canonical_hash();
    assert_eq!(
        h1, h2,
        "FIXTURE MISMATCH — nop_end.bcib: canonical_hash must be deterministic"
    );
}

// ---------------------------------------------------------------------------
// Fixture 2: data_create_query.bcib
// DataCreate (0x10) + DataQuery (0x12) + End (0x01)
// Expected: parse OK, plan OK (with capability token)
// Requirements: 1.5, 12.3, 12.4
// ---------------------------------------------------------------------------

#[test]
fn fixture_data_create_query_parse_and_plan_ok() {
    let data = load_fixture("data_create_query.bcib");
    let planner = BcibVerifierPlanner::new();

    // Parse
    let header = bcib_runtime::parse_header(&data)
        .expect("FIXTURE MISMATCH — data_create_query.bcib: parse_header must succeed");
    assert_eq!(
        header.version, BCIB_VERSION_V02,
        "FIXTURE MISMATCH — data_create_query.bcib: version must be 0x0002 (v0.2)"
    );

    // Verify + Plan with capability token (DataMutating instructions require it)
    let plan = planner
        .verify_and_plan(&data, &caps_with_token(), &default_limits())
        .expect("FIXTURE MISMATCH — data_create_query.bcib: verify_and_plan must succeed");

    let instrs = plan.instructions();
    assert_eq!(
        instrs.len(),
        3,
        "FIXTURE MISMATCH — data_create_query.bcib: expected 3 instructions"
    );
    assert_eq!(
        instrs[0].opcode, 0x10,
        "FIXTURE MISMATCH — data_create_query.bcib: instruction[0] must be DataCreate (0x10)"
    );
    assert_eq!(
        instrs[1].opcode, 0x12,
        "FIXTURE MISMATCH — data_create_query.bcib: instruction[1] must be DataQuery (0x12)"
    );
    assert_eq!(
        instrs[2].opcode, 0x01,
        "FIXTURE MISMATCH — data_create_query.bcib: instruction[2] must be End (0x01)"
    );

    // Without capability → must fail (fail-closed, Requirement 1.5)
    let err = planner
        .verify_and_plan(&data, &empty_caps(), &default_limits())
        .expect_err(
            "FIXTURE MISMATCH — data_create_query.bcib: verify_and_plan without capability must fail",
        );
    assert!(
        matches!(err, BcibError::CapabilityDenied(_)),
        "FIXTURE MISMATCH — data_create_query.bcib: expected BCIB_ERR_CAPABILITY_DENIED, got {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// Fixture 3: data_add.bcib
// DataCreate (0x10) + DataAdd (0x11) + End (0x01)
// Expected: parse OK, plan OK (with capability token)
// Requirements: 1.5, 12.3, 12.4
// ---------------------------------------------------------------------------

#[test]
fn fixture_data_add_parse_and_plan_ok() {
    let data = load_fixture("data_add.bcib");
    let planner = BcibVerifierPlanner::new();

    let header = bcib_runtime::parse_header(&data)
        .expect("FIXTURE MISMATCH — data_add.bcib: parse_header must succeed");
    assert_eq!(
        header.version, BCIB_VERSION_V02,
        "FIXTURE MISMATCH — data_add.bcib: version must be 0x0002 (v0.2)"
    );

    let plan = planner
        .verify_and_plan(&data, &caps_with_token(), &default_limits())
        .expect("FIXTURE MISMATCH — data_add.bcib: verify_and_plan must succeed");

    let instrs = plan.instructions();
    assert_eq!(
        instrs.len(),
        3,
        "FIXTURE MISMATCH — data_add.bcib: expected 3 instructions"
    );
    assert_eq!(
        instrs[0].opcode, 0x10,
        "FIXTURE MISMATCH — data_add.bcib: instruction[0] must be DataCreate (0x10)"
    );
    assert_eq!(
        instrs[1].opcode, 0x11,
        "FIXTURE MISMATCH — data_add.bcib: instruction[1] must be DataAdd (0x11)"
    );
    assert_eq!(
        instrs[2].opcode, 0x01,
        "FIXTURE MISMATCH — data_add.bcib: instruction[2] must be End (0x01)"
    );

    // Without capability → must fail
    let err = planner
        .verify_and_plan(&data, &empty_caps(), &default_limits())
        .expect_err("FIXTURE MISMATCH — data_add.bcib: must fail without capability");
    assert!(
        matches!(err, BcibError::CapabilityDenied(_)),
        "FIXTURE MISMATCH — data_add.bcib: expected BCIB_ERR_CAPABILITY_DENIED, got {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// Fixture 4: ui_render.bcib
// UiRender (0x20) + End (0x01)
// Expected: parse OK, plan OK (with capability token)
// Requirements: 1.5, 12.3, 12.4
// ---------------------------------------------------------------------------

#[test]
fn fixture_ui_render_parse_and_plan_ok() {
    let data = load_fixture("ui_render.bcib");
    let planner = BcibVerifierPlanner::new();

    let header = bcib_runtime::parse_header(&data)
        .expect("FIXTURE MISMATCH — ui_render.bcib: parse_header must succeed");
    assert_eq!(
        header.version, BCIB_VERSION_V02,
        "FIXTURE MISMATCH — ui_render.bcib: version must be 0x0002 (v0.2)"
    );

    let plan = planner
        .verify_and_plan(&data, &caps_with_token(), &default_limits())
        .expect("FIXTURE MISMATCH — ui_render.bcib: verify_and_plan must succeed");

    let instrs = plan.instructions();
    assert_eq!(
        instrs.len(),
        2,
        "FIXTURE MISMATCH — ui_render.bcib: expected 2 instructions"
    );
    assert_eq!(
        instrs[0].opcode, 0x20,
        "FIXTURE MISMATCH — ui_render.bcib: instruction[0] must be UiRender (0x20)"
    );
    assert_eq!(
        instrs[1].opcode, 0x01,
        "FIXTURE MISMATCH — ui_render.bcib: instruction[1] must be End (0x01)"
    );

    // Without capability → must fail
    let err = planner
        .verify_and_plan(&data, &empty_caps(), &default_limits())
        .expect_err("FIXTURE MISMATCH — ui_render.bcib: must fail without capability");
    assert!(
        matches!(err, BcibError::CapabilityDenied(_)),
        "FIXTURE MISMATCH — ui_render.bcib: expected BCIB_ERR_CAPABILITY_DENIED, got {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// Fixture 5: ai_ask.bcib
// AiAsk (0x30) + End (0x01)
// Expected: parse OK, plan OK (with capability token)
// Requirements: 1.5, 12.3, 12.4
// ---------------------------------------------------------------------------

#[test]
fn fixture_ai_ask_parse_and_plan_ok() {
    let data = load_fixture("ai_ask.bcib");
    let planner = BcibVerifierPlanner::new();

    let header = bcib_runtime::parse_header(&data)
        .expect("FIXTURE MISMATCH — ai_ask.bcib: parse_header must succeed");
    assert_eq!(
        header.version, BCIB_VERSION_V02,
        "FIXTURE MISMATCH — ai_ask.bcib: version must be 0x0002 (v0.2)"
    );

    let plan = planner
        .verify_and_plan(&data, &caps_with_token(), &default_limits())
        .expect("FIXTURE MISMATCH — ai_ask.bcib: verify_and_plan must succeed");

    let instrs = plan.instructions();
    assert_eq!(
        instrs.len(),
        2,
        "FIXTURE MISMATCH — ai_ask.bcib: expected 2 instructions"
    );
    assert_eq!(
        instrs[0].opcode, 0x30,
        "FIXTURE MISMATCH — ai_ask.bcib: instruction[0] must be AiAsk (0x30)"
    );
    assert_eq!(
        instrs[1].opcode, 0x01,
        "FIXTURE MISMATCH — ai_ask.bcib: instruction[1] must be End (0x01)"
    );

    // Without capability → must fail (AiAsk is External)
    let err = planner
        .verify_and_plan(&data, &empty_caps(), &default_limits())
        .expect_err("FIXTURE MISMATCH — ai_ask.bcib: must fail without capability");
    assert!(
        matches!(err, BcibError::CapabilityDenied(_)),
        "FIXTURE MISMATCH — ai_ask.bcib: expected BCIB_ERR_CAPABILITY_DENIED, got {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// Fixture 6: invalid_magic.bcib
// Negative test: magic bytes are "XBIB" instead of "BCIB"
// Expected: parse FAIL → BCIB_ERR_INVALID_GRAPH
// Requirements: 12.3, 12.4
// ---------------------------------------------------------------------------

#[test]
fn fixture_invalid_magic_parse_fails() {
    let data = load_fixture("invalid_magic.bcib");
    let planner = BcibVerifierPlanner::new();

    // parse_header must fail with BCIB_ERR_INVALID_GRAPH
    let err = bcib_runtime::parse_header(&data)
        .expect_err("FIXTURE MISMATCH — invalid_magic.bcib: parse_header must fail");
    assert!(
        matches!(err, BcibError::InvalidGraph(_)),
        "FIXTURE MISMATCH — invalid_magic.bcib: expected BCIB_ERR_INVALID_GRAPH, got {:?}",
        err
    );

    // verify_and_plan must also fail (fail-closed, Requirement 4.2)
    let err2 = planner
        .verify_and_plan(&data, &empty_caps(), &default_limits())
        .expect_err("FIXTURE MISMATCH — invalid_magic.bcib: verify_and_plan must fail");
    assert!(
        matches!(err2, BcibError::InvalidGraph(_)),
        "FIXTURE MISMATCH — invalid_magic.bcib: verify_and_plan must produce BCIB_ERR_INVALID_GRAPH, got {:?}",
        err2
    );
}

// ---------------------------------------------------------------------------
// Fixture 7: unsupported_version.bcib
// Negative test: version 0x0004 (future, unsupported)
// Expected: parse FAIL → BCIB_ERR_UNSUPPORTED_VERSION
// Requirements: 12.3, 12.4
// ---------------------------------------------------------------------------

#[test]
fn fixture_unsupported_version_parse_fails() {
    let data = load_fixture("unsupported_version.bcib");
    let planner = BcibVerifierPlanner::new();

    // parse_header must fail with BCIB_ERR_UNSUPPORTED_VERSION
    let err = bcib_runtime::parse_header(&data)
        .expect_err("FIXTURE MISMATCH — unsupported_version.bcib: parse_header must fail");
    assert!(
        matches!(err, BcibError::UnsupportedVersion(_)),
        "FIXTURE MISMATCH — unsupported_version.bcib: expected BCIB_ERR_UNSUPPORTED_VERSION, got {:?}",
        err
    );

    // verify_and_plan must also fail (fail-closed)
    let err2 = planner
        .verify_and_plan(&data, &empty_caps(), &default_limits())
        .expect_err("FIXTURE MISMATCH — unsupported_version.bcib: verify_and_plan must fail");
    assert!(
        matches!(err2, BcibError::UnsupportedVersion(_)),
        "FIXTURE MISMATCH — unsupported_version.bcib: verify_and_plan must produce BCIB_ERR_UNSUPPORTED_VERSION, got {:?}",
        err2
    );
}

// ---------------------------------------------------------------------------
// Cross-fixture: canonical hash stability across all valid fixtures
// (DETERMINISM.GLOBAL — Requirement 4.1)
// ---------------------------------------------------------------------------

#[test]
fn fixture_canonical_hash_stable_across_calls() {
    let planner = BcibVerifierPlanner::new();
    let valid_fixtures = [
        ("nop_end.bcib", false),
        ("data_create_query.bcib", true),
        ("data_add.bcib", true),
        ("ui_render.bcib", true),
        ("ai_ask.bcib", true),
    ];

    for (name, needs_cap) in &valid_fixtures {
        let data = load_fixture(name);
        let caps = if *needs_cap {
            caps_with_token()
        } else {
            empty_caps()
        };

        let plan = planner
            .verify_and_plan(&data, &caps, &default_limits())
            .unwrap_or_else(|e| {
                panic!(
                    "FIXTURE MISMATCH — {}: verify_and_plan failed unexpectedly: {:?}",
                    name, e
                )
            });

        let h1 = plan.canonical_hash();
        let h2 = plan.canonical_hash();
        assert_eq!(
            h1, h2,
            "FIXTURE MISMATCH — {}: canonical_hash must be deterministic (DETERMINISM.GLOBAL)",
            name
        );
    }
}

// ---------------------------------------------------------------------------
// Cross-fixture: all valid fixtures produce distinct canonical hashes
// (each fixture represents a distinct program)
// ---------------------------------------------------------------------------

#[test]
fn fixture_canonical_hashes_are_distinct() {
    let planner = BcibVerifierPlanner::new();
    let valid_fixtures = [
        ("nop_end.bcib", false),
        ("data_create_query.bcib", true),
        ("data_add.bcib", true),
        ("ui_render.bcib", true),
        ("ai_ask.bcib", true),
    ];

    let mut hashes = Vec::new();
    for (name, needs_cap) in &valid_fixtures {
        let data = load_fixture(name);
        let caps = if *needs_cap {
            caps_with_token()
        } else {
            empty_caps()
        };
        let plan = planner
            .verify_and_plan(&data, &caps, &default_limits())
            .unwrap_or_else(|e| panic!("FIXTURE MISMATCH — {}: {:?}", name, e));
        hashes.push((name, plan.canonical_hash()));
    }

    // All hashes must be distinct
    for i in 0..hashes.len() {
        for j in (i + 1)..hashes.len() {
            assert_ne!(
                hashes[i].1, hashes[j].1,
                "FIXTURE MISMATCH — fixtures '{}' and '{}' must have distinct canonical hashes",
                hashes[i].0, hashes[j].0
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Cross-fixture: v0.2 backward-compatibility (Requirement 1.5)
// All valid fixtures use v0.2 header — they must be accepted by the v3 engine
// ---------------------------------------------------------------------------

#[test]
fn fixture_v02_backward_compatibility() {
    let planner = BcibVerifierPlanner::new();
    let v02_fixtures = [
        ("nop_end.bcib", false),
        ("data_create_query.bcib", true),
        ("data_add.bcib", true),
        ("ui_render.bcib", true),
        ("ai_ask.bcib", true),
    ];

    for (name, needs_cap) in &v02_fixtures {
        let data = load_fixture(name);

        // Verify the fixture actually has v0.2 header
        let header = bcib_runtime::parse_header(&data).unwrap_or_else(|e| {
            panic!("FIXTURE MISMATCH — {}: parse_header failed: {:?}", name, e)
        });
        assert_eq!(
            header.version, BCIB_VERSION_V02,
            "FIXTURE MISMATCH — {}: fixture must use v0.2 header (0x0002) for backward-compat test",
            name
        );

        // v3 engine must accept v0.2 programs (backward-compatible, Requirement 1.5)
        let caps = if *needs_cap {
            caps_with_token()
        } else {
            empty_caps()
        };
        planner
            .verify_and_plan(&data, &caps, &default_limits())
            .unwrap_or_else(|e| {
                panic!(
                    "FIXTURE MISMATCH — {}: v3 engine must accept v0.2 program (Requirement 1.5), got: {:?}",
                    name, e
                )
            });
    }
}
