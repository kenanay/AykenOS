# Phase 4.1.0 — Warning Inventory
**AykenOS Constitutional Compliance Document**

## Scope & Freeze Declaration
- **Phase:** 4.1.0
- **Status:** Inventory Only (NO CODE CHANGE)
- **Date:** 2026-01-20
- **Total Warnings:** 144

🔒 **During this phase:**
- ❌ No code deletion
- ❌ No refactoring
- ❌ No API changes
- ✅ Classification only

## Warning Summary (Total View)

| Category | Count | Clean | Intentional | Phase 5 |
|----------|-------|-------|-------------|---------|
| dead_code | 1 | ☐ | ☐ | ☐ |
| deprecated | 2 | ☐ | ☐ | ☐ |
| lifetime_syntax | 2 | ☐ | ☐ | ☐ |
| meaningless_assertion | 19 | ☐ | ☐ | ☐ |
| must_use | 1 | ☐ | ☐ | ☐ |
| unused_import | 90 | ☐ | ☐ | ☐ |
| unused_mut | 7 | ☐ | ☐ | ☐ |
| unused_variable | 22 | ☐ | ☐ | ☐ |
| **TOTAL** | **144** | ☐ | ☐ | ☐ |

📌 **Source Commands:**
```bash
cargo test --lib 2>&1 | tee warnings.log
cargo clippy --all-targets --all-features 2>&1 | tee -a warnings.log
```

## Detailed Inventory (Line-by-Line Record)

**RULE:** Every warning is recorded individually. No "I know this" shortcuts.

### INV-4.1-unused_import-0001
- **Category:** `unused_import`
- **File:** `semantic-cli/src/validator.rs`
- **Line:** `37`

#### Current Status
warning: unused imports: `ComparisonOp`, `LogicalOperator`, and `Value`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0002
- **Category:** `unused_import`
- **File:** `semantic-cli/src/validator.rs`
- **Line:** `45`

#### Current Status
warning: unused imports: `BinaryOp`, `Expr`, and `UnaryOp`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0003
- **Category:** `unused_import`
- **File:** `semantic-cli/src/validator.rs`
- **Line:** `698`

#### Current Status
warning: unused import: `LogicalOperator`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0004
- **Category:** `unused_import`
- **File:** `semantic-cli/src/transformer.rs`
- **Line:** `39`

#### Current Status
warning: unused imports: `Capability` and `SystemScope`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0005
- **Category:** `unused_import`
- **File:** `semantic-cli/src/normalizer/instruction_orderer.rs`
- **Line:** `9`

#### Current Status
warning: unused import: `OperandRef`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0006
- **Category:** `unused_import`
- **File:** `semantic-cli/src/normalizer/instruction_orderer.rs`
- **Line:** `12`

#### Current Status
warning: unused import: `HashSet`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0007
- **Category:** `unused_import`
- **File:** `semantic-cli/src/normalizer/instruction_orderer.rs`
- **Line:** `319`

#### Current Status
warning: unused import: `DependencyTracker`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0008
- **Category:** `unused_import`
- **File:** `semantic-cli/src/normalizer/validator.rs`
- **Line:** `377`

#### Current Status
warning: unused imports: `ComparisonOp` and `Value`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0009
- **Category:** `unused_import`
- **File:** `semantic-cli/src/execution_plan/mod.rs`
- **Line:** `13`

#### Current Status
warning: unused import: `NormalizedInstruction`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0010
- **Category:** `unused_import`
- **File:** `semantic-cli/src/execution_plan/mod.rs`
- **Line:** `14`

#### Current Status
warning: unused imports: `BCIBInstruction`, `ContextInstruction`, and `QueryInstruction`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0011
- **Category:** `unused_import`
- **File:** `semantic-cli/src/execution_plan/builder.rs`
- **Line:** `12`

#### Current Status
warning: unused import: `ExecutionPlanError`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0012
- **Category:** `unused_import`
- **File:** `semantic-cli/src/execution_plan/builder.rs`
- **Line:** `15`

