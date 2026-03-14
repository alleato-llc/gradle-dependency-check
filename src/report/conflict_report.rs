use std::collections::BTreeMap;

use crate::dependency::models::{DependencyTree, ReportFormat};

pub fn report(tree: &DependencyTree, format: ReportFormat) -> String {
    match format {
        ReportFormat::Text => text_report(tree),
        ReportFormat::Json => json_report(tree),
    }
}

fn text_report(tree: &DependencyTree) -> String {
    if tree.conflicts.is_empty() {
        return format!(
            "No dependency conflicts found in {} ({}).",
            tree.project_name,
            tree.configuration.display_name()
        );
    }

    let mut lines = Vec::new();
    lines.push(format!(
        "Dependency Conflicts in {} ({})",
        tree.project_name,
        tree.configuration.display_name()
    ));
    lines.push("=".repeat(60));
    lines.push(String::new());

    let mut grouped: BTreeMap<&str, Vec<_>> = BTreeMap::new();
    for conflict in &tree.conflicts {
        grouped
            .entry(&conflict.coordinate)
            .or_default()
            .push(conflict);
    }

    for (coordinate, conflicts) in &grouped {
        lines.push(format!("  {}", coordinate));
        for conflict in conflicts {
            let risk_suffix = match &conflict.risk_level {
                Some(level) => format!(" [{}]", level),
                None => String::new(),
            };
            lines.push(format!(
                "    {} -> {} (requested by {}){}",
                conflict.requested_version, conflict.resolved_version, conflict.requested_by, risk_suffix
            ));
            if let Some(reason) = &conflict.risk_reason {
                lines.push(format!("    risk: {}", reason));
            }
        }
        lines.push(String::new());
    }

    lines.push(format!(
        "Total: {} conflict(s) across {} dependency(ies)",
        tree.conflicts.len(),
        grouped.len()
    ));
    lines.join("\n")
}

fn json_report(tree: &DependencyTree) -> String {
    let report = serde_json::json!({
        "projectName": tree.project_name,
        "configuration": tree.configuration.as_str(),
        "conflictCount": tree.conflicts.len(),
        "conflicts": tree.conflicts.iter().map(|c| {
            let mut map = serde_json::Map::new();
            map.insert("coordinate".to_string(), serde_json::json!(c.coordinate));
            map.insert("requestedVersion".to_string(), serde_json::json!(c.requested_version));
            map.insert("resolvedVersion".to_string(), serde_json::json!(c.resolved_version));
            map.insert("requestedBy".to_string(), serde_json::json!(c.requested_by));
            if let Some(level) = &c.risk_level {
                map.insert("riskLevel".to_string(), serde_json::json!(level));
            }
            if let Some(reason) = &c.risk_reason {
                map.insert("riskReason".to_string(), serde_json::json!(reason));
            }
            serde_json::Value::Object(map)
        }).collect::<Vec<_>>(),
    });

    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
}
