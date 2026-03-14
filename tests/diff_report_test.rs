mod support;

use gradle_dependency_check::dependency::models::*;
use gradle_dependency_check::report::diff_report;

#[test]
fn no_differences_produces_no_differences_message() {
    let result = DependencyDiffResult {
        baseline_name: "v1".to_string(),
        current_name: "v2".to_string(),
        entries: vec![],
    };
    let report = diff_report::report(&result.entries, &result, ReportFormat::Text);
    assert!(report.contains("No differences found"));
    assert!(report.contains("v1"));
    assert!(report.contains("v2"));
}

#[test]
fn text_report_uses_plus_minus_tilde_equals_symbols() {
    let entries = vec![
        DependencyDiffEntry {
            coordinate: "com.new:lib-new".to_string(),
            change_kind: ChangeKind::Added,
            before_version: None,
            after_version: Some("1.0".to_string()),
            before_resolved_version: None,
            after_resolved_version: None,
        },
        DependencyDiffEntry {
            coordinate: "com.old:lib-old".to_string(),
            change_kind: ChangeKind::Removed,
            before_version: Some("1.0".to_string()),
            after_version: None,
            before_resolved_version: None,
            after_resolved_version: None,
        },
        DependencyDiffEntry {
            coordinate: "com.changed:lib-changed".to_string(),
            change_kind: ChangeKind::VersionChanged,
            before_version: Some("1.0".to_string()),
            after_version: Some("2.0".to_string()),
            before_resolved_version: None,
            after_resolved_version: None,
        },
        DependencyDiffEntry {
            coordinate: "com.same:lib-same".to_string(),
            change_kind: ChangeKind::Unchanged,
            before_version: Some("1.0".to_string()),
            after_version: Some("1.0".to_string()),
            before_resolved_version: None,
            after_resolved_version: None,
        },
    ];
    let result = DependencyDiffResult {
        baseline_name: "v1".to_string(),
        current_name: "v2".to_string(),
        entries: entries.clone(),
    };
    let report = diff_report::report(&entries, &result, ReportFormat::Text);
    assert!(report.contains("+ com.new:lib-new"));
    assert!(report.contains("- com.old:lib-old"));
    assert!(report.contains("~ com.changed:lib-changed"));
    assert!(report.contains("= com.same:lib-same"));
}

#[test]
fn json_report_contains_required_fields() {
    let entries = vec![
        DependencyDiffEntry {
            coordinate: "com.new:lib-new".to_string(),
            change_kind: ChangeKind::Added,
            before_version: None,
            after_version: Some("1.0".to_string()),
            before_resolved_version: None,
            after_resolved_version: None,
        },
        DependencyDiffEntry {
            coordinate: "com.same:lib-same".to_string(),
            change_kind: ChangeKind::Unchanged,
            before_version: Some("1.0".to_string()),
            after_version: Some("1.0".to_string()),
            before_resolved_version: None,
            after_resolved_version: None,
        },
    ];
    let result = DependencyDiffResult {
        baseline_name: "baseline-proj".to_string(),
        current_name: "current-proj".to_string(),
        entries: entries.clone(),
    };
    let report = diff_report::report(&entries, &result, ReportFormat::Json);
    let json: serde_json::Value = serde_json::from_str(&report).unwrap();
    assert_eq!(json["baseline"], "baseline-proj");
    assert_eq!(json["current"], "current-proj");
    assert!(json["added"].is_number());
    assert!(json["removed"].is_number());
    assert!(json["changed"].is_number());
    assert!(json["unchanged"].is_number());
    assert!(json["entries"].is_array());
    assert_eq!(json["added"].as_u64().unwrap(), 1);
    assert_eq!(json["unchanged"].as_u64().unwrap(), 1);
}