#### Current Status
warning: unused import: `Value`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0013
- **Category:** `unused_import`
- **File:** `semantic-cli/src/execution_plan/validator.rs`
- **Line:** `10`

#### Current Status
warning: unused import: `DataflowGraph`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0014
- **Category:** `unused_import`
- **File:** `semantic-cli/src/execution_plan/validator.rs`
- **Line:** `11`

#### Current Status
warning: unused import: `HashMap`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0015
- **Category:** `unused_import`
- **File:** `semantic-cli/src/execution_plan/validator.rs`
- **Line:** `564`

#### Current Status
warning: unused import: `crate::bcib::Value`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0016
- **Category:** `unused_import`
- **File:** `semantic-cli/src/execution_plan/mod.rs`
- **Line:** `477`

#### Current Status
warning: unused imports: `InstructionGroup` and `NormalizedInstruction`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0017
- **Category:** `unused_import`
- **File:** `semantic-cli/src/ir_planner/mod.rs`
- **Line:** `12`

#### Current Status
warning: unused import: `IRBlock`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0018
- **Category:** `unused_import`
- **File:** `semantic-cli/src/ir_planner/replay.rs`
- **Line:** `11`

#### Current Status
warning: unused import: `crate::execution_plan::IRInstruction`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0019
- **Category:** `unused_import`
- **File:** `semantic-cli/src/ir_planner/replay.rs`
- **Line:** `413`

#### Current Status
warning: unused import: `crate::execution_plan::RegisterId`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0020
- **Category:** `unused_import`
- **File:** `semantic-cli/src/ir_planner/mod.rs`
- **Line:** `27`

#### Current Status
warning: unused import: `ReplayStep`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0021
- **Category:** `unused_import`
- **File:** `semantic-cli/src/parallelism/implementation/error.rs`
- **Line:** `444`

#### Current Status
warning: unused import: `crate::execution_plan::BlockId`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0022
- **Category:** `unused_import`
- **File:** `semantic-cli/src/parallelism/implementation/executor.rs`
- **Line:** `18`

#### Current Status
warning: unused import: `BlockId`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0023
- **Category:** `unused_import`
- **File:** `semantic-cli/src/parallelism/implementation/verification.rs`
- **Line:** `21`

#### Current Status
warning: unused import: `ParallelismError`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0024
- **Category:** `unused_import`
- **File:** `semantic-cli/src/parallelism/implementation/tests/constitutional_compliance.rs`
- **Line:** `7`

#### Current Status
warning: unused import: `ParallelismResult`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0025
- **Category:** `unused_import`
- **File:** `semantic-cli/src/loop_engine/fingerprint.rs`
- **Line:** `1936`

#### Current Status
warning: unused import: `state::LoopState`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0026
- **Category:** `unused_import`
- **File:** `semantic-cli/src/loop_engine/d2_integration.rs`
- **Line:** `27`

#### Current Status
warning: unused import: `LoopError`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0027
- **Category:** `unused_import`
- **File:** `semantic-cli/src/loop_engine/d2_integration.rs`
- **Line:** `32`

#### Current Status
warning: unused import: `LocalState`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0028
- **Category:** `unused_import`
- **File:** `semantic-cli/src/loop_engine/control.rs`
- **Line:** `526`

#### Current Status
warning: unused import: `LoopConfig`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0029
- **Category:** `unused_import`
- **File:** `semantic-cli/src/context.rs`
- **Line:** `34`

#### Current Status
warning: unused import: `Duration`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0030
- **Category:** `unused_import`
- **File:** `semantic-cli/src/context/registry.rs`
- **Line:** `8`

#### Current Status
warning: unused import: `serde_json::Value`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0031
- **Category:** `unused_import`
- **File:** `semantic-cli/src/context/registry.rs`
- **Line:** `10`

#### Current Status
warning: unused import: `std::sync::Arc`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0032
- **Category:** `unused_import`
- **File:** `semantic-cli/src/context/loaders.rs`
- **Line:** `7`

#### Current Status
warning: unused imports: `ErrorCode` and `SemanticCLIError`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0033
- **Category:** `unused_import`
- **File:** `semantic-cli/src/context/loaders.rs`
- **Line:** `8`

