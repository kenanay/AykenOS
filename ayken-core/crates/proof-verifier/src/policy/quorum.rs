pub fn quorum_satisfied(required_count: usize, accepted_count: usize) -> bool {
    accepted_count >= required_count
}
