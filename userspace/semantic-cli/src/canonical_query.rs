//! Canonical query planning surface.
//!
//! This module exposes a purpose-named semantic surface for canonical query
//! planning. Phase labels belong in specs and roadmap documents; implementation
//! modules stay named by stable responsibility.
//!
//! The current slice freezes:
//!
//! `DSL -> canonical semantic plan -> canonical IR`
//!
//! BCIB lowering and submission stay out of this module on purpose.

use crate::ast::{AstNode, BinaryOp, CommandNode, Expr, UnaryOp};
use crate::bcib::{ComparisonOp, LogicalOperator, Value};
use crate::error::{ErrorCode, Result, SemanticCLIError};
use crate::execution_plan::RegisterId;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::types::SourceLocation;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const ACTIVE_CONTEXT_REGISTER: RegisterId = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalCommandKind {
    List,
    Show,
    Query,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalPredicateKind {
    All,
    IdEq,
    Filter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalQueryBinding {
    pub context_path: String,
    pub predicate_kind: CanonicalPredicateKind,
    pub predicate_fingerprint: Option<String>,
}

impl CanonicalQueryBinding {
    pub fn fingerprint_hex(&self) -> String {
        fingerprint_hex(self)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CanonicalPredicate {
    Compare {
        field: String,
        operator: ComparisonOp,
        value: Value,
    },
    Logical {
        operator: LogicalOperator,
        inputs: Vec<CanonicalPredicate>,
    },
}

impl CanonicalPredicate {
    pub fn fingerprint_hex(&self) -> String {
        fingerprint_hex(self)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CanonicalIrInstruction {
    LoadContext {
        context: String,
        out: RegisterId,
    },
    LoadField {
        src: RegisterId,
        field: String,
        out: RegisterId,
    },
    LoadLiteral {
        value: Value,
        out: RegisterId,
    },
    Compare {
        left: RegisterId,
        op: ComparisonOp,
        right: RegisterId,
        out: RegisterId,
    },
    LogicalOp {
        op: LogicalOperator,
        inputs: Vec<RegisterId>,
        out: RegisterId,
    },
    ApplyFilter {
        ctx: RegisterId,
        predicate: RegisterId,
        out: RegisterId,
    },
    Return {
        src: RegisterId,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalPlan {
    pub command_kind: CanonicalCommandKind,
    pub location: SourceLocation,
    pub context_path: String,
    pub predicate: Option<CanonicalPredicate>,
    pub binding: CanonicalQueryBinding,
    pub instructions: Vec<CanonicalIrInstruction>,
}

impl CanonicalPlan {
    pub fn validate(&self) -> Result<()> {
        if self.context_path.is_empty() {
            return Err(SemanticCLIError::semantic_error(
                self.location,
                "Phase-16A canonical plan requires a non-empty context path",
                "Provide a queryable context like 'data.users'",
                ErrorCode::E100,
            ));
        }

        if !self.context_path.contains('.') {
            return Err(SemanticCLIError::semantic_error(
                self.location,
                format!(
                    "Phase-16A context path '{}' must contain at least one '.' separator",
                    self.context_path
                ),
                "Use a dotted context path like 'data.users'",
                ErrorCode::E100,
            ));
        }

        if self.instructions.is_empty() {
            return Err(SemanticCLIError::transform_error(
                "Phase-16A canonical plan cannot be empty",
                ErrorCode::E300,
            ));
        }

        match self.instructions.first() {
            Some(CanonicalIrInstruction::LoadContext { context, out }) => {
                if *out != ACTIVE_CONTEXT_REGISTER {
                    return Err(SemanticCLIError::transform_error(
                        "Phase-16A LoadContext must target register r0",
                        ErrorCode::E300,
                    ));
                }

                if context != &self.context_path {
                    return Err(SemanticCLIError::transform_error(
                        "Phase-16A LoadContext must use the canonical context path",
                        ErrorCode::E300,
                    ));
                }
            }
            _ => {
                return Err(SemanticCLIError::transform_error(
                    "Phase-16A canonical plans must start with LoadContext",
                    ErrorCode::E300,
                ));
            }
        }

        match self.instructions.last() {
            Some(CanonicalIrInstruction::Return { src }) if *src == ACTIVE_CONTEXT_REGISTER => {}
            _ => {
                return Err(SemanticCLIError::transform_error(
                    "Phase-16A canonical plans must end with Return { src: r0 }",
                    ErrorCode::E300,
                ));
            }
        }

        match (
            &self.command_kind,
            &self.predicate,
            self.binding.predicate_kind,
        ) {
            (CanonicalCommandKind::List, None, CanonicalPredicateKind::All) => {}
            (CanonicalCommandKind::Show, Some(_), CanonicalPredicateKind::IdEq) => {}
            (CanonicalCommandKind::Query, Some(_), CanonicalPredicateKind::Filter) => {}
            _ => {
                return Err(SemanticCLIError::transform_error(
                    "Phase-16A command, predicate and binding kind are inconsistent",
                    ErrorCode::E300,
                ));
            }
        }

        let apply_filter_count = self
            .instructions
            .iter()
            .filter(|instruction| matches!(instruction, CanonicalIrInstruction::ApplyFilter { .. }))
            .count();

        match self.command_kind {
            CanonicalCommandKind::List if apply_filter_count == 0 => {}
            CanonicalCommandKind::Show | CanonicalCommandKind::Query if apply_filter_count == 1 => {
            }
            _ => {
                return Err(SemanticCLIError::transform_error(
                    "Phase-16A canonical plans must be filter-free for list and single-filter for show/query",
                    ErrorCode::E300,
                ));
            }
        }

        Ok(())
    }

    pub fn fingerprint_hex(&self) -> String {
        fingerprint_hex(self)
    }
}

pub fn parse_canonical_plan(input: &str) -> Result<CanonicalPlan> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let ast = Parser::parse(tokens)?;
    build_canonical_plan(&ast)
}

pub fn build_canonical_plan(ast: &AstNode) -> Result<CanonicalPlan> {
    build_canonical_plan_from_command(&ast.command)
}

pub fn build_canonical_plan_from_command(command: &CommandNode) -> Result<CanonicalPlan> {
    let mut builder = CanonicalPlanBuilder::new();
    let plan = match command {
        CommandNode::List { location, context } => builder.build_list(*location, context)?,
        CommandNode::Show {
            location,
            context,
            id,
        } => builder.build_show(*location, context, id)?,
        CommandNode::Query {
            location,
            context,
            filter,
        } => builder.build_query(*location, context, filter.as_ref())?,
        _ => {
            return Err(unsupported_canonical_query_command(command.location()));
        }
    };

    plan.validate()?;
    Ok(plan)
}

struct CanonicalPlanBuilder {
    instructions: Vec<CanonicalIrInstruction>,
    next_register: RegisterId,
}

impl CanonicalPlanBuilder {
    fn new() -> Self {
        Self {
            instructions: Vec::new(),
            next_register: ACTIVE_CONTEXT_REGISTER + 1,
        }
    }

    fn build_list(
        &mut self,
        location: SourceLocation,
        context: &[String],
    ) -> Result<CanonicalPlan> {
        let context_path = canonicalize_context_path(context, location)?;
        self.instructions.push(CanonicalIrInstruction::LoadContext {
            context: context_path.clone(),
            out: ACTIVE_CONTEXT_REGISTER,
        });
        self.instructions.push(CanonicalIrInstruction::Return {
            src: ACTIVE_CONTEXT_REGISTER,
        });

        Ok(CanonicalPlan {
            command_kind: CanonicalCommandKind::List,
            location,
            context_path: context_path.clone(),
            predicate: None,
            binding: CanonicalQueryBinding {
                context_path,
                predicate_kind: CanonicalPredicateKind::All,
                predicate_fingerprint: None,
            },
            instructions: self.instructions.clone(),
        })
    }

    fn build_show(
        &mut self,
        location: SourceLocation,
        context: &[String],
        id: &Expr,
    ) -> Result<CanonicalPlan> {
        let context_path = canonicalize_context_path(context, location)?;
        let id_value = expr_to_literal_value(id)?;
        let predicate = CanonicalPredicate::Compare {
            field: "id".to_string(),
            operator: ComparisonOp::Equal,
            value: id_value,
        };

        self.instructions.push(CanonicalIrInstruction::LoadContext {
            context: context_path.clone(),
            out: ACTIVE_CONTEXT_REGISTER,
        });

        let predicate_register = self.emit_predicate(&predicate)?;
        self.instructions.push(CanonicalIrInstruction::ApplyFilter {
            ctx: ACTIVE_CONTEXT_REGISTER,
            predicate: predicate_register,
            out: ACTIVE_CONTEXT_REGISTER,
        });
        self.instructions.push(CanonicalIrInstruction::Return {
            src: ACTIVE_CONTEXT_REGISTER,
        });

        Ok(CanonicalPlan {
            command_kind: CanonicalCommandKind::Show,
            location,
            context_path: context_path.clone(),
            binding: CanonicalQueryBinding {
                context_path,
                predicate_kind: CanonicalPredicateKind::IdEq,
                predicate_fingerprint: Some(predicate.fingerprint_hex()),
            },
            predicate: Some(predicate),
            instructions: self.instructions.clone(),
        })
    }

    fn build_query(
        &mut self,
        location: SourceLocation,
        context: &[String],
        filter: Option<&Expr>,
    ) -> Result<CanonicalPlan> {
        let context_path = canonicalize_context_path(context, location)?;
        let filter = filter.ok_or_else(|| {
            SemanticCLIError::semantic_error(
                location,
                "Phase-16A query requires an explicit predicate",
                "Use `list <context>` for unfiltered reads or add a filter predicate",
                ErrorCode::E103,
            )
        })?;
        let predicate = canonicalize_predicate(filter)?;

        self.instructions.push(CanonicalIrInstruction::LoadContext {
            context: context_path.clone(),
            out: ACTIVE_CONTEXT_REGISTER,
        });

        let predicate_register = self.emit_predicate(&predicate)?;
        self.instructions.push(CanonicalIrInstruction::ApplyFilter {
            ctx: ACTIVE_CONTEXT_REGISTER,
            predicate: predicate_register,
            out: ACTIVE_CONTEXT_REGISTER,
        });
        self.instructions.push(CanonicalIrInstruction::Return {
            src: ACTIVE_CONTEXT_REGISTER,
        });

        Ok(CanonicalPlan {
            command_kind: CanonicalCommandKind::Query,
            location,
            context_path: context_path.clone(),
            binding: CanonicalQueryBinding {
                context_path,
                predicate_kind: CanonicalPredicateKind::Filter,
                predicate_fingerprint: Some(predicate.fingerprint_hex()),
            },
            predicate: Some(predicate),
            instructions: self.instructions.clone(),
        })
    }

    fn emit_predicate(&mut self, predicate: &CanonicalPredicate) -> Result<RegisterId> {
        match predicate {
            CanonicalPredicate::Compare {
                field,
                operator,
                value,
            } => {
                let field_register = self.allocate_register();
                self.instructions.push(CanonicalIrInstruction::LoadField {
                    src: ACTIVE_CONTEXT_REGISTER,
                    field: field.clone(),
                    out: field_register,
                });

                let literal_register = self.allocate_register();
                self.instructions.push(CanonicalIrInstruction::LoadLiteral {
                    value: value.clone(),
                    out: literal_register,
                });

                let compare_register = self.allocate_register();
                self.instructions.push(CanonicalIrInstruction::Compare {
                    left: field_register,
                    op: *operator,
                    right: literal_register,
                    out: compare_register,
                });

                Ok(compare_register)
            }
            CanonicalPredicate::Logical { operator, inputs } => {
                if inputs.is_empty() {
                    return Err(SemanticCLIError::transform_error(
                        "Phase-16A logical predicates require at least one input",
                        ErrorCode::E300,
                    ));
                }

                let mut input_registers = Vec::with_capacity(inputs.len());
                for input in inputs {
                    input_registers.push(self.emit_predicate(input)?);
                }

                let logical_register = self.allocate_register();
                self.instructions.push(CanonicalIrInstruction::LogicalOp {
                    op: *operator,
                    inputs: input_registers,
                    out: logical_register,
                });
                Ok(logical_register)
            }
        }
    }

    fn allocate_register(&mut self) -> RegisterId {
        let register = self.next_register;
        self.next_register += 1;
        register
    }
}

fn canonicalize_context_path(context: &[String], location: SourceLocation) -> Result<String> {
    if context.is_empty() || context.iter().any(|segment| segment.trim().is_empty()) {
        return Err(SemanticCLIError::semantic_error(
            location,
            "Phase-16A requires a non-empty dotted context path",
            "Provide a context like 'data.users'",
            ErrorCode::E100,
        ));
    }

    let context_path = context.join(".");
    if !context_path.contains('.') {
        return Err(SemanticCLIError::semantic_error(
            location,
            format!(
                "Phase-16A context '{}' is too narrow for canonical query binding",
                context_path
            ),
            "Use a dotted context path like 'data.users'",
            ErrorCode::E100,
        ));
    }

    Ok(context_path)
}

fn canonicalize_predicate(expr: &Expr) -> Result<CanonicalPredicate> {
    match expr {
        Expr::Binary {
            left,
            op,
            right,
            location,
        } => match op {
            BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge => Ok(CanonicalPredicate::Compare {
                field: expr_to_field_name(left)?,
                operator: binary_op_to_comparison(*op),
                value: expr_to_literal_value(right)?,
            }),
            BinaryOp::And | BinaryOp::Or => Ok(CanonicalPredicate::Logical {
                operator: binary_op_to_logical(*op).ok_or_else(|| {
                    SemanticCLIError::semantic_error(
                        *location,
                        "Phase-16A logical predicate is not supported",
                        "Use comparison predicates joined by 'and' or 'or'",
                        ErrorCode::E103,
                    )
                })?,
                inputs: vec![
                    canonicalize_predicate(left)?,
                    canonicalize_predicate(right)?,
                ],
            }),
        },
        Expr::Unary {
            op: UnaryOp::Not,
            operand,
            location: _,
        } => Ok(CanonicalPredicate::Logical {
            operator: LogicalOperator::Not,
            inputs: vec![canonicalize_predicate(operand)?],
        }),
        _ => Err(SemanticCLIError::semantic_error(
            expr.location(),
            "Phase-16A predicates must be comparisons or logical compositions",
            "Use expressions like `age > 18`, `active == true`, or combine them with `and` / `or` / `not`",
            ErrorCode::E103,
        )),
    }
}

fn expr_to_field_name(expr: &Expr) -> Result<String> {
    match expr {
        Expr::Identifier { name, .. } => Ok(name.clone()),
        _ => Err(SemanticCLIError::semantic_error(
            expr.location(),
            "Phase-16A comparisons require a field identifier on the left-hand side",
            "Rewrite the predicate as `<field> <op> <literal>`",
            ErrorCode::E102,
        )),
    }
}

fn expr_to_literal_value(expr: &Expr) -> Result<Value> {
    match expr {
        Expr::Number { value, .. } => value.parse::<f64>().map(Value::Number).map_err(|_| {
            SemanticCLIError::semantic_error(
                expr.location(),
                format!("Invalid numeric literal '{}'", value),
                "Use a finite numeric literal",
                ErrorCode::E005,
            )
        }),
        Expr::String { value, .. } => Ok(Value::String(value.clone())),
        Expr::Boolean { value, .. } => Ok(Value::Boolean(*value)),
        _ => Err(SemanticCLIError::semantic_error(
            expr.location(),
            "Phase-16A literal position only accepts number, string or boolean values",
            "Use a literal on the right-hand side of the predicate",
            ErrorCode::E005,
        )),
    }
}

fn binary_op_to_comparison(op: BinaryOp) -> ComparisonOp {
    match op {
        BinaryOp::Eq => ComparisonOp::Equal,
        BinaryOp::Ne => ComparisonOp::NotEqual,
        BinaryOp::Lt => ComparisonOp::LessThan,
        BinaryOp::Le => ComparisonOp::LessThanOrEqual,
        BinaryOp::Gt => ComparisonOp::GreaterThan,
        BinaryOp::Ge => ComparisonOp::GreaterThanOrEqual,
        BinaryOp::And | BinaryOp::Or => {
            unreachable!("logical operators cannot map to ComparisonOp")
        }
    }
}

fn binary_op_to_logical(op: BinaryOp) -> Option<LogicalOperator> {
    match op {
        BinaryOp::And => Some(LogicalOperator::And),
        BinaryOp::Or => Some(LogicalOperator::Or),
        _ => None,
    }
}

fn unsupported_canonical_query_command(location: SourceLocation) -> SemanticCLIError {
    SemanticCLIError::semantic_error(
        location,
        "Phase-16A only supports list, show and query commands",
        "Use `list <context>`, `show <context> <id>`, or `query <context> {predicate}`",
        ErrorCode::E103,
    )
}

fn fingerprint_hex<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value)
        .expect("Phase-16A fingerprint generation requires serializable canonical data");
    let digest = Sha256::digest(bytes);
    hex::encode(digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expect_compare(
        predicate: &CanonicalPredicate,
        field: &str,
        operator: ComparisonOp,
        value: Value,
    ) {
        match predicate {
            CanonicalPredicate::Compare {
                field: actual_field,
                operator: actual_operator,
                value: actual_value,
            } => {
                assert_eq!(actual_field, field);
                assert_eq!(*actual_operator, operator);
                assert_eq!(actual_value, &value);
            }
            _ => panic!("expected comparison predicate"),
        }
    }

    #[test]
    fn canonical_query_builds_list_plan_from_dsl() {
        let plan = parse_canonical_plan("list data.users").unwrap();

        assert_eq!(plan.command_kind, CanonicalCommandKind::List);
        assert_eq!(plan.context_path, "data.users");
        assert!(plan.predicate.is_none());
        assert_eq!(plan.binding.predicate_kind, CanonicalPredicateKind::All);
        assert_eq!(
            plan.instructions,
            vec![
                CanonicalIrInstruction::LoadContext {
                    context: "data.users".to_string(),
                    out: 0,
                },
                CanonicalIrInstruction::Return { src: 0 },
            ]
        );
    }

    #[test]
    fn canonical_query_builds_show_plan_from_dsl() {
        let plan = parse_canonical_plan("show data.users 42").unwrap();

        assert_eq!(plan.command_kind, CanonicalCommandKind::Show);
        assert_eq!(plan.binding.predicate_kind, CanonicalPredicateKind::IdEq);
        assert!(plan.binding.predicate_fingerprint.is_some());

        let predicate = plan.predicate.as_ref().unwrap();
        expect_compare(predicate, "id", ComparisonOp::Equal, Value::Number(42.0));

        assert_eq!(plan.instructions.len(), 6);
        assert!(matches!(
            plan.instructions[1],
            CanonicalIrInstruction::LoadField { ref field, src: 0, .. } if field == "id"
        ));
        assert!(matches!(
            plan.instructions[4],
            CanonicalIrInstruction::ApplyFilter {
                ctx: 0,
                predicate: _,
                out: 0,
            }
        ));
    }

    #[test]
    fn canonical_query_builds_logical_query_plan_from_dsl() {
        let plan = parse_canonical_plan("query data.users {age > 18 and active == true}").unwrap();

        assert_eq!(plan.command_kind, CanonicalCommandKind::Query);
        assert_eq!(plan.binding.predicate_kind, CanonicalPredicateKind::Filter);
        assert!(plan.binding.predicate_fingerprint.is_some());

        match plan.predicate.as_ref().unwrap() {
            CanonicalPredicate::Logical { operator, inputs } => {
                assert_eq!(*operator, LogicalOperator::And);
                assert_eq!(inputs.len(), 2);
            }
            _ => panic!("expected logical predicate"),
        }

        assert!(plan.instructions.iter().any(|instruction| matches!(
            instruction,
            CanonicalIrInstruction::LogicalOp {
                op: LogicalOperator::And,
                ..
            }
        )));
        assert!(matches!(
            plan.instructions.last().unwrap(),
            CanonicalIrInstruction::Return { src: 0 }
        ));
    }

    #[test]
    fn canonical_query_rejects_query_without_filter() {
        let error = parse_canonical_plan("query data.users").unwrap_err();

        assert_eq!(error.code(), Some(ErrorCode::E103));
        assert!(error.to_string().contains("requires an explicit predicate"));
    }

    #[test]
    fn canonical_query_rejects_unsupported_command_surface() {
        let error = parse_canonical_plan("status").unwrap_err();

        assert_eq!(error.code(), Some(ErrorCode::E103));
        assert!(error
            .to_string()
            .contains("only supports list, show and query commands"));
    }

    #[test]
    fn canonical_query_rejects_non_literal_show_identifier() {
        let error = parse_canonical_plan("show data.users user_id").unwrap_err();

        assert_eq!(error.code(), Some(ErrorCode::E005));
        assert!(error
            .to_string()
            .contains("literal position only accepts number, string or boolean values"));
    }

    #[test]
    fn canonical_query_generates_stable_binding_fingerprint() {
        let plan = parse_canonical_plan("query data.users {age > 18}").unwrap();
        let first = plan.binding.fingerprint_hex();
        let second = plan.binding.fingerprint_hex();

        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
    }
}