#### Current Status
warning: unused import: `Value`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0034
- **Category:** `unused_import`
- **File:** `semantic-cli/src/operations/query.rs`
- **Line:** `13`

#### Current Status
warning: unused import: `NormalizationError`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0035
- **Category:** `unused_import`
- **File:** `semantic-cli/src/operations/query.rs`
- **Line:** `14`

#### Current Status
warning: unused import: `IRBuildError`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0036
- **Category:** `unused_import`
- **File:** `semantic-cli/src/operations/query.rs`
- **Line:** `15`

#### Current Status
warning: unused import: `ExecutionError`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0037
- **Category:** `unused_import`
- **File:** `semantic-cli/src/gate_c/types.rs`
- **Line:** `261`

#### Current Status
warning: unused import: `crate::gate_c::deterministic::deterministic_hash_fnv1a`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0038
- **Category:** `unused_import`
- **File:** `semantic-cli/src/gate_c/ir/mod.rs`
- **Line:** `1038`

#### Current Status
warning: unused import: `InvalidationReason`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0039
- **Category:** `unused_import`
- **File:** `semantic-cli/src/gate_c/normalizer/mod.rs`
- **Line:** `961`

#### Current Status
warning: unused import: `InvalidationReason`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0040
- **Category:** `unused_import`
- **File:** `semantic-cli/src/gate_c/snapshot_tests.rs`
- **Line:** `337`

#### Current Status
warning: unused import: `super::*`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-deprecated-0001
- **Category:** `deprecated`
- **File:** `semantic-cli/src/validator.rs`
- **Line:** `388`

#### Current Status
warning: use of deprecated struct `validator::Validator`: Use BCIBValidator for Gate B functionality

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0001
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/normalizer/instruction_orderer.rs`
- **Line:** `123`

#### Current Status
warning: unused variable: `idx`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0002
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/normalizer/validator.rs`
- **Line:** `326`

#### Current Status
warning: unused variable: `idx`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0003
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/normalizer/mod.rs`
- **Line:** `147`

#### Current Status
warning: unused variable: `dependencies`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_mut-0001
- **Category:** `unused_mut`
- **File:** `semantic-cli/src/execution_plan/dataflow.rs`
- **Line:** `360`

#### Current Status
warning: variable does not need to be mutable

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0004
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/ir_planner/replay.rs`
- **Line:** `187`

#### Current Status
warning: unused variable: `register_id`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0005
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/ir_planner/mod.rs`
- **Line:** `174`

#### Current Status
warning: unused variable: `plan`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0006
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/parallelism/implementation/error.rs`
- **Line:** `244`

#### Current Status
warning: unused variable: `source`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0007
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/parallelism/implementation/verification.rs`
- **Line:** `325`

#### Current Status
warning: unused variable: `verification_overhead`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_mut-0002
- **Category:** `unused_mut`
- **File:** `semantic-cli/src/loop_engine/fingerprint.rs`
- **Line:** `1197`

#### Current Status
warning: variable does not need to be mutable

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_mut-0003
- **Category:** `unused_mut`
- **File:** `semantic-cli/src/operations/query.rs`
- **Line:** `353`

#### Current Status
warning: variable does not need to be mutable

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0008
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/gate_c/security_ops/mod.rs`
- **Line:** `540`

#### Current Status
warning: unused variable: `capability_requirements`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0009
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/gate_c/ir/mod.rs`
- **Line:** `1282`

#### Current Status
warning: unused variable: `parallel_hints`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0010
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/gate_c/normalizer/mod.rs`
- **Line:** `211`

