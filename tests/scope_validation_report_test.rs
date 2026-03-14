mod support;

use gradle_dependency_check::dependency::models::*;
use gradle_dependency_check::report::scope_validation_report;
use support::factories;

#[test]
fn no_issues_produces_no_scope_issues_message() {
    let tree = factories::simple_tree();
    let results: Vec<ScopeValidationResult> = vec![];
    let report = scope_validation_report::report(&results, &tree, ReportFormat::Text);
    assert!(report.contains("No scope issues found"));
    assert!(report.contains("test-project"));
}

#[test]
fn text_report_contains_coordinate_and_library_and_recommendation() {
    let tree = factories::simple_tree();
    let results = vec![ScopeValidationResult {
        coordinate: "junit:junit".to_string(),
        version: "4.13.2".to_string(),
        matched_library: "JUnit 4".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        recommendation: "Move to testImplementation".to_string(),
    }];
    let report = scope_validation_report::report(&results, &tree, ReportFormat::Text);
    assert!(report.contains("junit:junit"));
    assert!(report.contains("JUnit 4"));
    assert!(report.contains("Move to testImplementation"));
}

#[test]
fn json_report_contains_required_fields() {
    let tree = factories::simple_tree();
    let results = vec![ScopeValidationResult {
        coordinate: "junit:junit".to_string(),
        version: "4.13.2".to_string(),
        matched_library: "JUnit 4".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        recommendation: "Move to testImplementation".to_string(),
    }];
    let report = scope_validation_report::report(&results, &tree, ReportFormat::Json);
    let json: serde_json::Value = serde_json::from_str(&report).unwrap();
    assert_eq!(json["projectName"], "test-project");
    assert_eq!(json["configuration"], "compileClasspath");
    assert!(json["issueCount"].as_u64().unwrap() > 0);
    assert!(json["issues"].is_array());
    let issue = &json["issues"][0];
    assert_eq!(issue["coordinate"], "junit:junit");
    assert_eq!(issue["matchedLibrary"], "JUnit 4");
    assert_eq!(issue["recommendation"], "Move to testImplementation");
}
