use crate::types::VerdictSubject;

pub fn build_verdict_subject(
    bundle_id: &str,
    trust_overlay_hash: &str,
    policy_hash: &str,
    registry_snapshot_hash: &str,
) -> VerdictSubject {
    VerdictSubject {
        bundle_id: bundle_id.to_string(),
        trust_overlay_hash: trust_overlay_hash.to_string(),
        policy_hash: policy_hash.to_string(),
        registry_snapshot_hash: registry_snapshot_hash.to_string(),
    }
}
