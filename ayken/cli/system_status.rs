// Constitutional Module: CLI System Status
// Read-only status reporting; must not influence decisions.

//! Render system health report for CLI output (read-only).

use crate::cde::health_monitor::{HealthFlag, HealthReport};
use crate::reporting::system_health::{
    ArhActivitySummary, CdeHealthSummary, SystemHealthReport, SystemRiskSummary, WaiverRefactorSummary,
};

pub fn render_cde_health(report: &HealthReport) -> String {
    let mut lines = Vec::new();
    lines.push("CDE Health Report".to_string());
    lines.push(format!("Phase: {}", report.snapshot.phase));
    lines.push(format!("Entropy: {:.3}", report.snapshot.entropy));
    lines.push(format!(
        "Waiver ratio: {:.2}",
        report.snapshot.waiver_ratio
    ));
    lines.push(format!("Active rules: {}", report.snapshot.active_rules));

    if report.flags.is_empty() {
        lines.push("Flags: none".to_string());
    } else {
        lines.push("Flags:".to_string());
        lines.extend(format_flags(&report.flags));
    }

    if !report.messages.is_empty() {
        lines.push("Notes:".to_string());
        lines.extend(report.messages.iter().map(|m| format!("- {}", m)));
    }

    lines.join("\n")
}

fn format_flags(flags: &[HealthFlag]) -> Vec<String> {
    flags.iter().map(|f| format!("- {}", f.message)).collect()
}

pub fn render_system_status(report: &SystemHealthReport) -> String {
    let mut lines = Vec::new();
    lines.push("=== Ayken System Status ===".to_string());
    lines.extend(render_cde_section(report.cde.as_ref()));
    lines.extend(render_arh_section(report.arh.as_ref()));
    lines.extend(render_waiver_section(report.waivers.as_ref()));
    lines.extend(render_risk_section(&report.risks));
    lines.join("\n")
}

fn render_cde_section(summary: Option<&CdeHealthSummary>) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push("CDE Health".to_string());
    match summary {
        None => lines.push("- unavailable".to_string()),
        Some(cde) => {
            lines.push(format!("- Phase: {} ({})", cde.phase, cde.phase_alignment));
            lines.push(format!("- Entropy: {:.3}", cde.entropy));
            lines.push(format!(
                "- Outcome balance: FAIL={} ALLOW={} WAIVER={} REFACTOR={}",
                cde.outcome_distribution
                    .get(&crate::cde::decision_metrics::DecisionOutcome::Fail)
                    .unwrap_or(&0),
                cde.outcome_distribution
                    .get(&crate::cde::decision_metrics::DecisionOutcome::Allow)
                    .unwrap_or(&0),
                cde.outcome_distribution
                    .get(&crate::cde::decision_metrics::DecisionOutcome::Waiver)
                    .unwrap_or(&0),
                cde.outcome_distribution
                    .get(&crate::cde::decision_metrics::DecisionOutcome::Refactor)
                    .unwrap_or(&0),
            ));
            if cde.flags.is_empty() {
                lines.push("- Flags: none".to_string());
            } else {
                lines.push(format!("- Flags: {:?}", cde.flags));
            }
        }
    }
    lines
}

fn render_arh_section(summary: Option<&ArhActivitySummary>) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push("ARH / ARRE Activity".to_string());
    match summary {
        None => lines.push("- unavailable".to_string()),
        Some(arh) => {
            lines.push(format!("- Recent refactors: {}", arh.recent_refactors));
            lines.push(format!(
                "- Effectiveness: +{} / ~{} / -{}",
                arh.effectiveness_distribution
                    .counts
                    .get(&crate::arh::refactor_outcome::Effectiveness::Positive)
                    .unwrap_or(&0),
                arh.effectiveness_distribution
                    .counts
                    .get(&crate::arh::refactor_outcome::Effectiveness::Neutral)
                    .unwrap_or(&0),
                arh.effectiveness_distribution
                    .counts
                    .get(&crate::arh::refactor_outcome::Effectiveness::Negative)
                    .unwrap_or(&0),
            ));
            lines.push(format!(
                "- Confidence adjustments: {}",
                arh.confidence_adjustment_count
            ));
        }
    }
    lines
}

fn render_waiver_section(summary: Option<&WaiverRefactorSummary>) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push("Waivers & Refactors".to_string());
    match summary {
        None => lines.push("- unavailable".to_string()),
        Some(w) => {
            lines.push(format!("- Open waivers: {}", w.open_waivers));
            lines.push(format!(
                "- Oldest waiver age (days): {}",
                w.oldest_waiver_days.map(|d| d.to_string()).unwrap_or_else(|| "unavailable".to_string())
            ));
            lines.push(format!("- Aging refactors: {}", w.aging_refactors));
            lines.push(format!("- Repeated waivers: {}", w.repeated_waivers));
        }
    }
    lines
}

fn render_risk_section(summary: &SystemRiskSummary) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push("Systemic Risks".to_string());
    if summary.dead_controls == 0
        && !summary.over_waivering
        && !summary.rule_inflation
        && summary.governance_flags.is_empty()
    {
        lines.push("- No systemic risks detected".to_string());
        return lines;
    }
    lines.push(format!("- Dead controls: {}", summary.dead_controls));
    lines.push(format!(
        "- Over-waivering: {}",
        if summary.over_waivering { "yes" } else { "no" }
    ));
    lines.push(format!(
        "- Rule inflation: {}",
        if summary.rule_inflation { "yes" } else { "no" }
    ));
    if !summary.governance_flags.is_empty() {
        lines.push(format!("- Governance flags: {:?}", summary.governance_flags));
    }
    lines
}
