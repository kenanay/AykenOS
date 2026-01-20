//! Property-based tests for BCIB OperandRef model (AR-1)
//!
//! These tests verify that the OperandRef model correctly replaces
//! embedded Value types to create a flat instruction graph.

use proptest::prelude::*;
use semantic_cli::bcib::*;
use semantic_cli::types::SourceLocation;

fn test_location() -> SourceLocation {
    SourceLocation::new(1, 1, 0)
}

// Property test generators for OperandRef
prop_compose! {
    fn arb_operand_ref()(
        operand_type in 0..3u8,
        field_name in "[a-zA-Z][a-zA-Z0-9_]*",
        string_value in ".*",
        number_value in any::<f64>().prop_filter("Must be finite", |n| n.is_finite()),
        bool_value in any::<bool>(),
        temp_reg in 0..1000u16,
    ) -> OperandRef {
        match operand_type {
            0 => OperandRef::Field(field_name),
            1 => {
                let value = match temp_reg % 3 {
                    0 => Value::String(string_value),
                    1 => Value::Number(number_value),
                    _ => Value::Boolean(bool_value),
                };
                OperandRef::Literal(value)
            }
            _ => OperandRef::TempRegister(temp_reg),
        }
    }
}

prop_compose! {
    fn arb_comparison_op()(op in 0..6u8) -> ComparisonOp {
        match op {
            0 => ComparisonOp::Equal,
            1 => ComparisonOp::NotEqual,
            2 => ComparisonOp::LessThan,
            3 => ComparisonOp::LessThanOrEqual,
            4 => ComparisonOp::GreaterThan,
            _ => ComparisonOp::GreaterThanOrEqual,
        }
    }
}

prop_compose! {
    fn arb_logical_op()(op in 0..3u8) -> LogicalOperator {
        match op {
            0 => LogicalOperator::And,
            1 => LogicalOperator::Or,
            _ => LogicalOperator::Not,
        }
    }
}