#### Current Status
warning: unused variable: `intent`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0011
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/gate_c/snapshot_tests.rs`
- **Line:** `249`

#### Current Status
warning: unused variable: `config`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-dead_code-0001
- **Category:** `dead_code`
- **File:** `semantic-cli/src/execution_plan/mod.rs`
- **Line:** `480`

#### Current Status
warning: function `test_location` is never used

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-lifetime_syntax-0001
- **Category:** `lifetime_syntax`
- **File:** `semantic-cli/src/bcib.rs`
- **Line:** `1026`

#### Current Status
warning: hiding a lifetime that's elided elsewhere is confusing

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-lifetime_syntax-0002
- **Category:** `lifetime_syntax`
- **File:** `semantic-cli/src/parallelism/implementation/executor.rs`
- **Line:** `322`

#### Current Status
warning: hiding a lifetime that's elided elsewhere is confusing

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-meaningless_assertion-0001
- **Category:** `meaningless_assertion`
- **File:** `semantic-cli/src/loop_engine/tests/architecture_preservation_tests.rs`
- **Line:** `413`

#### Current Status
warning: comparison is useless due to type limits

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-meaningless_assertion-0002
- **Category:** `meaningless_assertion`
- **File:** `semantic-cli/src/loop_engine/tests/architecture_preservation_tests.rs`
- **Line:** `417`

#### Current Status
warning: comparison is useless due to type limits

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-meaningless_assertion-0003
- **Category:** `meaningless_assertion`
- **File:** `semantic-cli/src/loop_engine/tests/architecture_preservation_tests.rs`
- **Line:** `747`

#### Current Status
warning: comparison is useless due to type limits

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-meaningless_assertion-0004
- **Category:** `meaningless_assertion`
- **File:** `semantic-cli/src/loop_engine/tests/architecture_preservation_tests.rs`
- **Line:** `751`

#### Current Status
warning: comparison is useless due to type limits

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-meaningless_assertion-0005
- **Category:** `meaningless_assertion`
- **File:** `semantic-cli/src/loop_engine/tests/architecture_preservation_tests.rs`
- **Line:** `868`

#### Current Status
warning: comparison is useless due to type limits

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-meaningless_assertion-0006
- **Category:** `meaningless_assertion`
- **File:** `semantic-cli/src/loop_engine/tests/architecture_preservation_tests.rs`
- **Line:** `869`

#### Current Status
warning: comparison is useless due to type limits

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-meaningless_assertion-0007
- **Category:** `meaningless_assertion`
- **File:** `semantic-cli/src/loop_engine/tests/architecture_preservation_tests.rs`
- **Line:** `870`

#### Current Status
warning: comparison is useless due to type limits

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-meaningless_assertion-0008
- **Category:** `meaningless_assertion`
- **File:** `semantic-cli/src/loop_engine/tests/architecture_preservation_tests.rs`
- **Line:** `908`

#### Current Status
warning: comparison is useless due to type limits

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-meaningless_assertion-0009
- **Category:** `meaningless_assertion`
- **File:** `semantic-cli/src/loop_engine/tests/architecture_preservation_tests.rs`
- **Line:** `920`

#### Current Status
warning: comparison is useless due to type limits

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-meaningless_assertion-0010
- **Category:** `meaningless_assertion`
- **File:** `semantic-cli/src/loop_engine/tests/architecture_preservation_tests.rs`
- **Line:** `927`

#### Current Status
warning: comparison is useless due to type limits

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-meaningless_assertion-0011
- **Category:** `meaningless_assertion`
- **File:** `semantic-cli/src/loop_engine/tests/architecture_preservation_tests.rs`
- **Line:** `930`

#### Current Status
warning: comparison is useless due to type limits

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-meaningless_assertion-0012
- **Category:** `meaningless_assertion`
- **File:** `semantic-cli/src/loop_engine/tests/architecture_preservation_tests.rs`
- **Line:** `934`

#### Current Status
warning: comparison is useless due to type limits

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-meaningless_assertion-0013
- **Category:** `meaningless_assertion`
- **File:** `semantic-cli/src/gate_c/security_ops/mod.rs`
- **Line:** `1376`

#### Current Status
warning: comparison is useless due to type limits

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-meaningless_assertion-0014
- **Category:** `meaningless_assertion`
- **File:** `semantic-cli/src/gate_c/security_ops/mod.rs`
- **Line:** `1700`

#### Current Status
warning: comparison is useless due to type limits

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-meaningless_assertion-0015
- **Category:** `meaningless_assertion`
- **File:** `semantic-cli/src/gate_c/security_ops/mod.rs`
- **Line:** `2168`

#### Current Status
warning: comparison is useless due to type limits

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-meaningless_assertion-0016
- **Category:** `meaningless_assertion`
- **File:** `semantic-cli/src/gate_c/security_ops/mod.rs`
- **Line:** `2232`

#### Current Status
warning: comparison is useless due to type limits

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-meaningless_assertion-0017
- **Category:** `meaningless_assertion`
- **File:** `semantic-cli/src/gate_c/security_ops/mod.rs`
- **Line:** `2233`

#### Current Status
warning: comparison is useless due to type limits

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-meaningless_assertion-0018
- **Category:** `meaningless_assertion`
- **File:** `semantic-cli/src/gate_c/repl_visibility/mod.rs`
- **Line:** `1671`

#### Current Status
warning: comparison is useless due to type limits

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-meaningless_assertion-0019
- **Category:** `meaningless_assertion`
- **File:** `semantic-cli/src/gate_c/repl_visibility/mod.rs`
- **Line:** `1950`

#### Current Status
warning: comparison is useless due to type limits

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-must_use-0001
- **Category:** `must_use`
- **File:** `semantic-cli/src/repl/mod.rs`
- **Line:** `122`

#### Current Status
warning: unused `std::result::Result` that must be used

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0041
- **Category:** `unused_import`
- **File:** `semantic-cli/src/validator.rs`
- **Line:** `37`

#### Current Status
warning: unused imports: `ComparisonOp`, `LogicalOperator`, and `Value`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0042
- **Category:** `unused_import`
- **File:** `semantic-cli/src/validator.rs`
- **Line:** `45`

#### Current Status
warning: unused imports: `BinaryOp`, `Expr`, and `UnaryOp`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0043
- **Category:** `unused_import`
- **File:** `semantic-cli/src/transformer.rs`
- **Line:** `39`

#### Current Status
warning: unused imports: `Capability` and `SystemScope`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0044
- **Category:** `unused_import`
- **File:** `semantic-cli/src/normalizer/instruction_orderer.rs`
- **Line:** `9`

#### Current Status
warning: unused import: `OperandRef`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0045
- **Category:** `unused_import`
- **File:** `semantic-cli/src/normalizer/instruction_orderer.rs`
- **Line:** `12`

#### Current Status
warning: unused import: `HashSet`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0046
- **Category:** `unused_import`
- **File:** `semantic-cli/src/execution_plan/mod.rs`
- **Line:** `13`

#### Current Status
warning: unused import: `NormalizedInstruction`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0047
- **Category:** `unused_import`
- **File:** `semantic-cli/src/execution_plan/mod.rs`
- **Line:** `14`

#### Current Status
warning: unused imports: `BCIBInstruction`, `ContextInstruction`, and `QueryInstruction`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0048
- **Category:** `unused_import`
- **File:** `semantic-cli/src/execution_plan/mod.rs`
- **Line:** `15`

#### Current Status
warning: unused import: `std::collections::HashMap`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0049
- **Category:** `unused_import`
- **File:** `semantic-cli/src/execution_plan/builder.rs`
- **Line:** `12`

#### Current Status
warning: unused imports: `ExecutionPlanError` and `ParallelSafety`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0050
- **Category:** `unused_import`
- **File:** `semantic-cli/src/execution_plan/builder.rs`
- **Line:** `14`

#### Current Status
warning: unused import: `InstructionGroup`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0051
- **Category:** `unused_import`
- **File:** `semantic-cli/src/execution_plan/builder.rs`
- **Line:** `15`

#### Current Status
warning: unused import: `Value`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0052
- **Category:** `unused_import`
- **File:** `semantic-cli/src/execution_plan/validator.rs`
- **Line:** `10`

#### Current Status
warning: unused import: `DataflowGraph`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0053
- **Category:** `unused_import`
- **File:** `semantic-cli/src/execution_plan/validator.rs`
- **Line:** `11`

#### Current Status
warning: unused import: `HashMap`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0054
- **Category:** `unused_import`
- **File:** `semantic-cli/src/ir_planner/mod.rs`
- **Line:** `12`

#### Current Status
warning: unused import: `IRBlock`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0055
- **Category:** `unused_import`
- **File:** `semantic-cli/src/ir_planner/replay.rs`
- **Line:** `11`

#### Current Status
warning: unused import: `crate::execution_plan::IRInstruction`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0056
- **Category:** `unused_import`
- **File:** `semantic-cli/src/ir_planner/mod.rs`
- **Line:** `27`

#### Current Status
warning: unused import: `ReplayStep`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0057
- **Category:** `unused_import`
- **File:** `semantic-cli/src/parallelism/implementation/constitutional.rs`
- **Line:** `20`

#### Current Status
warning: unused import: `BlockId`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0058
- **Category:** `unused_import`
- **File:** `semantic-cli/src/parallelism/implementation/executor.rs`
- **Line:** `18`

#### Current Status
warning: unused import: `BlockId`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0059
- **Category:** `unused_import`
- **File:** `semantic-cli/src/parallelism/implementation/verification.rs`
- **Line:** `21`

#### Current Status
warning: unused import: `ParallelismError`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0060
- **Category:** `unused_import`
- **File:** `semantic-cli/src/validator.rs`
- **Line:** `698`

#### Current Status
warning: unused import: `LogicalOperator`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0061
- **Category:** `unused_import`
- **File:** `semantic-cli/src/loop_engine/d2_integration.rs`
- **Line:** `27`

#### Current Status
warning: unused import: `LoopError`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0062
- **Category:** `unused_import`
- **File:** `semantic-cli/src/loop_engine/d2_integration.rs`
- **Line:** `32`

#### Current Status
warning: unused import: `LocalState`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0063
- **Category:** `unused_import`
- **File:** `semantic-cli/src/loop_engine/tests/support.rs`
- **Line:** `15`

#### Current Status
warning: unused import: `LoopError`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0064
- **Category:** `unused_import`
- **File:** `semantic-cli/src/normalizer/instruction_orderer.rs`
- **Line:** `319`

#### Current Status
warning: unused import: `DependencyTracker`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0065
- **Category:** `unused_import`
- **File:** `semantic-cli/src/normalizer/validator.rs`
- **Line:** `377`

#### Current Status
warning: unused imports: `ComparisonOp` and `Value`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0066
- **Category:** `unused_import`
- **File:** `semantic-cli/src/execution_plan/builder.rs`
- **Line:** `12`

#### Current Status
warning: unused import: `ExecutionPlanError`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0067
- **Category:** `unused_import`
- **File:** `semantic-cli/src/context.rs`
- **Line:** `34`

#### Current Status
warning: unused imports: `Duration` and `Instant`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0068
- **Category:** `unused_import`
- **File:** `semantic-cli/src/context/registry.rs`
- **Line:** `8`

#### Current Status
warning: unused import: `serde_json::Value`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0069
- **Category:** `unused_import`
- **File:** `semantic-cli/src/context/registry.rs`
- **Line:** `10`

#### Current Status
warning: unused import: `std::sync::Arc`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0070
- **Category:** `unused_import`
- **File:** `semantic-cli/src/context/loaders.rs`
- **Line:** `7`

#### Current Status
warning: unused imports: `ErrorCode` and `SemanticCLIError`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0071
- **Category:** `unused_import`
- **File:** `semantic-cli/src/context/loaders.rs`
- **Line:** `8`

#### Current Status
warning: unused import: `Value`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0072
- **Category:** `unused_import`
- **File:** `semantic-cli/src/context/loaders.rs`
- **Line:** `9`

#### Current Status
warning: unused import: `Instant`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0073
- **Category:** `unused_import`
- **File:** `semantic-cli/src/operations/query.rs`
- **Line:** `13`

#### Current Status
warning: unused import: `NormalizationError`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0074
- **Category:** `unused_import`
- **File:** `semantic-cli/src/operations/query.rs`
- **Line:** `14`

#### Current Status
warning: unused import: `IRBuildError`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0075
- **Category:** `unused_import`
- **File:** `semantic-cli/src/operations/query.rs`
- **Line:** `15`

#### Current Status
warning: unused import: `ExecutionError`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0076
- **Category:** `unused_import`
- **File:** `semantic-cli/src/execution_plan/validator.rs`
- **Line:** `564`

#### Current Status
warning: unused import: `crate::bcib::Value`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0077
- **Category:** `unused_import`
- **File:** `semantic-cli/src/execution_plan/mod.rs`
- **Line:** `477`

#### Current Status
warning: unused imports: `InstructionGroup` and `NormalizedInstruction`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0078
- **Category:** `unused_import`
- **File:** `semantic-cli/src/ir_planner/replay.rs`
- **Line:** `413`

#### Current Status
warning: unused import: `crate::execution_plan::RegisterId`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0079
- **Category:** `unused_import`
- **File:** `semantic-cli/src/parallelism/implementation/error.rs`
- **Line:** `444`

#### Current Status
warning: unused import: `crate::execution_plan::BlockId`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0080
- **Category:** `unused_import`
- **File:** `semantic-cli/src/gate_c/types.rs`
- **Line:** `261`

#### Current Status
warning: unused import: `crate::gate_c::deterministic::deterministic_hash_fnv1a`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0081
- **Category:** `unused_import`
- **File:** `semantic-cli/src/gate_c/security_ops/mod.rs`
- **Line:** `14`

#### Current Status
warning: unused import: `DataRef`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0082
- **Category:** `unused_import`
- **File:** `semantic-cli/src/parallelism/implementation/tests/constitutional_compliance.rs`
- **Line:** `7`

#### Current Status
warning: unused import: `ParallelismResult`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0083
- **Category:** `unused_import`
- **File:** `semantic-cli/src/gate_c/ir/mod.rs`
- **Line:** `14`

#### Current Status
warning: unused import: `DataRef`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0084
- **Category:** `unused_import`
- **File:** `semantic-cli/src/gate_c/repl_visibility/mod.rs`
- **Line:** `14`

#### Current Status
warning: unused imports: `DataRef` and `Dependency`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0085
- **Category:** `unused_import`
- **File:** `semantic-cli/src/loop_engine/fingerprint.rs`
- **Line:** `1936`

#### Current Status
warning: unused import: `state::LoopState`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0086
- **Category:** `unused_import`
- **File:** `semantic-cli/src/loop_engine/control.rs`
- **Line:** `526`

#### Current Status
warning: unused import: `LoopConfig`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0087
- **Category:** `unused_import`
- **File:** `semantic-cli/src/context.rs`
- **Line:** `34`

#### Current Status
warning: unused import: `Duration`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0088
- **Category:** `unused_import`
- **File:** `semantic-cli/src/gate_c/ir/mod.rs`
- **Line:** `1038`

#### Current Status
warning: unused import: `InvalidationReason`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0089
- **Category:** `unused_import`
- **File:** `semantic-cli/src/gate_c/normalizer/mod.rs`
- **Line:** `961`

#### Current Status
warning: unused import: `InvalidationReason`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_import-0090
- **Category:** `unused_import`
- **File:** `semantic-cli/src/gate_c/snapshot_tests.rs`
- **Line:** `337`

#### Current Status
warning: unused import: `super::*`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-deprecated-0002
- **Category:** `deprecated`
- **File:** `semantic-cli/src/validator.rs`
- **Line:** `388`

#### Current Status
warning: use of deprecated struct `validator::Validator`: Use BCIBValidator for Gate B functionality

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0012
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/normalizer/instruction_orderer.rs`
- **Line:** `123`

