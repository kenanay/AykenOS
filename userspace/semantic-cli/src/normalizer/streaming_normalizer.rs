use super::{InstructionGroup, NormalizationError, NormalizedBCIB, OptimizedBCIBNormalizer};
use crate::bcib::{BCIBInstruction, BCIBSequence, ContextInstruction};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StreamingPerformanceStats {
    pub instructions_processed: usize,
    pub current_instruction_index: usize,
    pub buffer_length: usize,
}

pub struct StreamingNormalizer {
    _expected_instruction_count: usize,
    stats: StreamingPerformanceStats,
    optimized_normalizer: OptimizedBCIBNormalizer,
}

impl StreamingNormalizer {
    pub fn new(expected_instruction_count: usize) -> Self {
        Self {
            _expected_instruction_count: expected_instruction_count,
            stats: StreamingPerformanceStats::default(),
            optimized_normalizer: OptimizedBCIBNormalizer::new(),
        }
    }

    pub fn normalize_streaming(
        &mut self,
        bcib: BCIBSequence,
    ) -> Result<NormalizedBCIB, NormalizationError> {
        let instruction_count = bcib.instructions.len();
        let mut normalized = self.optimized_normalizer.normalize(bcib)?;
        for instruction in &mut normalized.instructions {
            if matches!(
                instruction.instruction,
                BCIBInstruction::Context(ContextInstruction::Return { .. })
            ) {
                instruction.instruction_group = InstructionGroup::Context;
            }
        }
        self.stats = StreamingPerformanceStats {
            instructions_processed: instruction_count,
            current_instruction_index: instruction_count,
            buffer_length: normalized.instructions.len(),
        };
        Ok(normalized)
    }

    pub fn get_performance_stats(&self) -> StreamingPerformanceStats {
        self.stats
    }

    pub fn canonical_groups_used(&self, normalized: &NormalizedBCIB) -> usize {
        let mut mask = 0u8;
        for instruction in &normalized.instructions {
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
}
