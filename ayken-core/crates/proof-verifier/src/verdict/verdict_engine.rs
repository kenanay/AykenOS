use crate::types::{
    VerdictSubject, VerificationAuditEvent, VerificationFinding, VerificationOutcome,
    VerificationReceipt, VerificationVerdict,
};

pub fn build_outcome(
    verdict: VerificationVerdict,
    subject: VerdictSubject,
    findings: Vec<VerificationFinding>,
    receipt: Option<VerificationReceipt>,
    audit_event: Option<VerificationAuditEvent>,
) -> VerificationOutcome {
    VerificationOutcome {
        verdict,
        subject,
        findings,
        receipt,
        audit_event,
    }
}
