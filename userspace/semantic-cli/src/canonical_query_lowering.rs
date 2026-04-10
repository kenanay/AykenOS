//! Canonical query -> BCIB lowering.
//!
//! This module owns the binary lowering boundary for the canonical query
//! production path. It emits a minimal BCIB v3 instruction stream using only
//! `DataQuery`, optional `TraceEmit`, and `End`.

use crate::bcib::Capability;
use crate::canonical_query::{
    CanonicalCommandKind, CanonicalPlan, CanonicalQueryBinding,
};
use crate::error::{ErrorCode, Result, SemanticCLIError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const BCIB_MAGIC: &[u8; 4] = b"BCIB";
const BCIB_VERSION_V3: u16 = 0x0003;
const HEADER_SIZE: usize = 16;
const SECTION_ENTRY_SIZE: usize = 8;
const INSTRUCTION_SECTION_ID: u16 = 0x0001;

const OPCODE_END: u8 = 0x01;
const OPCODE_DATA_CREATE: u8 = 0x10;
const OPCODE_DATA_ADD: u8 = 0x11;
const OPCODE_DATA_QUERY: u8 = 0x12;
const OPCODE_UI_RENDER: u8 = 0x20;
const OPCODE_AI_ASK: u8 = 0x30;
const OPCODE_TRACE_EMIT: u8 = 0x50;
const OPCODE_NOP: u8 = 0x00;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalQueryLoweringOptions {
    pub emit_trace: bool,
}

impl Default for CanonicalQueryLoweringOptions {
    fn default() -> Self {
        Self { emit_trace: false }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredBcibInstruction {
    pub opcode: u8,
    pub operands: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredCanonicalQuery {
    pub command_kind: CanonicalCommandKind,
    pub binding: CanonicalQueryBinding,
    pub required_capabilities: Vec<Capability>,
    pub instructions: Vec<LoweredBcibInstruction>,
    pub bytes: Vec<u8>,
    pub bcib_sha256: String,
}

impl LoweredCanonicalQuery {
    pub fn contains_forbidden_opcode(&self) -> bool {
        self.instructions.iter().any(|instruction| {
            matches!(
                instruction.opcode,
                OPCODE_NOP
                    | OPCODE_DATA_CREATE
                    | OPCODE_DATA_ADD
                    | OPCODE_UI_RENDER
                    | OPCODE_AI_ASK
            )
        })
    }
}

pub fn lower_canonical_query_to_bcib(plan: &CanonicalPlan) -> Result<LoweredCanonicalQuery> {
    lower_canonical_query_to_bcib_with_options(plan, CanonicalQueryLoweringOptions::default())
}

pub fn lower_canonical_query_to_bcib_with_options(
    plan: &CanonicalPlan,
    options: CanonicalQueryLoweringOptions,
) -> Result<LoweredCanonicalQuery> {
    plan.validate()?;

    let instruction_count = if options.emit_trace { 3 } else { 2 };

    let mut instructions = vec![LoweredBcibInstruction {
        opcode: OPCODE_DATA_QUERY,
        operands: encode_data_query_operands(plan, instruction_count)?,
    }];

    if options.emit_trace {
        instructions.push(LoweredBcibInstruction {
            opcode: OPCODE_TRACE_EMIT,
            operands: encode_trace_operands(plan, instruction_count)?,
        });
    }

    instructions.push(LoweredBcibInstruction {
        opcode: OPCODE_END,
        operands: Vec::new(),
    });

    validate_lowered_instructions(&instructions)?;

    let bytes = build_v3_bcib_buffer(&instructions);
    let decoded = validate_canonical_query_bcib(&bytes)?;
    if decoded != instructions {
        return Err(SemanticCLIError::transform_error(
            "Canonical query BCIB decode roundtrip drifted from lowered instruction sequence",
            ErrorCode::E301,
        ));
    }

    let bcib_sha256 = sha256_hex(&bytes);
    let required_capabilities = vec![Capability::Read {
        context: plan.context_path.clone(),
    }];

    Ok(LoweredCanonicalQuery {
        command_kind: plan.command_kind,
        binding: plan.binding.clone(),
        required_capabilities,
        instructions,
        bytes,
        bcib_sha256,
    })
}

pub fn validate_canonical_query_bcib(bytes: &[u8]) -> Result<Vec<LoweredBcibInstruction>> {
    if bytes.len() < HEADER_SIZE + SECTION_ENTRY_SIZE {
        return Err(SemanticCLIError::validation_error(
            "Canonical query BCIB buffer is too short",
            "Emit a complete BCIB v3 buffer with header, section table and instructions",
            ErrorCode::E301,
        ));
    }

    if &bytes[0..4] != BCIB_MAGIC {
        return Err(SemanticCLIError::validation_error(
            "Canonical query BCIB magic is invalid",
            "Encode the BCIB buffer with the standard BCIB header",
            ErrorCode::E301,
        ));
    }

    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != BCIB_VERSION_V3 {
        return Err(SemanticCLIError::validation_error(
            format!(
                "Canonical query BCIB must use version 0x{:04x}, got 0x{:04x}",
                BCIB_VERSION_V3, version
            ),
            "Encode the production path as BCIB v3",
            ErrorCode::E301,
        ));
    }

    let section_count = u16::from_le_bytes([bytes[8], bytes[9]]);
    if section_count != 1 {
        return Err(SemanticCLIError::validation_error(
            format!(
                "Canonical query BCIB must expose exactly one instruction section, got {}",
                section_count
            ),
            "Emit a single instructions section for the canonical query program",
            ErrorCode::E301,
        ));
    }

    let section_id = u16::from_le_bytes([bytes[16], bytes[17]]);
    if section_id != INSTRUCTION_SECTION_ID {
        return Err(SemanticCLIError::validation_error(
            "Canonical query BCIB first section is not the instructions section",
            "Encode the first section as instructions",
            ErrorCode::E301,
        ));
    }

    let offset = u32::from_le_bytes([bytes[18], bytes[19], bytes[20], bytes[21]]) as usize;
    let length = u16::from_le_bytes([bytes[22], bytes[23]]) as usize;

    if offset < HEADER_SIZE + SECTION_ENTRY_SIZE || offset + length > bytes.len() {
        return Err(SemanticCLIError::validation_error(
            "Canonical query BCIB instruction section is out of bounds",
            "Encode a valid instruction section offset and length",
            ErrorCode::E301,
        ));
    }

    let mut cursor = offset;
    let end = offset + length;
    let mut instructions = Vec::new();

    while cursor < end {
        if cursor + 2 > end {
            return Err(SemanticCLIError::validation_error(
                "Canonical query BCIB instruction is truncated",
                "Encode opcode and operand count for every instruction",
                ErrorCode::E301,
            ));
        }

        let opcode = bytes[cursor];
        let operand_count = bytes[cursor + 1] as usize;
        cursor += 2;

        let operand_bytes = operand_count
            .checked_mul(4)
            .ok_or_else(|| SemanticCLIError::transform_error(
                "Canonical query BCIB operand count overflowed",
                ErrorCode::E301,
            ))?;

        if cursor + operand_bytes > end {
            return Err(SemanticCLIError::validation_error(
                "Canonical query BCIB operands are truncated",
                "Encode all operands as u32 little-endian values",
                ErrorCode::E301,
            ));
        }

        let mut operands = Vec::with_capacity(operand_count);
        for _ in 0..operand_count {
            operands.push(u32::from_le_bytes([
                bytes[cursor],
                bytes[cursor + 1],
                bytes[cursor + 2],
                bytes[cursor + 3],
            ]));
            cursor += 4;
        }

        instructions.push(LoweredBcibInstruction { opcode, operands });
    }

    validate_lowered_instructions(&instructions)?;
    Ok(instructions)
}

fn validate_lowered_instructions(instructions: &[LoweredBcibInstruction]) -> Result<()> {
    if instructions.is_empty() {
        return Err(SemanticCLIError::validation_error(
            "Canonical query BCIB cannot be empty",
            "Emit DataQuery followed by End",
            ErrorCode::E301,
        ));
    }

    let allowed_trace_count = instructions
        .iter()
        .filter(|instruction| instruction.opcode == OPCODE_TRACE_EMIT)
        .count();
    if allowed_trace_count > 1 {
        return Err(SemanticCLIError::validation_error(
            "Canonical query BCIB may include at most one TraceEmit instruction",
            "Emit zero or one TraceEmit after DataQuery",
            ErrorCode::E301,
        ));
    }

    let data_query_count = instructions
        .iter()
        .filter(|instruction| instruction.opcode == OPCODE_DATA_QUERY)
        .count();
    if data_query_count != 1 {
        return Err(SemanticCLIError::validation_error(
            format!(
                "Canonical query BCIB must contain exactly one DataQuery instruction, got {}",
                data_query_count
            ),
            "Emit exactly one DataQuery for the canonical query production path",
            ErrorCode::E301,
        ));
    }

    if instructions.last().map(|instruction| instruction.opcode) != Some(OPCODE_END) {
        return Err(SemanticCLIError::validation_error(
            "Canonical query BCIB must terminate with End",
            "Append End as the last instruction in the production path",
            ErrorCode::E301,
        ));
    }

    let instruction_count = instructions.len() as u32;

    for (index, instruction) in instructions.iter().enumerate() {
        for &operand in &instruction.operands {
            if operand >= instruction_count {
                return Err(SemanticCLIError::validation_error(
                    format!(
                        "Canonical query BCIB operand {} exceeds instruction bounds {}",
                        operand, instruction_count
                    ),
                    "Keep all lowered operands within the instruction-count bound required by the runtime verifier",
                    ErrorCode::E301,
                ));
            }
        }

        match instruction.opcode {
            OPCODE_DATA_QUERY => {
                if index != 0 {
                    return Err(SemanticCLIError::validation_error(
                        "Canonical query BCIB requires DataQuery to be the first instruction",
                        "Emit DataQuery before TraceEmit or End",
                        ErrorCode::E301,
                    ));
                }
            }
            OPCODE_TRACE_EMIT => {
                if index != 1 || instructions.first().map(|i| i.opcode) != Some(OPCODE_DATA_QUERY)
                {
                    return Err(SemanticCLIError::validation_error(
                        "Canonical query BCIB may place TraceEmit only between DataQuery and End",
                        "Emit TraceEmit immediately after DataQuery when tracing is enabled",
                        ErrorCode::E301,
                    ));
                }
            }
            OPCODE_END => {
                if index + 1 != instructions.len() {
                    return Err(SemanticCLIError::validation_error(
                        "Canonical query BCIB may only place End at the end of the program",
                        "Move End to the final instruction slot",
                        ErrorCode::E301,
                    ));
                }
            }
            OPCODE_NOP => {
                return Err(SemanticCLIError::validation_error(
                    "Canonical query BCIB forbids Nop in the production path",
                    "Lower semantic intent to DataQuery directly; do not use placeholder Nop",
                    ErrorCode::E301,
                ));
            }
            OPCODE_DATA_CREATE | OPCODE_DATA_ADD | OPCODE_UI_RENDER | OPCODE_AI_ASK => {
                return Err(SemanticCLIError::validation_error(
                    format!(
                        "Canonical query BCIB contains forbidden opcode 0x{:02x}",
                        instruction.opcode
                    ),
                    "Keep the production path limited to DataQuery, optional TraceEmit, and End",
                    ErrorCode::E301,
                ));
            }
            other => {
                return Err(SemanticCLIError::validation_error(
                    format!(
                        "Canonical query BCIB contains unsupported opcode 0x{:02x}",
                        other
                    ),
                    "Emit only DataQuery, optional TraceEmit, and End",
                    ErrorCode::E301,
                ));
            }
        }
    }

    Ok(())
}

fn encode_data_query_operands(plan: &CanonicalPlan, instruction_count: u32) -> Result<Vec<u32>> {
    let seed = serde_json::to_vec(&(plan.command_kind, &plan.binding)).map_err(|_| {
        SemanticCLIError::transform_error(
            "Canonical query binding could not be serialized for deterministic lowering",
            ErrorCode::E301,
        )
    })?;

    deterministic_operands(&seed, instruction_count, 32)
}

fn encode_trace_operands(plan: &CanonicalPlan, instruction_count: u32) -> Result<Vec<u32>> {
    let seed = serde_json::to_vec(&(plan.command_kind, plan.fingerprint_hex())).map_err(|_| {
        SemanticCLIError::transform_error(
            "Canonical query trace identity could not be serialized for deterministic lowering",
            ErrorCode::E301,
        )
    })?;

    deterministic_operands(&seed, instruction_count, 16)
}

fn deterministic_operands(
    seed: &[u8],
    instruction_count: u32,
    operand_count: usize,
) -> Result<Vec<u32>> {
    if instruction_count < 2 {
        return Err(SemanticCLIError::transform_error(
            "Canonical query BCIB requires at least DataQuery and End",
            ErrorCode::E301,
        ));
    }

    let digest = Sha256::digest(seed);
    let mut operands = Vec::with_capacity(operand_count);

    for index in 0..operand_count {
        let byte = digest[index % digest.len()];
        operands.push((byte as u32) % instruction_count);
    }

    Ok(operands)
}

fn build_v3_bcib_buffer(instructions: &[LoweredBcibInstruction]) -> Vec<u8> {
    let mut instruction_bytes = Vec::new();
    for instruction in instructions {
        instruction_bytes.push(instruction.opcode);
        instruction_bytes.push(instruction.operands.len() as u8);
        for operand in &instruction.operands {
            instruction_bytes.extend_from_slice(&operand.to_le_bytes());
        }
    }

    let instruction_offset = (HEADER_SIZE + SECTION_ENTRY_SIZE) as u32;
    let instruction_length = instruction_bytes.len() as u16;

    let mut buffer = Vec::with_capacity(HEADER_SIZE + SECTION_ENTRY_SIZE + instruction_bytes.len());
    buffer.extend_from_slice(BCIB_MAGIC);
    buffer.extend_from_slice(&BCIB_VERSION_V3.to_le_bytes());
    buffer.extend_from_slice(&0u16.to_le_bytes());
    buffer.extend_from_slice(&1u16.to_le_bytes());
    buffer.extend_from_slice(&[0u8; 4]);
    buffer.extend_from_slice(&[0u8; 2]);

    buffer.extend_from_slice(&INSTRUCTION_SECTION_ID.to_le_bytes());
    buffer.extend_from_slice(&instruction_offset.to_le_bytes());
    buffer.extend_from_slice(&instruction_length.to_le_bytes());
    buffer.extend_from_slice(&instruction_bytes);
    buffer
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    hex::encode(digest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical_query::parse_canonical_plan;
    use bcib_runtime::{BcibVerifierPlanner, CapabilitySet, ResourceLimits};

    #[test]
    fn canonical_query_lowering_emits_nop_free_bcib_for_list() {
        let plan = parse_canonical_plan("list data.users").unwrap();
        let lowered = lower_canonical_query_to_bcib(&plan).unwrap();

        assert_eq!(
            lowered.required_capabilities,
            vec![Capability::Read {
                context: "data.users".to_string()
            }]
        );
        assert_eq!(lowered.instructions.len(), 2);
        assert_eq!(lowered.instructions[0].opcode, OPCODE_DATA_QUERY);
        assert_eq!(lowered.instructions[1].opcode, OPCODE_END);
        assert!(!lowered.contains_forbidden_opcode());
        assert_eq!(lowered.bcib_sha256.len(), 64);

        let decoded = validate_canonical_query_bcib(&lowered.bytes).unwrap();
        assert_eq!(decoded, lowered.instructions);
    }

    #[test]
    fn canonical_query_lowering_preserves_show_vs_query_distinction() {
        let show_plan = parse_canonical_plan("show data.users 42").unwrap();
        let query_plan = parse_canonical_plan("query data.users {id == 42}").unwrap();

        let show_lowered = lower_canonical_query_to_bcib(&show_plan).unwrap();
        let query_lowered = lower_canonical_query_to_bcib(&query_plan).unwrap();

        assert_ne!(show_lowered.binding.predicate_kind, query_lowered.binding.predicate_kind);
        assert_ne!(
            show_lowered.instructions[0].operands,
            query_lowered.instructions[0].operands
        );
    }

    #[test]
    fn canonical_query_lowering_can_emit_trace_without_forbidden_opcodes() {
        let plan = parse_canonical_plan("query data.users {age > 18}").unwrap();
        let lowered = lower_canonical_query_to_bcib_with_options(
            &plan,
            CanonicalQueryLoweringOptions { emit_trace: true },
        )
        .unwrap();

        assert_eq!(lowered.instructions.len(), 3);
        assert_eq!(lowered.instructions[0].opcode, OPCODE_DATA_QUERY);
        assert_eq!(lowered.instructions[1].opcode, OPCODE_TRACE_EMIT);
        assert_eq!(lowered.instructions[2].opcode, OPCODE_END);
        assert!(!lowered.contains_forbidden_opcode());
    }

    #[test]
    fn canonical_query_lowering_validator_rejects_nop() {
        let bytes = build_v3_bcib_buffer(&[
            LoweredBcibInstruction {
                opcode: OPCODE_DATA_QUERY,
                operands: vec![1, 0, 1, 0, 1, 0],
            },
            LoweredBcibInstruction {
                opcode: OPCODE_NOP,
                operands: Vec::new(),
            },
            LoweredBcibInstruction {
                opcode: OPCODE_END,
                operands: Vec::new(),
            },
        ]);

        let error = validate_canonical_query_bcib(&bytes).unwrap_err();
        assert_eq!(error.code(), Some(ErrorCode::E301));
        assert!(error.to_string().contains("forbids Nop"));
    }

    #[test]
    fn canonical_query_lowering_validator_rejects_forbidden_opcode() {
        let bytes = build_v3_bcib_buffer(&[
            LoweredBcibInstruction {
                opcode: OPCODE_DATA_QUERY,
                operands: vec![1, 0, 1, 0, 1, 0],
            },
            LoweredBcibInstruction {
                opcode: OPCODE_UI_RENDER,
                operands: vec![0],
            },
            LoweredBcibInstruction {
                opcode: OPCODE_END,
                operands: Vec::new(),
            },
        ]);

        let error = validate_canonical_query_bcib(&bytes).unwrap_err();
        assert_eq!(error.code(), Some(ErrorCode::E301));
        assert!(error.to_string().contains("forbidden opcode"));
    }

    #[test]
    fn canonical_query_lowering_is_runtime_verifier_compatible() {
        let plan = parse_canonical_plan("query data.users {age > 18}").unwrap();
        let lowered = lower_canonical_query_to_bcib(&plan).unwrap();

        let planner = BcibVerifierPlanner::new();
        let caps = CapabilitySet { token_ids: vec![1] };
        let limits = ResourceLimits::default();

        let plan = planner.verify_and_plan(&lowered.bytes, &caps, &limits);
        assert!(plan.is_ok(), "runtime verifier should accept lowered BCIB");
    }
}
