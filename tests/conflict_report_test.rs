mod support;

use gradle_dependency_check::dependency::models::*;
use gradle_dependency_check::report::conflict_report;
use support::factories;

#[test]
fn no_conflicts_produces_no_conflicts_message() {
    let tree = factories::simple_tree();
    let result = conflict_report::report(&tree, ReportFormat::Text);
    assert!(result.contains("No dependency conflicts found"));
    assert!(result.contains("test-project"));
}

#[test]
fn text_report_contains_conflict_details() {
    let tree = tree_with_conflicts();
    let result = conflict_report::report(&tree, ReportFormat::Text);
    assert!(result.contains("com.google.guava:guava"));
    assert!(result.contains("30.0-jre"));
    assert!(result.contains("31.1-jre"));
    assert!(result.contains("requested by"));
    assert!(result.contains("spring-core"));
}

#[test]
fn json_report_contains_required_fields() {
    let tree = tree_with_conflicts();
    let result = conflict_report::report(&tree, ReportFormat::Json);
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(json["projectName"], "test-project");
    assert_eq!(json["configuration"], "compileClasspath");
    assert!(json["conflictCount"].as_u64().unwrap() > 0);
    assert!(json["conflicts"].is_array());
    let conflict = &json["conflicts"][0];
    assert!(conflict["coordinate"].is_string());
    assert!(conflict["requestedVersion"].is_string());
    assert!(conflict["resolvedVersion"].is_string());
    assert!(conflict["requestedBy"].is_string());
}

#[test]
fn json_report_no_conflicts_has_zero_count() {
    let tree = factories::simple_tree();
    let result = conflict_report::report(&tree, ReportFormat::Json);
    let json: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(json["conflictCount"], 0);
    assert_eq!(json["conflicts"].as_array().unwrap().len(), 0);
}

fn tree_with_conflicts() -> DependencyTree {
    let mut guava = DependencyNode::new("com.google.guava", "guava", "30.0-jre");
    guava.resolved_version = Some("31.1-jre".to_string());

    let mut root = DependencyNode::new("org.springframework", "spring-core", "5.3.20");
    root.children = vec![guava];

    DependencyTree {
        project_name: "test-project".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![root],
        conflicts: vec![DependencyConflict {
            coordinate: "com.google.guava:guava".to_string(),
            requested_version: "30.0-jre".to_string(),
            resolved_version: "31.1-jre".to_string(),
            requested_by: "spring-core".to_string(),
        }],
    }
}