#### Current Status
warning: unused variable: `idx`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0013
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/normalizer/validator.rs`
- **Line:** `326`

#### Current Status
warning: unused variable: `idx`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0014
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/normalizer/mod.rs`
- **Line:** `147`

#### Current Status
warning: unused variable: `dependencies`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0015
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/ir_planner/replay.rs`
- **Line:** `187`

#### Current Status
warning: unused variable: `register_id`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0016
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/ir_planner/mod.rs`
- **Line:** `174`

#### Current Status
warning: unused variable: `plan`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0017
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/parallelism/implementation/error.rs`
- **Line:** `244`

#### Current Status
warning: unused variable: `source`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0018
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/parallelism/implementation/verification.rs`
- **Line:** `325`

#### Current Status
warning: unused variable: `verification_overhead`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_mut-0004
- **Category:** `unused_mut`
- **File:** `semantic-cli/src/loop_engine/fingerprint.rs`
- **Line:** `1197`

#### Current Status
warning: variable does not need to be mutable

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_mut-0005
- **Category:** `unused_mut`
- **File:** `semantic-cli/src/operations/query.rs`
- **Line:** `353`

#### Current Status
warning: variable does not need to be mutable

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_mut-0006
- **Category:** `unused_mut`
- **File:** `semantic-cli/src/execution_plan/dataflow.rs`
- **Line:** `360`

