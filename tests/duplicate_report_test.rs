mod support;

use std::collections::HashMap;

use gradle_dependency_check::dependency::models::*;
use gradle_dependency_check::report::duplicate_report;
use support::factories;

#[test]
fn no_duplicates_produces_no_duplicates_message() {
    let tree = factories::simple_tree();
    let results: Vec<DuplicateDependencyResult> = vec![];
    let report = duplicate_report::report(&results, &tree, ReportFormat::Text);
    assert!(report.contains("No duplicate dependencies found"));
    assert!(report.contains("test-project"));
}

#[test]
fn text_report_separates_cross_module_and_within_module() {
    let tree = factories::simple_tree();
    let results = vec![
        DuplicateDependencyResult {
            coordinate: "com.google.guava:guava".to_string(),
            kind: DuplicateKind::CrossModule,
            modules: vec!["app".to_string(), "core".to_string()],
            versions: {
                let mut m = HashMap::new();
                m.insert("app".to_string(), "31.1-jre".to_string());
                m.insert("core".to_string(), "30.0-jre".to_string());
                m
            },
            has_version_mismatch: true,
            recommendation: "Align versions across modules".to_string(),
        },
        DuplicateDependencyResult {
            coordinate: "org.slf4j:slf4j-api".to_string(),
            kind: DuplicateKind::WithinModule,
            modules: vec!["app".to_string()],
            versions: {
                let mut m = HashMap::new();
                m.insert("app".to_string(), "1.7.36".to_string());
                m
            },
            has_version_mismatch: false,
            recommendation: "Remove duplicate declaration".to_string(),
        },
    ];
    let report = duplicate_report::report(&results, &tree, ReportFormat::Text);
    assert!(report.contains("Cross-module duplicates:"));
    assert!(report.contains("Within-module duplicates:"));
}

#[test]
fn cross_module_shows_modules_and_version_mismatch() {
    let tree = factories::simple_tree();
    let results = vec![DuplicateDependencyResult {
        coordinate: "com.google.guava:guava".to_string(),
        kind: DuplicateKind::CrossModule,
        modules: vec!["app".to_string(), "core".to_string()],
        versions: {
            let mut m = HashMap::new();
            m.insert("app".to_string(), "31.1-jre".to_string());
            m.insert("core".to_string(), "30.0-jre".to_string());
            m
        },
        has_version_mismatch: true,
        recommendation: "Align versions".to_string(),
    }];
    let report = duplicate_report::report(&results, &tree, ReportFormat::Text);
    assert!(report.contains("app, core"));
    assert!(report.contains("[VERSION MISMATCH]"));
}

#[test]
fn json_report_contains_required_fields() {
    let tree = factories::simple_tree();
    let results = vec![DuplicateDependencyResult {
        coordinate: "com.google.guava:guava".to_string(),
        kind: DuplicateKind::CrossModule,
        modules: vec!["app".to_string(), "core".to_string()],
        versions: {
            let mut m = HashMap::new();
            m.insert("app".to_string(), "31.1-jre".to_string());
            m
        },
        has_version_mismatch: false,
        recommendation: "Align versions".to_string(),
    }];
    let report = duplicate_report::report(&results, &tree, ReportFormat::Json);
    let json: serde_json::Value = serde_json::from_str(&report).unwrap();
    assert_eq!(json["projectName"], "test-project");
    assert_eq!(json["configuration"], "compileClasspath");
    assert!(json["duplicateCount"].as_u64().unwrap() > 0);
    assert!(json["duplicates"].is_array());
    let dup = &json["duplicates"][0];
    assert_eq!(dup["coordinate"], "com.google.guava:guava");
    assert_eq!(dup["kind"], "crossModule");
    assert!(dup["modules"].is_array());
    assert!(dup["versions"].is_object());
    assert!(dup.get("hasVersionMismatch").is_some());
    assert!(dup["recommendation"].is_string());
}
