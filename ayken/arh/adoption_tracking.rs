// Constitutional Module: Adoption Tracking
// Tracks project-level adoption only (no identity profiling).

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdoptionMetrics {
    pub hints_shown: usize,
    pub hints_accepted: usize,
    pub hints_ignored: usize,
}

impl AdoptionMetrics {
    pub fn acceptance_rate(&self) -> f32 {
        if self.hints_shown == 0 {
            0.0
        } else {
            self.hints_accepted as f32 / self.hints_shown as f32
        }
    }
}