#### Current Status
warning: variable does not need to be mutable

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0019
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/gate_c/security_ops/mod.rs`
- **Line:** `540`

#### Current Status
warning: unused variable: `capability_requirements`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0020
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/gate_c/normalizer/mod.rs`
- **Line:** `211`

#### Current Status
warning: unused variable: `intent`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_mut-0007
- **Category:** `unused_mut`
- **File:** `semantic-cli/src/loop_engine/fingerprint.rs`
- **Line:** `1197`

#### Current Status
warning: variable does not need to be mutable

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0021
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/gate_c/ir/mod.rs`
- **Line:** `1282`

#### Current Status
warning: unused variable: `parallel_hints`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

### INV-4.1-unused_variable-0022
- **Category:** `unused_variable`
- **File:** `semantic-cli/src/gate_c/snapshot_tests.rs`
- **Line:** `249`

#### Current Status
warning: unused variable: `config`

#### Technical Assessment
( ) Actually unnecessary  
( ) Will be used soon  
( ) Intentional placeholder  
( ) Architectural debt  

#### Decision
☐ Clean in Phase 4.1.1  
☐ Intentionally kept  
☐ Deferred to Phase 5  

#### Rationale
> (Required - cannot be left empty)

#### Action
- [ ] Delete code
- [ ] Ignore with `_`
- [ ] `#[allow(...)]` + explanation
- [ ] Documentation note