proptest! {
    /// Property 1: OperandRef Model Compliance (AR-1)
    /// For any Compare instruction, operands should use OperandRef instead of embedded Value types
    /// **Validates: Requirements AR-1.1**
    #[test]
    fn prop_compare_uses_operand_ref_model(
        left in arb_operand_ref(),
        right in arb_operand_ref(),
        op in arb_comparison_op(),
        target_reg in 0..100u16
    ) {
        let compare = QueryInstruction::Compare {
            left: left.clone(),
            operator: op,
            right: right.clone(),
            target_register: target_reg,
            location: test_location(),
        };

        // Verify that Compare uses OperandRef model
        match &compare {
            QueryInstruction::Compare { left: l, right: r, target_register: t, .. } => {
                // Should be OperandRef, not embedded Value
                assert!(matches!(l, OperandRef::Field(_) | OperandRef::Literal(_) | OperandRef::TempRegister(_)));
                assert!(matches!(r, OperandRef::Field(_) | OperandRef::Literal(_) | OperandRef::TempRegister(_)));
                // Should have target register for flat instruction graph
                assert_eq!(*t, target_reg);
            }
            _ => panic!("Should be Compare instruction"),
        }

        // Validation should work for valid operands
        if left.validate().is_ok() && right.validate().is_ok() {
            prop_assert!(compare.validate().is_ok());
        }
    }

    /// Property 2: LogicalOp OperandRef Model Compliance (AR-1)
    /// For any LogicalOp instruction, operands should use Vec<OperandRef> instead of Vec<Value>
    /// **Validates: Requirements AR-1.1**
    #[test]
    fn prop_logical_op_uses_operand_ref_model(
        operands in prop::collection::vec(arb_operand_ref(), 1..4),
        op in arb_logical_op(),
        target_reg in 0..100u16
    ) {
        let logical_op = QueryInstruction::LogicalOp {
            operator: op,
            operands: operands.clone(),
            target_register: target_reg,
            location: test_location(),
        };

        // Verify that LogicalOp uses Vec<OperandRef> model
        match &logical_op {
            QueryInstruction::LogicalOp { operands: ops, target_register: t, .. } => {
                // Should be Vec<OperandRef>, not Vec<Value>
                for operand in ops {
                    assert!(matches!(operand, OperandRef::Field(_) | OperandRef::Literal(_) | OperandRef::TempRegister(_)));
                }
                // Should have target register for flat instruction graph
                assert_eq!(*t, target_reg);
            }
            _ => panic!("Should be LogicalOp instruction"),
        }

        // Validation should respect operand count rules
        let expected_valid = match op {
            LogicalOperator::Not => operands.len() == 1,
            LogicalOperator::And | LogicalOperator::Or => operands.len() == 2,
        } && operands.iter().all(|op| op.validate().is_ok());

        prop_assert_eq!(logical_op.validate().is_ok(), expected_valid);
    }

    /// Property 3: OperandRef Validation Correctness
    /// For any OperandRef, validation should correctly accept valid operands and reject invalid ones
    /// **Validates: Requirements AR-1.2**
    #[test]
    fn prop_operand_ref_validation_correctness(operand in arb_operand_ref()) {
        let validation_result = operand.validate();

        match &operand {
            OperandRef::Field(field) => {
                // Field references should be valid if non-empty
                prop_assert_eq!(validation_result.is_ok(), !field.is_empty());
            }
            OperandRef::Literal(value) => {
                // Literal operands should have same validation as underlying Value
                prop_assert_eq!(validation_result.is_ok(), value.validate().is_ok());
            }
            OperandRef::TempRegister(_) => {
                // Temp registers are always valid
                prop_assert!(validation_result.is_ok());
            }
        }
    }

    /// Property 4: OperandRef Type Detection
    /// For any OperandRef, type detection should correctly identify the operand type
    /// **Validates: Requirements AR-1.2**
    #[test]
    fn prop_operand_ref_type_detection(operand in arb_operand_ref()) {
        let operand_type = operand.operand_type();

        match &operand {
            OperandRef::Field(_) => {
                prop_assert_eq!(operand_type, OperandType::Field);
            }
            OperandRef::Literal(value) => {
                prop_assert_eq!(operand_type, OperandType::Literal(value.value_type()));
            }
            OperandRef::TempRegister(_) => {
                prop_assert_eq!(operand_type, OperandType::TempRegister);
            }
        }
    }

    /// Property 5: BCIB Instruction Graph Flatness (AR-1)
    /// For any BCIB sequence with Compare/LogicalOp instructions, 
    /// the sequence should be a flat list without nested expression trees
    /// **Validates: Requirements AR-1.2**
    #[test]
    fn prop_bcib_instruction_graph_flatness(
        left in arb_operand_ref(),
        right in arb_operand_ref(),
        op in arb_comparison_op(),
        target_reg in 0..100u16
    ) {
        let instructions = vec![
            BCIBInstruction::Context(ContextInstruction::LoadContext {
                path: "data.users".to_string(),
                location: test_location(),
            }),
            BCIBInstruction::Query(QueryInstruction::Compare {
                left: left.clone(),
                operator: op,
                right: right.clone(),
                target_register: target_reg,
                location: test_location(),
            }),
        ];

        let sequence = BCIBSequence::new(instructions);

        // Verify that the sequence is a flat list
        prop_assert_eq!(sequence.instructions.len(), 2);

        // Verify that Compare instruction uses OperandRef (no nested expressions)
        if let BCIBInstruction::Query(QueryInstruction::Compare { left: l, right: r, target_register: t, .. }) = &sequence.instructions[1] {
            // Should be direct OperandRef, not nested expression trees
            prop_assert!(matches!(l, OperandRef::Field(_) | OperandRef::Literal(_) | OperandRef::TempRegister(_)));
            prop_assert!(matches!(r, OperandRef::Field(_) | OperandRef::Literal(_) | OperandRef::TempRegister(_)));
            // Should have target register for flat instruction graph
            prop_assert_eq!(*t, target_reg);
        } else {
            prop_assert!(false, "Second instruction should be Compare");
        }

        // Validation should work if operands are valid
        if left.validate().is_ok() && right.validate().is_ok() {
            prop_assert!(sequence.validate().is_ok());
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_operand_ref_model_examples() {
        // Example: Compare instruction using OperandRef model (AR-1)
        let compare = QueryInstruction::Compare {
            left: OperandRef::Field("age".to_string()),
            operator: ComparisonOp::GreaterThan,
            right: OperandRef::Literal(Value::Number(18.0)),
            target_register: 0,
            location: test_location(),
        };

        assert!(compare.validate().is_ok());

        // Verify it uses OperandRef, not embedded Value
        match compare {
            QueryInstruction::Compare { left, right, target_register, .. } => {
                assert!(matches!(left, OperandRef::Field(_)));
                assert!(matches!(right, OperandRef::Literal(_)));
                assert_eq!(target_register, 0);
            }
            _ => panic!("Should be Compare instruction"),
        }
    }

    #[test]
    fn test_logical_op_operand_ref_model() {
        // Example: LogicalOp instruction using OperandRef model (AR-1)
        let logical_op = QueryInstruction::LogicalOp {
            operator: LogicalOperator::And,
            operands: vec![
                OperandRef::Field("active".to_string()),
                OperandRef::Literal(Value::Boolean(true)),
            ],
            target_register: 1,
            location: test_location(),
        };

        assert!(logical_op.validate().is_ok());

        // Verify it uses Vec<OperandRef>, not Vec<Value>
        match logical_op {
            QueryInstruction::LogicalOp { operands, target_register, .. } => {
                assert_eq!(operands.len(), 2);
                assert!(matches!(operands[0], OperandRef::Field(_)));
                assert!(matches!(operands[1], OperandRef::Literal(_)));
                assert_eq!(target_register, 1);
            }
            _ => panic!("Should be LogicalOp instruction"),
        }
    }

    #[test]
    fn test_temp_register_operands() {
        // Example: Using temporary registers for intermediate results (AR-1)
        let compare_with_temp = QueryInstruction::Compare {
            left: OperandRef::TempRegister(0),
            operator: ComparisonOp::Equal,
            right: OperandRef::TempRegister(1),
            target_register: 2,
            location: test_location(),
        };

        assert!(compare_with_temp.validate().is_ok());

        // Verify temp registers are valid operands
        assert_eq!(OperandRef::TempRegister(0).operand_type(), OperandType::TempRegister);
        assert_eq!(OperandRef::TempRegister(1).operand_type(), OperandType::TempRegister);
    }
}