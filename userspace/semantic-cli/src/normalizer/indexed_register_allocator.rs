use super::dependency_tracker::RegisterId;
use super::register_allocator::{RegisterAllocationError, RegisterAllocator};
use super::RegisterAllocation;
use crate::bcib::BCIBSequence;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct IndexedRegisterAllocation {
    pub allocated_registers: Vec<RegisterId>,
    pub register_dependencies: HashMap<RegisterId, Vec<RegisterId>>,
    pub next_register: u16,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IndexedRegisterAllocationStats {
    pub total_registers: usize,
    pub capacity: usize,
}

pub struct IndexedRegisterAllocator {
    inner: RegisterAllocator,
    stats: IndexedRegisterAllocationStats,
}

impl IndexedRegisterAllocator {
    pub fn new() -> Self {
        Self {
            inner: RegisterAllocator::new(),
            stats: IndexedRegisterAllocationStats::default(),
        }
    }

    pub fn allocate_for_sequence(
        &mut self,
        bcib: &BCIBSequence,
    ) -> Result<IndexedRegisterAllocation, RegisterAllocationError> {
        let allocation = self.inner.allocate_for_sequence(bcib)?;
        self.stats = IndexedRegisterAllocationStats {
            total_registers: allocation.allocated_registers.len(),
            capacity: bcib
                .instructions
                .len()
                .max(allocation.allocated_registers.len()),
        };
        Ok(IndexedRegisterAllocation {
            allocated_registers: allocation.allocated_registers,
            register_dependencies: allocation.register_dependencies,
            next_register: allocation.next_register,
        })
    }

    pub fn get_stats(&self) -> IndexedRegisterAllocationStats {
        self.stats
    }
}

impl Default for IndexedRegisterAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl From<IndexedRegisterAllocation> for RegisterAllocation {
    fn from(value: IndexedRegisterAllocation) -> Self {
        RegisterAllocation {
            allocated_registers: value.allocated_registers,
            register_dependencies: value.register_dependencies,
            next_register: value.next_register,
        }
    }
}
