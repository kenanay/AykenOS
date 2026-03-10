use crate::types::{LoadedBundle, VerificationFinding};

pub fn validate_bundle_layout(bundle: &LoadedBundle) -> Vec<VerificationFinding> {
    let required_paths = [
        (&bundle.manifest_path, "manifest.json"),
        (&bundle.checksums_path, "checksums.json"),
        (&bundle.evidence_dir, "evidence/"),
        (&bundle.traces_dir, "traces/"),
        (&bundle.reports_dir, "reports/"),
        (&bundle.meta_run_path, "meta/run.json"),
        (&bundle.producer_path, "producer/producer.json"),
        (
            &bundle.signature_envelope_path,
            "signatures/signature-envelope.json",
        ),
    ];

    let mut findings = Vec::new();
    for (path, label) in required_paths {
        if !path.exists() {
            findings.push(VerificationFinding::error(
                "PV0100",
                format!("required bundle path missing: {label}"),
            ));
        }
    }
    findings
}
