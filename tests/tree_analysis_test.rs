mod support;

use gradle_dependency_check::analysis::tree_analysis;
use support::factories;

#[test]
fn all_nodes_collects_all_nodes() {
    let tree = factories::simple_tree();
    let nodes = tree_analysis::all_nodes(&tree);
    assert_eq!(nodes.len(), 3);
}

#[test]
fn unique_coordinates_returns_set() {
    let tree = factories::simple_tree();
    let coordinates = tree_analysis::unique_coordinates(&tree);
    assert_eq!(coordinates.len(), 3);
    assert!(coordinates.contains("org.springframework:spring-core"));
    assert!(coordinates.contains("com.google.guava:guava"));
    assert!(coordinates.contains("org.slf4j:slf4j-api"));
}

#[test]
fn subtree_sizes_returns_correct_sizes() {
    let tree = factories::simple_tree();
    let sizes = tree_analysis::subtree_sizes(&tree);
    assert_eq!(sizes["org.springframework:spring-core"], 3);
    assert_eq!(sizes["com.google.guava:guava"], 1);
    assert_eq!(sizes["org.slf4j:slf4j-api"], 1);
}

#[test]
fn conflicts_by_coordinate_groups_correctly() {
    let tree = factories::tree_with_conflicts();
    let grouped = tree_analysis::conflicts_by_coordinate(&tree);
    assert_eq!(grouped.len(), 1);
    assert_eq!(
        grouped["com.fasterxml.jackson.core:jackson-databind"].len(),
        1
    );
}

#[test]
fn conflicts_by_coordinate_empty_for_no_conflicts() {
    let tree = factories::simple_tree();
    let grouped = tree_analysis::conflicts_by_coordinate(&tree);
    assert!(grouped.is_empty());
}
