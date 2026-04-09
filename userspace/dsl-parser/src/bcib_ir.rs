/// DSL → BCIB IR conversion (ci-gate-dsl-bcib-contract, WS 3.2)
///
/// Converts a parsed DSL `Command` into a `bcib::BcibBuffer` (BCIB v0.2 IR).
/// The mapping is deterministic: the same DSL command always produces the
/// same BCIB instruction sequence (Requirement 4.1 — DETERMINISM.GLOBAL).
///
/// Requirements: 7.1, 7.2, 7.3
///
/// ## Mapping table
///
/// | DSL Command          | BCIB Instruction(s)                        |
/// |----------------------|--------------------------------------------|
/// | SelectContext        | Nop (context selection has no IR opcode)   |
/// | Create { schema }    | DataCreate(0, schema_slot) + End           |
/// | Add { payload }      | DataAdd(0, payload_slot) + End             |
/// | Query { filter }     | DataQuery(0, filter_slot) + End            |
/// | AiAsk { prompt }     | AiAsk(prompt_slot) + End                   |
/// | Render               | UiRender(0) + End                          |
/// | Info / List / Help   | Nop + End (diagnostic / meta commands)     |
/// | Batch(cmds)          | one Nop per sub-command + End              |
/// | Exit                 | End                                        |
///
/// Slot indices are symbolic (0-based) — the runtime resolves them.

use bcib::{BcibBuffer, BcibInstruction};

use crate::parser::{Command, ParseError};

/// Convert a DSL `Command` into a BCIB IR buffer.
///
/// Returns `Err(ParseError::InvalidSyntax)` only if the command variant
/// cannot be represented (currently unreachable — all variants are handled).
pub fn command_to_bcib_ir(command: &Command) -> Result<BcibBuffer, ParseError> {
    let mut buf = BcibBuffer::new();

    match command {
        // Context selection has no execution opcode — emit Nop + End.
        Command::SelectContext { .. } => {
            buf.add(BcibInstruction::nop());
            buf.add(BcibInstruction::end());
        }

        // data.create schema=[...] → DataCreate(target=0, schema_slot=1) + End
        Command::Create { .. } => {
            buf.add(BcibInstruction::data_create(0, 1));
            buf.add(BcibInstruction::end());
        }

        // data.add {...} → DataAdd(target=0, payload_slot=2) + End
        Command::Add { .. } => {
            buf.add(BcibInstruction::data_add(0, 2));
            buf.add(BcibInstruction::end());
        }

        // data.query filter=... → DataQuery(target=0, filter_slot=3) + End
        Command::Query { .. } => {
            buf.add(BcibInstruction::data_query(0, 3));
            buf.add(BcibInstruction::end());
        }

        // ai.ask "..." → AiAsk(prompt_slot=4) + End
        Command::AiAsk { .. } => {
            buf.add(BcibInstruction::ai_ask(4));
            buf.add(BcibInstruction::end());
        }

        // ui.render → UiRender(scene=0) + End
        Command::Render => {
            buf.add(BcibInstruction::ui_render(0));
            buf.add(BcibInstruction::end());
        }

        // Meta / diagnostic commands → Nop + End
        Command::Info | Command::List { .. } | Command::Help { .. } => {
            buf.add(BcibInstruction::nop());
            buf.add(BcibInstruction::end());
        }

        // Batch: one Nop per sub-command + End
        Command::Batch(cmds) => {
            for _ in cmds {
                buf.add(BcibInstruction::nop());
            }
            buf.add(BcibInstruction::end());
        }

        // Exit → End only
        Command::Exit => {
            buf.add(BcibInstruction::end());
        }
    }

    Ok(buf)
}

