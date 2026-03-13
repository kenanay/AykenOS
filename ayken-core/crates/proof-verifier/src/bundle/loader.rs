use crate::types::LoadedBundle;
use std::path::Path;

pub fn load_bundle(root: &Path) -> LoadedBundle {
    LoadedBundle {
        root: root.to_path_buf(),
        manifest_path: root.join("manifest.json"),
        checksums_path: root.join("checksums.json"),
        evidence_dir: root.join("evidence"),
        traces_dir: root.join("traces"),
        reports_dir: root.join("reports"),
        meta_run_path: root.join("meta").join("run.json"),
        producer_path: root.join("producer").join("producer.json"),
        signature_envelope_path: root.join("signatures").join("signature-envelope.json"),
    }
}
