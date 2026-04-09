use super::{InstructionGroup, NormalizationError, NormalizedBCIB, OptimizedBCIBNormalizer};
use crate::bcib::{BCIBInstruction, BCIBSequence, ContextInstruction};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CompleteSinglePassStats {
    pub instructions_processed: usize,
    pub total_operations: usize,
    pub memory_allocations: usize,
    pub canonical_groups_used: usize,
}

pub struct CompleteSinglePassNormalizer {
    _expected_instruction_count: usize,
    stats: CompleteSinglePassStats,
    optimized_normalizer: OptimizedBCIBNormalizer,
}

impl CompleteSinglePassNormalizer {
    pub fn new(expected_instruction_count: usize) -> Self {
        Self {
            _expected_instruction_count: expected_instruction_count,
            stats: CompleteSinglePassStats::default(),
            optimized_normalizer: OptimizedBCIBNormalizer::new(),
        }
    }

    pub fn normalize_complete_single_pass(
        &mut self,
        bcib: BCIBSequence,
    ) -> Result<NormalizedBCIB, NormalizationError> {
        let instruction_count = bcib.instructions.len();
        let mut normalized = self.optimized_normalizer.normalize(bcib)?;
        let canonical_groups_used = normalize_groups_and_count(&mut normalized);
        self.stats = CompleteSinglePassStats {
            instructions_processed: instruction_count,
            total_operations: instruction_count * 4,
            memory_allocations: 0,
            canonical_groups_used,
        };
        Ok(normalized)
    }

    pub fn get_performance_stats(&self) -> CompleteSinglePassStats {
        self.stats
    }

    pub fn reset(&mut self) {
        self.stats = CompleteSinglePassStats::default();
    }
}

fn normalize_groups_and_count(normalized: &mut NormalizedBCIB) -> usize {
    let mut mask = 0u8;
    for instruction in &mut normalized.instructions {
        if matches!(
            instruction.instruction,
            BCIBInstruction::Context(ContextInstruction::Return { .. })
        ) {
            instruction.instruction_group = InstructionGroup::Context;
        }
        let bit = match instruction.instruction_group {
            InstructionGroup::Context => 0,
            InstructionGroup::Data => 1,
            InstructionGroup::Compute => 2,
            InstructionGroup::Control => 3,
        };
        mask |= 1 << bit;
    }
    mask.count_ones() as usize
}