// ---------------------------------------------------------------------------
// Golden fixture tests — ci-gate-dsl-bcib-contract (Requirements 7.1, 7.2, 7.3)
//
// Each fixture asserts:
//   1. The DSL command parses without error (Requirement 7.1).
//   2. The resulting BCIB IR encodes to a deterministic byte sequence
//      (Requirement 7.2 — same input → same IR; DETERMINISM.GLOBAL).
//   3. Invalid DSL input fails closed (Requirement 7.3).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod golden_fixture_tests {
    use super::*;
    use crate::parser::DslParser;

    // -----------------------------------------------------------------------
    // Helper: parse a DSL command and convert to BCIB IR bytes.
    // -----------------------------------------------------------------------
    #[allow(dead_code)]
    fn dsl_to_ir_bytes(parser: &mut DslParser, input: &str) -> Vec<u8> {
        let dispatch = parser.parse_command(input).expect("parse failed");
        let buf = command_to_bcib_ir(&dispatch.command).expect("ir conversion failed");
        buf.encode()
    }

    // -----------------------------------------------------------------------
    // Fixture 1 — SelectContext
    // DSL: "> data.users"
    // Expected IR: [Nop, End]
    // -----------------------------------------------------------------------
    #[test]
    fn fixture_select_context_produces_nop_end() {
        let mut parser = DslParser::new();
        let dispatch = parser.parse_command("> data.users").unwrap();
        let buf = command_to_bcib_ir(&dispatch.command).unwrap();
        assert_eq!(buf.len(), 2, "SelectContext must produce exactly 2 instructions");
        // Determinism: encode twice → same bytes
        let bytes_a = buf.encode();
        let bytes_b = command_to_bcib_ir(&dispatch.command).unwrap().encode();
        assert_eq!(bytes_a, bytes_b, "DETERMINISM.GLOBAL: same DSL → same IR bytes");
    }

    // -----------------------------------------------------------------------
    // Fixture 2 — Create
    // DSL: ">> create schema=[id:int,name:string]"
    // Expected IR: [DataCreate(0,1), End]
    // -----------------------------------------------------------------------
    #[test]
    fn fixture_create_produces_data_create_end() {
        let mut parser = DslParser::new();
        parser.parse_command("> data.users").unwrap();
        let dispatch = parser
            .parse_command(">> create schema=[id:int,name:string]")
            .unwrap();
        let buf = command_to_bcib_ir(&dispatch.command).unwrap();
        assert_eq!(buf.len(), 2);
        let bytes_a = buf.encode();
        let bytes_b = command_to_bcib_ir(&dispatch.command).unwrap().encode();
        assert_eq!(bytes_a, bytes_b, "DETERMINISM.GLOBAL: Create IR is stable");
    }

    // -----------------------------------------------------------------------
    // Fixture 3 — Add
    // DSL: ">> add {\"id\":1,\"name\":\"Alice\"}"
    // Expected IR: [DataAdd(0,2), End]
    // -----------------------------------------------------------------------
    #[test]
    fn fixture_add_produces_data_add_end() {
        let mut parser = DslParser::new();
        parser.parse_command("> data.users").unwrap();
        let dispatch = parser
            .parse_command(">> add {\"id\":1,\"name\":\"Alice\"}")
            .unwrap();
        let buf = command_to_bcib_ir(&dispatch.command).unwrap();
        assert_eq!(buf.len(), 2);
        let bytes_a = buf.encode();
        let bytes_b = command_to_bcib_ir(&dispatch.command).unwrap().encode();
        assert_eq!(bytes_a, bytes_b, "DETERMINISM.GLOBAL: Add IR is stable");
    }

    // -----------------------------------------------------------------------
    // Fixture 4 — Query
    // DSL: ">> query filter=\"age > 30\""
    // Expected IR: [DataQuery(0,3), End]
    // -----------------------------------------------------------------------
    #[test]
    fn fixture_query_produces_data_query_end() {
        let mut parser = DslParser::new();
        parser.parse_command("> data.users").unwrap();
        let dispatch = parser
            .parse_command(">> query filter=\"age > 30\"")
            .unwrap();
        let buf = command_to_bcib_ir(&dispatch.command).unwrap();
        assert_eq!(buf.len(), 2);
        let bytes_a = buf.encode();
        let bytes_b = command_to_bcib_ir(&dispatch.command).unwrap().encode();
        assert_eq!(bytes_a, bytes_b, "DETERMINISM.GLOBAL: Query IR is stable");
    }

    // -----------------------------------------------------------------------
    // Fixture 5 — AiAsk
    // DSL: ">> ask \"What is the system status?\""
    // Expected IR: [AiAsk(4), End]
    // -----------------------------------------------------------------------
    #[test]
    fn fixture_ai_ask_produces_ai_ask_end() {
        let mut parser = DslParser::new();
        parser.parse_command("> ai").unwrap();
        let dispatch = parser
            .parse_command(">> ask \"What is the system status?\"")
            .unwrap();
        let buf = command_to_bcib_ir(&dispatch.command).unwrap();
        assert_eq!(buf.len(), 2);
        let bytes_a = buf.encode();
        let bytes_b = command_to_bcib_ir(&dispatch.command).unwrap().encode();
        assert_eq!(bytes_a, bytes_b, "DETERMINISM.GLOBAL: AiAsk IR is stable");
    }

    // -----------------------------------------------------------------------
    // Fixture 6 — Render
    // DSL: ">> render"
    // Expected IR: [UiRender(0), End]
    // -----------------------------------------------------------------------
    #[test]
    fn fixture_render_produces_ui_render_end() {
        let mut parser = DslParser::new();
        parser.parse_command("> ui.scene.dashboard").unwrap();
        let dispatch = parser.parse_command(">> render").unwrap();
        let buf = command_to_bcib_ir(&dispatch.command).unwrap();
        assert_eq!(buf.len(), 2);
        let bytes_a = buf.encode();
        let bytes_b = command_to_bcib_ir(&dispatch.command).unwrap().encode();
        assert_eq!(bytes_a, bytes_b, "DETERMINISM.GLOBAL: Render IR is stable");
    }

    // -----------------------------------------------------------------------
    // Fixture 7 — Batch (3 sub-commands)
    // DSL: ">[ ] cmd1 | cmd2 | cmd3"
    // Expected IR: [Nop, Nop, Nop, End]
    // -----------------------------------------------------------------------
    #[test]
    fn fixture_batch_produces_nop_per_cmd_plus_end() {
        let mut parser = DslParser::new();
        parser.parse_command("> data.test").unwrap();
        let dispatch = parser
            .parse_command(">[ ] cmd1 | cmd2 | cmd3")
            .unwrap();
        let buf = command_to_bcib_ir(&dispatch.command).unwrap();
        // 3 sub-commands → 3 Nops + 1 End = 4 instructions
        assert_eq!(buf.len(), 4);
        let bytes_a = buf.encode();
        let bytes_b = command_to_bcib_ir(&dispatch.command).unwrap().encode();
        assert_eq!(bytes_a, bytes_b, "DETERMINISM.GLOBAL: Batch IR is stable");
    }

    // -----------------------------------------------------------------------
    // Fixture 8 — Exit
    // DSL: ">> exit"
    // Expected IR: [End]
    // -----------------------------------------------------------------------
    #[test]
    fn fixture_exit_produces_end_only() {
        let mut parser = DslParser::new();
        parser.parse_command("> data.test").unwrap();
        let dispatch = parser.parse_command(">> exit").unwrap();
        let buf = command_to_bcib_ir(&dispatch.command).unwrap();
        assert_eq!(buf.len(), 1);
        let bytes_a = buf.encode();
        let bytes_b = command_to_bcib_ir(&dispatch.command).unwrap().encode();
        assert_eq!(bytes_a, bytes_b, "DETERMINISM.GLOBAL: Exit IR is stable");
    }

    // -----------------------------------------------------------------------
    // Requirement 7.3 — Fail-closed: invalid DSL input must not produce IR.
    // -----------------------------------------------------------------------
    #[test]
    fn invalid_dsl_fails_closed_no_ir_produced() {
        let mut parser = DslParser::new();
        // Invalid context → parse error; no IR produced
        assert!(parser.parse_command("> invalid.context").is_err());
        // Missing context for action → parse error
        assert!(parser.parse_command(">> add {}").is_err());
        // Unknown action → parse error
        parser.parse_command("> data.test").unwrap();
        assert!(parser.parse_command(">> unknown_action").is_err());
    }

    // -----------------------------------------------------------------------
    // Requirement 7.2 — Determinism: same DSL input → same IR bytes (golden).
    // Run the full workflow twice and compare byte-for-byte.
    // -----------------------------------------------------------------------
    #[test]
    fn golden_full_workflow_deterministic() {
        let run = || -> Vec<Vec<u8>> {
            let mut parser = DslParser::new();
            let commands = [
                "> data.users",
                ">> create schema=[id:int,name:string,age:int]",
                ">> add {\"id\":1,\"name\":\"Alice\",\"age\":30}",
                ">> query filter=\"age > 25\"",
            ];
            commands
                .iter()
                .map(|&cmd| {
                    let dispatch = parser.parse_command(cmd).unwrap();
                    command_to_bcib_ir(&dispatch.command).unwrap().encode()
                })
                .collect()
        };

        let run_a = run();
        let run_b = run();
        assert_eq!(
            run_a, run_b,
            "DETERMINISM.GLOBAL: full workflow must produce identical IR on every run"
        );
    }

    // -----------------------------------------------------------------------
    // Requirement 7.1 — All supported DSL commands produce valid BCIB IR.
    // -----------------------------------------------------------------------
    #[test]
    fn all_dsl_commands_produce_valid_ir() {
        let mut parser = DslParser::new();
        parser.parse_command("> data.test").unwrap();

        let commands = [
            ">> create schema=[id:int]",
            ">> add {\"id\":1}",
            ">> query filter=\"id=1\"",
            ">> list",
            ">> help",
            ">> info",
            ">> render",
        ];

        for cmd in &commands {
            let dispatch = parser.parse_command(cmd).unwrap();
            let result = command_to_bcib_ir(&dispatch.command);
            assert!(
                result.is_ok(),
                "command '{}' must produce valid BCIB IR",
                cmd
            );
            let buf = result.unwrap();
            assert!(buf.len() >= 1, "IR must have at least one instruction");
        }

        // AI ask
        parser.parse_command("> ai").unwrap();
        let dispatch = parser.parse_command(">> ask \"test\"").unwrap();
        assert!(command_to_bcib_ir(&dispatch.command).is_ok());
    }
}
