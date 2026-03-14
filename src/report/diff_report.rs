use crate::dependency::models::{
    ChangeKind, DependencyDiffEntry, DependencyDiffResult, ReportFormat,
};

pub fn report(entries: &[DependencyDiffEntry], result: &DependencyDiffResult, format: ReportFormat) -> String {
    match format {
        ReportFormat::Text => text_report(entries, result),
        ReportFormat::Json => json_report(entries, result),
    }
}

fn text_report(entries: &[DependencyDiffEntry], result: &DependencyDiffResult) -> String {
    if entries.is_empty() {
        return format!(
            "No differences found between {} and {}.",
            result.baseline_name, result.current_name
        );
    }

    let mut lines = Vec::new();
    lines.push(format!(
        "Dependency Diff: {} → {}",
        result.baseline_name, result.current_name
    ));
    lines.push("=".repeat(60));
    lines.push(String::new());

    let summary_parts: Vec<String> = [
        (result.added().len(), "added"),
        (result.removed().len(), "removed"),
        (result.version_changed().len(), "changed"),
        (result.unchanged().len(), "unchanged"),
    ]
    .iter()
    .filter(|(count, _)| *count > 0)
    .map(|(count, label)| format!("{} {}", count, label))
    .collect();

    lines.push(format!("Summary: {}", summary_parts.join(", ")));
    lines.push(String::new());

    let mut sorted = entries.to_vec();
    sorted.sort_by(|a, b| a.coordinate.cmp(&b.coordinate));

    for entry in &sorted {
        let before = entry.effective_before_version().unwrap_or("-");
        let after = entry.effective_after_version().unwrap_or("-");

        match entry.change_kind {
            ChangeKind::Added => lines.push(format!("  + {}:{}", entry.coordinate, after)),
            ChangeKind::Removed => lines.push(format!("  - {}:{}", entry.coordinate, before)),
            ChangeKind::VersionChanged => {
                lines.push(format!("  ~ {}: {} → {}", entry.coordinate, before, after))
            }
            ChangeKind::Unchanged => {
                lines.push(format!("  = {}:{}", entry.coordinate, before))
            }
        }
    }

    lines.join("\n")
}

fn json_report(entries: &[DependencyDiffEntry], result: &DependencyDiffResult) -> String {
    let mut sorted = entries.to_vec();
    sorted.sort_by(|a, b| a.coordinate.cmp(&b.coordinate));

    let json_entries: Vec<serde_json::Value> = sorted
        .iter()
        .map(|e| {
            let mut obj = serde_json::json!({
                "coordinate": e.coordinate,
                "changeKind": e.change_kind.as_str(),
            });
            if let Some(before) = e.effective_before_version() {
                obj["beforeVersion"] = serde_json::Value::String(before.to_string());
            }
            if let Some(after) = e.effective_after_version() {
                obj["afterVersion"] = serde_json::Value::String(after.to_string());
            }
            obj
        })
        .collect();

    let report = serde_json::json!({
        "baseline": result.baseline_name,
        "current": result.current_name,
        "added": result.added().len(),
        "removed": result.removed().len(),
        "changed": result.version_changed().len(),
        "unchanged": result.unchanged().len(),
        "entries": json_entries,
    });

    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
}
