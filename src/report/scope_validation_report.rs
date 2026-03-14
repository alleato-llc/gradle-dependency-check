use crate::dependency::models::{DependencyTree, ReportFormat, ScopeValidationResult};

pub fn report(results: &[ScopeValidationResult], tree: &DependencyTree, format: ReportFormat) -> String {
    match format {
        ReportFormat::Text => text_report(results, tree),
        ReportFormat::Json => json_report(results, tree),
    }
}

fn text_report(results: &[ScopeValidationResult], tree: &DependencyTree) -> String {
    if results.is_empty() {
        return format!(
            "No scope issues found in {} ({}).",
            tree.project_name,
            tree.configuration.display_name()
        );
    }

    let mut lines = Vec::new();
    lines.push(format!(
        "Scope Validation Issues in {} ({})",
        tree.project_name,
        tree.configuration.display_name()
    ));
    lines.push("=".repeat(60));
    lines.push(String::new());

    for result in results {
        lines.push(format!("  {}:{}", result.coordinate, result.version));
        lines.push(format!("    detected as: {}", result.matched_library));
        lines.push(format!("    recommendation: {}", result.recommendation));
        lines.push(String::new());
    }

    lines.push(format!("Total: {} issue(s)", results.len()));
    lines.join("\n")
}

fn json_report(results: &[ScopeValidationResult], tree: &DependencyTree) -> String {
    let json_results: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "coordinate": r.coordinate,
                "version": r.version,
                "matchedLibrary": r.matched_library,
                "configuration": r.configuration.as_str(),
                "recommendation": r.recommendation,
            })
        })
        .collect();

    let report = serde_json::json!({
        "projectName": tree.project_name,
        "configuration": tree.configuration.as_str(),
        "issueCount": results.len(),
        "issues": json_results,
    });

    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
}
