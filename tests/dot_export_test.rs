mod support;

use gradle_dependency_check::dependency::models::*;
use gradle_dependency_check::report::dot_export;
use support::factories;

#[test]
fn output_starts_with_digraph() {
    let tree = factories::simple_tree();
    let result = dot_export::export(&tree);
    assert!(result.starts_with("digraph dependencies {"));
}

#[test]
fn contains_node_labels_with_coordinate_and_version() {
    let tree = factories::simple_tree();
    let result = dot_export::export(&tree);
    assert!(result.contains("org.springframework:spring-core"));
    assert!(result.contains("5.3.20"));
    assert!(result.contains("com.google.guava:guava"));
    assert!(result.contains("31.1-jre"));
}

#[test]
fn conflict_nodes_have_red_fillcolor() {
    let tree = tree_with_conflict_node();
    let result = dot_export::export(&tree);
    assert!(result.contains("#ffcccc"));
}

#[test]
fn normal_nodes_have_green_fillcolor() {
    let tree = factories::simple_tree();
    let result = dot_export::export(&tree);
    assert!(result.contains("#e8f4e8"));
}

fn tree_with_conflict_node() -> DependencyTree {
    let mut guava = DependencyNode::new("com.google.guava", "guava", "30.0-jre");
    guava.resolved_version = Some("31.1-jre".to_string());

    let mut root = DependencyNode::new("org.springframework", "spring-core", "5.3.20");
    root.children = vec![guava];

    DependencyTree {
        project_name: "test-project".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![root],
        conflicts: vec![],
    }
}
