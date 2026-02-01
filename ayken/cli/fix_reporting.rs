// Constitutional Module: Fix Reporting
// This module MUST NOT mutate code.
// Outputs are advisory-only reports.

use std::collections::BTreeMap;

use crate::arh::arh_engine::ArhOutput;
use crate::arh::fix_mapping::HintType;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixReportEntry {
    pub rule_id: String,
    pub total: usize,
    pub assisted: usize,
    pub design: usize,
    pub kernel_blocked: usize,
    pub avg_confidence: u8,
    pub max_risk: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixReport {
    pub entries: Vec<FixReportEntry>,
}

pub fn generate_report(outputs: &[(String, ArhOutput, bool)]) -> FixReport {
    let mut map: BTreeMap<String, FixReportEntry> = BTreeMap::new();

    for (rule_id, output, is_kernel) in outputs {
        let entry = map.entry(rule_id.clone()).or_insert(FixReportEntry {
            rule_id: rule_id.clone(),
            total: 0,
            assisted: 0,
            design: 0,
            kernel_blocked: 0,
            avg_confidence: 0,
            max_risk: 0,
        });

        let mut confidence_sum = 0u32;
        let mut max_risk = entry.max_risk;
        for hint in &output.ranked_hints {
            entry.total += 1;
            match hint.hint_type {
                HintType::AssistedFix => entry.assisted += 1,
                HintType::DesignHint => entry.design += 1,
            }
            confidence_sum += hint.confidence_score as u32;
            if hint.risk_score > max_risk {
                max_risk = hint.risk_score;
            }
            if *is_kernel {
                entry.kernel_blocked += 1;
            }
        }
        if entry.total > 0 {
            entry.avg_confidence = (confidence_sum / entry.total as u32) as u8;
        }
        entry.max_risk = max_risk;
    }

    FixReport {
        entries: map.into_values().collect(),
    }
}

pub fn report_to_text(report: &FixReport) -> String {
    let mut lines = Vec::new();
    for entry in &report.entries {
        lines.push(format!("Rule: {}", entry.rule_id));
        lines.push(format!("  Fixes available: {}", entry.total));
        lines.push(format!("  Assisted: {}", entry.assisted));
        lines.push(format!("  Design: {}", entry.design));
        lines.push(format!("  Kernel-blocked: {}", entry.kernel_blocked));
        lines.push(format!("  Avg confidence: {}%", entry.avg_confidence));
        lines.push(format!("  Max risk: {}", entry.max_risk));
    }
    lines.join("\n")
}
