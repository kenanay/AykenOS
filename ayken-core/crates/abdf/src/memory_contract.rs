//! ABDF Memory Contract (Phase-16)
//!
//! This module defines memory layout guarantees for ABDF buffers.
//! These guarantees ensure compatibility with:
//! - BCIB execution (Phase-16)
//! - GPU execution (Phase-17+)
//! - Zero-copy operations
//!
//! # Alignment Guarantees
//!
//! - **Header**: 8-byte aligned
//! - **Segment Table**: 8-byte aligned
//! - **Meta Table**: 8-byte aligned
//! - **String Pool**: 8-byte aligned
//! - **Data Section**: 8-byte aligned (minimum)
//!
//! Future phases may require stricter alignment (16/32/64 bytes for GPU).

use std::mem;

/// Memory alignment requirements for ABDF structures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlignmentContract {
    /// Minimum alignment for all ABDF sections (bytes).
    pub min_alignment: usize,
    /// Whether strict alignment validation is enforced.
    pub strict_validation: bool,
}

impl Default for AlignmentContract {
    fn default() -> Self {
        Self::phase16()
    }
}

impl AlignmentContract {
    /// Phase-16 alignment contract (BCIB execution).
    ///
    /// Minimum 8-byte alignment for all sections.
    pub const fn phase16() -> Self {
        Self {
            min_alignment: 8,
            strict_validation: true,
        }
    }

    /// Phase-17+ alignment contract (GPU execution).
    ///
    /// Stricter alignment for GPU-friendly layouts.
    /// This is a placeholder for future implementation.
    pub const fn phase17_gpu() -> Self {
        Self {
            min_alignment: 64,
            strict_validation: true,
        }
    }

    /// Validate that a pointer/offset meets alignment requirements.
    pub fn validate_alignment(&self, offset: usize) -> Result<(), AlignmentError> {
        if !self.strict_validation {
            return Ok(());
        }

        if offset % self.min_alignment != 0 {
            return Err(AlignmentError::Misaligned {
                offset,
                required: self.min_alignment,
                actual: offset % self.min_alignment,
            });
        }

        Ok(())
    }

    /// Align an offset up to the required alignment.
    pub fn align_up(&self, offset: usize) -> usize {
        (offset + self.min_alignment - 1) & !(self.min_alignment - 1)
    }

    /// Check if a size is properly aligned.
    pub fn is_aligned(&self, offset: usize) -> bool {
        offset % self.min_alignment == 0
    }
}

/// Memory contract validation errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlignmentError {
    /// Offset is not properly aligned.
    Misaligned {
        offset: usize,
        required: usize,
        actual: usize,
    },
    /// Structure size violates alignment contract.
    InvalidStructSize {
        structure: &'static str,
        size: usize,
        required: usize,
    },
}

impl std::fmt::Display for AlignmentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlignmentError::Misaligned {
                offset,
                required,
                actual,
            } => write!(
                f,
                "Misaligned offset: {} (required: {}-byte alignment, actual: {} mod {})",
                offset, required, actual, required
            ),
            AlignmentError::InvalidStructSize {
                structure,
                size,
                required,
            } => write!(
                f,
                "Invalid structure size: {} has size {} (required: multiple of {})",
                structure, size, required
            ),
        }
    }
}

impl std::error::Error for AlignmentError {}

/// Validate that ABDF core structures meet alignment requirements.
pub fn validate_structure_alignment(contract: &AlignmentContract) -> Result<(), AlignmentError> {
    use crate::header::AbdfHeader;
    use crate::segment::SegmentDescriptor;

    // Validate AbdfHeader size
    let header_size = mem::size_of::<AbdfHeader>();
    if header_size % contract.min_alignment != 0 {
        return Err(AlignmentError::InvalidStructSize {
            structure: "AbdfHeader",
            size: header_size,
            required: contract.min_alignment,
        });
    }

    // Validate SegmentDescriptor size
    let seg_size = mem::size_of::<SegmentDescriptor>();
    if seg_size % contract.min_alignment != 0 {
        return Err(AlignmentError::InvalidStructSize {
            structure: "SegmentDescriptor",
            size: seg_size,
            required: contract.min_alignment,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase16_alignment() {
        let contract = AlignmentContract::phase16();
        assert_eq!(contract.min_alignment, 8);
        assert!(contract.strict_validation);
    }

    #[test]
    fn test_alignment_validation() {
        let contract = AlignmentContract::phase16();

        // Valid alignments
        assert!(contract.validate_alignment(0).is_ok());
        assert!(contract.validate_alignment(8).is_ok());
        assert!(contract.validate_alignment(16).is_ok());
        assert!(contract.validate_alignment(24).is_ok());

        // Invalid alignments
        assert!(contract.validate_alignment(1).is_err());
        assert!(contract.validate_alignment(7).is_err());
        assert!(contract.validate_alignment(9).is_err());
        assert!(contract.validate_alignment(15).is_err());
    }

    #[test]
    fn test_align_up() {
        let contract = AlignmentContract::phase16();

        assert_eq!(contract.align_up(0), 0);
        assert_eq!(contract.align_up(1), 8);
        assert_eq!(contract.align_up(7), 8);
        assert_eq!(contract.align_up(8), 8);
        assert_eq!(contract.align_up(9), 16);
        assert_eq!(contract.align_up(15), 16);
        assert_eq!(contract.align_up(16), 16);
    }

    #[test]
    fn test_is_aligned() {
        let contract = AlignmentContract::phase16();

        assert!(contract.is_aligned(0));
        assert!(!contract.is_aligned(1));
        assert!(!contract.is_aligned(7));
        assert!(contract.is_aligned(8));
        assert!(!contract.is_aligned(9));
        assert!(contract.is_aligned(16));
    }

    #[test]
    fn test_structure_alignment() {
        let contract = AlignmentContract::phase16();
        // This should pass if structures are properly aligned
        validate_structure_alignment(&contract).expect("Structure alignment validation failed");
    }

    #[test]
    fn test_phase17_gpu_alignment() {
        let contract = AlignmentContract::phase17_gpu();
        assert_eq!(contract.min_alignment, 64);

        assert!(contract.validate_alignment(0).is_ok());
        assert!(contract.validate_alignment(64).is_ok());
        assert!(contract.validate_alignment(128).is_ok());

        assert!(contract.validate_alignment(8).is_err());
        assert!(contract.validate_alignment(16).is_err());
        assert!(contract.validate_alignment(32).is_err());
    }
}
