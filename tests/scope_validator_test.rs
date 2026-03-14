mod support;

use gradle_dependency_check::analysis::scope_validator;
use gradle_dependency_check::dependency::models::*;
use support::factories;

#[test]
fn production_config_with_test_libraries_returns_results() {
    let tree = factories::tree_with_test_libraries();
    let results = scope_validator::validate(&tree);
    assert!(!results.is_empty());
    assert_eq!(results.len(), 3);
}

#[test]
fn test_config_returns_empty() {
    let mut tree = factories::tree_with_test_libraries();
    tree.configuration = GradleConfiguration::TestCompileClasspath;
    let results = scope_validator::validate(&tree);
    assert!(results.is_empty());
}

#[test]
fn no_test_libraries_returns_empty() {
    let tree = factories::simple_tree();
    let results = scope_validator::validate(&tree);
    assert!(results.is_empty());
}

#[test]
fn results_sorted_by_coordinate() {
    let tree = factories::tree_with_test_libraries();
    let results = scope_validator::validate(&tree);
    let coords: Vec<&str> = results.iter().map(|r| r.coordinate.as_str()).collect();
    let mut sorted = coords.clone();
    sorted.sort();
    assert_eq!(coords, sorted);
}
