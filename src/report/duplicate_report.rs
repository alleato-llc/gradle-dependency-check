use crate::dependency::models::{
    DependencyTree, DuplicateDependencyResult, DuplicateKind, ReportFormat,
};

pub fn report(results: &[DuplicateDependencyResult], tree: &DependencyTree, format: ReportFormat) -> String {
    match format {
        ReportFormat::Text => text_report(results, tree),
        ReportFormat::Json => json_report(results, tree),
    }
}

fn text_report(results: &[DuplicateDependencyResult], tree: &DependencyTree) -> String {
    if results.is_empty() {
        return format!(
            "No duplicate dependencies found in {} ({}).",
            tree.project_name,
            tree.configuration.display_name()
        );
    }

    let mut lines = Vec::new();
    lines.push(format!(
        "Duplicate Dependencies in {} ({})",
        tree.project_name,
        tree.configuration.display_name()
    ));
    lines.push("=".repeat(60));
    lines.push(String::new());

    let cross_module: Vec<_> = results.iter().filter(|r| r.kind == DuplicateKind::CrossModule).collect();
    let within_module: Vec<_> = results.iter().filter(|r| r.kind == DuplicateKind::WithinModule).collect();

    if !cross_module.is_empty() {
        lines.push("Cross-module duplicates:".to_string());
        lines.push(String::new());
        for result in &cross_module {
            let mismatch = if result.has_version_mismatch { " [VERSION MISMATCH]" } else { "" };
            lines.push(format!("  {}{}", result.coordinate, mismatch));
            lines.push(format!("    modules: {}", result.modules.join(", ")));
            let mut sorted_versions: Vec<_> = result.versions.iter().collect();
            sorted_versions.sort_by_key(|(k, _)| (*k).clone());
            for (module, version) in &sorted_versions {
                lines.push(format!("    {}: {}", module, version));
            }
            lines.push(format!("    recommendation: {}", result.recommendation));
            lines.push(String::new());
        }
    }

    if !within_module.is_empty() {
        lines.push("Within-module duplicates:".to_string());
        lines.push(String::new());
        for result in &within_module {
            lines.push(format!("  {}", result.coordinate));
            lines.push(format!("    {}", result.recommendation));
            lines.push(String::new());
        }
    }

    lines.push(format!(
        "Total: {} duplicate(s) ({} cross-module, {} within-module)",
        results.len(),
        cross_module.len(),
        within_module.len()
    ));
    lines.join("\n")
}

fn json_report(results: &[DuplicateDependencyResult], tree: &DependencyTree) -> String {
    let json_results: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            let kind = match r.kind {
                DuplicateKind::CrossModule => "crossModule",
                DuplicateKind::WithinModule => "withinModule",
            };
            serde_json::json!({
                "coordinate": r.coordinate,
                "kind": kind,
                "modules": r.modules,
                "versions": r.versions,
                "hasVersionMismatch": r.has_version_mismatch,
                "recommendation": r.recommendation,
            })
        })
        .collect();

    let report = serde_json::json!({
        "projectName": tree.project_name,
        "configuration": tree.configuration.as_str(),
        "duplicateCount": results.len(),
        "duplicates": json_results,
    });

    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
}