---

## Decision Rules (CONSTITUTIONAL)

These rules cannot be interpreted or bent.

### 🔴 IMMEDIATE CLEANUP
- Actually unused imports
- Meaningless assertions in test code
- Type-limit violations like `>= 0`

### 🟡 INTENTIONALLY KEPT
- Phase 5 parallelism / policy infrastructure
- Not yet active but architectural areas
- Areas waiting for replay / audit

📌 **REQUIREMENT:**
```rust
#[allow(dead_code)]
// Reserved for Phase 5 constitutional enforcement
```

### 🔵 DEFERRED TO PHASE 5
- Requiring architectural transformation
- Would cause API changes
- Behavioral risk

## Risk Register (MANDATORY)

| Risk | Description | Mitigation |
|------|-------------|------------|
| Wrong deletion | Critical area for future phase deleted | Inventory requirement |
| Silent behavior change | Semantics broken while fixing warning | Freeze + diff |
| Over-cleanup | Unnecessary "cleaning" | Phase boundaries |

## Exit Criteria (HOW THIS PHASE ENDS)

Phase 4.1.0 closes only when:
- ✅ All warnings in inventory
- ✅ Decision marked for each entry  
- ✅ Phase 4.1.1 cleanup list clear
- ✅ No commits made (except documentation)

📌 **Exit Tag:** `phase-4.1.0-inventory-complete`

## Strategic Note (IMPORTANT)

This document is the answer when someone asks "why didn't you delete this?" in the future.
Decisions are preserved, not code.

---

**Generated by:** `tools/ci/phase_4_1_inventory.py`  
**Constitutional Compliance:** Phase 4.1.0 Inventory Standard
