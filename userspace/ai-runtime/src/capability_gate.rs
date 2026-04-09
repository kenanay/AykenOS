use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AiCapability {
    Suggest,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AiCapabilitySet {
    granted: BTreeSet<AiCapability>,
}

impl AiCapabilitySet {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn suggestion_only() -> Self {
        let mut granted = BTreeSet::new();
        granted.insert(AiCapability::Suggest);
        Self { granted }
    }

    pub fn grant(&mut self, capability: AiCapability) {
        self.granted.insert(capability);
    }

    pub fn contains(&self, capability: AiCapability) -> bool {
        self.granted.contains(&capability)
    }

    pub fn allow_ai(&self) -> bool {
        self.contains(AiCapability::Suggest)
    }
}

#[cfg(test)]
mod tests {
    use super::{AiCapability, AiCapabilitySet};

    #[test]
    fn suggestion_only_grants_ai_access() {
        assert!(AiCapabilitySet::suggestion_only().allow_ai());
    }

    #[test]
    fn empty_set_denies_ai_access() {
        assert!(!AiCapabilitySet::none().allow_ai());
    }

    #[test]
    fn grant_adds_capability() {
        let mut caps = AiCapabilitySet::none();
        caps.grant(AiCapability::Suggest);
        assert!(caps.contains(AiCapability::Suggest));
    }
}
