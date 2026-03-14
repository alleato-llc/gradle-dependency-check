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
            lines.push(format!(
                "    {} -> {} (requested by {})",
                conflict.requested_version, conflict.resolved_version, conflict.requested_by
            ));
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
    let conflicts: Vec<serde_json::Value> = tree
        .conflicts
        .iter()
        .map(|c| {
            serde_json::json!({
                "coordinate": c.coordinate,
                "requestedVersion": c.requested_version,
                "resolvedVersion": c.resolved_version,
                "requestedBy": c.requested_by,
            })
        })
        .collect();

    let report = serde_json::json!({
        "projectName": tree.project_name,
        "configuration": tree.configuration.as_str(),
        "conflictCount": tree.conflicts.len(),
        "conflicts": conflicts,
    });

    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
}
