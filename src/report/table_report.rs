use crate::dependency::models::{DependencyTree, FlatDependencyEntry, ReportFormat};

pub fn report(entries: &[FlatDependencyEntry], tree: &DependencyTree, format: ReportFormat) -> String {
    match format {
        ReportFormat::Text => text_report(entries, tree),
        ReportFormat::Json => json_report(entries, tree),
    }
}

fn text_report(entries: &[FlatDependencyEntry], tree: &DependencyTree) -> String {
    if entries.is_empty() {
        return format!(
            "No dependencies found in {} ({}).",
            tree.project_name,
            tree.configuration.display_name()
        );
    }

    let mut lines = Vec::new();
    lines.push(format!(
        "Dependencies in {} ({})",
        tree.project_name,
        tree.configuration.display_name()
    ));
    lines.push("=".repeat(60));
    lines.push(String::new());

    for entry in entries {
        let conflict = if entry.has_conflict { " [CONFLICT]" } else { "" };
        let versions = if entry.versions.len() > 1 {
            let mut sorted: Vec<_> = entry.versions.iter().collect();
            sorted.sort();
            format!(" (versions: {})", sorted.into_iter().cloned().collect::<Vec<_>>().join(", "))
        } else {
            String::new()
        };
        lines.push(format!("  {}:{}{}{}", entry.coordinate, entry.version, conflict, versions));
        if !entry.used_by.is_empty() {
            lines.push(format!("    used by: {}", entry.used_by.join(", ")));
        }
    }

    lines.push(String::new());
    lines.push(format!("Total: {} unique dependency(ies)", entries.len()));
    lines.join("\n")
}

fn json_report(entries: &[FlatDependencyEntry], tree: &DependencyTree) -> String {
    let json_entries: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| {
            let mut sorted_versions: Vec<_> = e.versions.iter().cloned().collect();
            sorted_versions.sort();
            serde_json::json!({
                "coordinate": e.coordinate,
                "group": e.group,
                "artifact": e.artifact,
                "version": e.version,
                "hasConflict": e.has_conflict,
                "occurrenceCount": e.occurrence_count,
                "usedBy": e.used_by,
                "versions": sorted_versions,
            })
        })
        .collect();

    let report = serde_json::json!({
        "projectName": tree.project_name,
        "configuration": tree.configuration.as_str(),
        "dependencyCount": entries.len(),
        "dependencies": json_entries,
    });

    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
}
