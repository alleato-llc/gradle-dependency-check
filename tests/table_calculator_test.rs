mod support;

use gradle_dependency_check::analysis::table_calculator;
use gradle_dependency_check::dependency::models::*;
use support::factories;

#[test]
fn flat_entries_from_simple_tree() {
    let tree = factories::simple_tree();
    let entries = table_calculator::flat_entries(&tree);
    assert_eq!(entries.len(), 3);
    let guava = entries.iter().find(|e| e.artifact == "guava");
    assert!(guava.is_some());
    let guava = guava.unwrap();
    assert_eq!(guava.version, "31.1-jre");
    assert_eq!(guava.occurrence_count, 1);
}

#[test]
fn flat_entries_from_tree_with_conflicts() {
    let tree = factories::tree_with_conflicts();
    let entries = table_calculator::flat_entries(&tree);
    let jackson = entries
        .iter()
        .find(|e| e.coordinate == "com.fasterxml.jackson.core:jackson-databind");
    assert!(jackson.is_some());
    assert!(jackson.unwrap().has_conflict);
}

#[test]
fn flat_entries_used_by() {
    let tree = factories::simple_tree();
    let entries = table_calculator::flat_entries(&tree);
    let guava = entries
        .iter()
        .find(|e| e.coordinate == "com.google.guava:guava");
    assert!(guava.is_some());
    assert!(guava
        .unwrap()
        .used_by
        .contains(&"org.springframework:spring-core".to_string()));
}

#[test]
fn flat_entries_from_empty_tree() {
    let tree = DependencyTree {
        project_name: "empty".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![],
        conflicts: vec![],
    };
    let entries = table_calculator::flat_entries(&tree);
    assert!(entries.is_empty());
}

#[test]
fn parent_map_from_tree() {
    let tree = factories::simple_tree();
    let parents = table_calculator::parent_map(&tree);
    assert!(parents
        .get("com.google.guava:guava")
        .unwrap()
        .contains("org.springframework:spring-core"));
    assert!(parents
        .get("org.slf4j:slf4j-api")
        .unwrap()
        .contains("org.springframework:spring-core"));
    assert!(parents.get("org.springframework:spring-core").is_none());
}

#[test]
fn flat_entries_version_aggregation() {
    let tree = factories::tree_with_conflicts();
    let entries = table_calculator::flat_entries(&tree);
    let jackson = entries
        .iter()
        .find(|e| e.coordinate == "com.fasterxml.jackson.core:jackson-databind");
    assert!(jackson.is_some());
    let jackson = jackson.unwrap();
    assert!(jackson.versions.contains("2.14.2"));
}

#[test]
fn flat_entries_sorted_by_coordinate() {
    let tree = factories::simple_tree();
    let entries = table_calculator::flat_entries(&tree);
    let coordinates: Vec<&str> = entries.iter().map(|e| e.coordinate.as_str()).collect();
    let mut sorted = coordinates.clone();
    sorted.sort();
    assert_eq!(coordinates, sorted);
}

#[test]
fn flat_entries_count_matches_unique_coordinates() {
    let tree = factories::simple_tree();
    let entries = table_calculator::flat_entries(&tree);
    let mut coords: Vec<&str> = entries.iter().map(|e| e.coordinate.as_str()).collect();
    coords.sort();
    coords.dedup();
    assert_eq!(entries.len(), coords.len());
}
