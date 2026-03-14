mod support;

use gradle_dependency_check::analysis::diff_calculator;
use gradle_dependency_check::dependency::models::*;
use support::factories;

#[test]
fn identical_trees_all_unchanged() {
    let tree = factories::simple_tree();
    let result = diff_calculator::diff(&tree, &tree);
    assert!(result.added().is_empty());
    assert!(result.removed().is_empty());
    assert!(result.version_changed().is_empty());
    assert!(!result.unchanged().is_empty());
}

#[test]
fn added_dependency_detected() {
    let baseline = DependencyTree {
        project_name: "test".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![factories::node("com.a", "lib-a", "1.0")],
        conflicts: vec![],
    };
    let current = DependencyTree {
        project_name: "test".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![
            factories::node("com.a", "lib-a", "1.0"),
            factories::node("com.b", "lib-b", "2.0"),
        ],
        conflicts: vec![],
    };

    let result = diff_calculator::diff(&baseline, &current);
    assert_eq!(result.added().len(), 1);
    assert_eq!(result.added()[0].coordinate, "com.b:lib-b");
}

#[test]
fn removed_dependency_detected() {
    let baseline = DependencyTree {
        project_name: "test".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![
            factories::node("com.a", "lib-a", "1.0"),
            factories::node("com.b", "lib-b", "2.0"),
        ],
        conflicts: vec![],
    };
    let current = DependencyTree {
        project_name: "test".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![factories::node("com.a", "lib-a", "1.0")],
        conflicts: vec![],
    };

    let result = diff_calculator::diff(&baseline, &current);
    assert_eq!(result.removed().len(), 1);
    assert_eq!(result.removed()[0].coordinate, "com.b:lib-b");
}

#[test]
fn version_change_detected() {
    let baseline = DependencyTree {
        project_name: "test".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![factories::node("com.a", "lib-a", "1.0")],
        conflicts: vec![],
    };
    let current = DependencyTree {
        project_name: "test".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![factories::node("com.a", "lib-a", "2.0")],
        conflicts: vec![],
    };

    let result = diff_calculator::diff(&baseline, &current);
    assert_eq!(result.version_changed().len(), 1);
}

#[test]
fn entries_sorted_by_coordinate() {
    let baseline = DependencyTree {
        project_name: "test".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![
            factories::node("org.z", "z-lib", "1.0"),
            factories::node("com.a", "a-lib", "1.0"),
        ],
        conflicts: vec![],
    };

    let result = diff_calculator::diff(&baseline, &baseline);
    let coords: Vec<&str> = result.entries.iter().map(|e| e.coordinate.as_str()).collect();
    let mut sorted = coords.clone();
    sorted.sort();
    assert_eq!(coords, sorted);
}
