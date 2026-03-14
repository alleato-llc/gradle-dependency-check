mod support;

use gradle_dependency_check::analysis::duplicate_detector;
use gradle_dependency_check::dependency::models::*;
use support::factories;

#[test]
fn cross_module_shared_dep() {
    let tree = factories::multi_module_tree(false);
    let results = duplicate_detector::detect_cross_module(&tree);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].coordinate, "com.google.guava:guava");
    assert_eq!(results[0].kind, DuplicateKind::CrossModule);
    assert_eq!(results[0].modules.len(), 2);
}

#[test]
fn cross_module_unique_deps_returns_empty() {
    let mut module_a = DependencyNode::new("test-project", "app", "module");
    module_a.children = vec![factories::node("com.a", "lib-a", "1.0")];
    let mut module_b = DependencyNode::new("test-project", "core", "module");
    module_b.children = vec![factories::node("com.b", "lib-b", "1.0")];

    let tree = DependencyTree {
        project_name: "test".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![module_a, module_b],
        conflicts: vec![],
    };

    let results = duplicate_detector::detect_cross_module(&tree);
    assert!(results.is_empty());
}

#[test]
fn cross_module_version_mismatch() {
    let tree = factories::multi_module_tree(true);
    let results = duplicate_detector::detect_cross_module(&tree);
    assert_eq!(results.len(), 1);
    assert!(results[0].has_version_mismatch);
    assert!(results[0].recommendation.contains("mismatch"));
}

#[test]
fn single_module_returns_no_cross_module() {
    let tree = factories::simple_tree();
    let results = duplicate_detector::detect_cross_module(&tree);
    assert!(results.is_empty());
}

#[test]
fn within_module_duplicate_declaration() {
    let dir = tempfile::tempdir().unwrap();
    let build_content = "\
dependencies {
    implementation 'com.google.guava:guava:31.1-jre'
    testImplementation 'com.google.guava:guava:31.1-jre'
}";
    std::fs::write(dir.path().join("build.gradle"), build_content).unwrap();

    let results = duplicate_detector::detect_within_module(
        dir.path().to_str().unwrap(),
        &[],
    );
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].kind, DuplicateKind::WithinModule);
    assert_eq!(results[0].coordinate, "com.google.guava:guava");
}

#[test]
fn within_module_no_duplicates() {
    let dir = tempfile::tempdir().unwrap();
    let build_content = "\
dependencies {
    implementation 'com.google.guava:guava:31.1-jre'
    implementation 'org.slf4j:slf4j-api:1.7.36'
}";
    std::fs::write(dir.path().join("build.gradle"), build_content).unwrap();

    let results = duplicate_detector::detect_within_module(
        dir.path().to_str().unwrap(),
        &[],
    );
    assert!(results.is_empty());
}

#[test]
fn results_sorted_by_coordinate() {
    let mut module_a = DependencyNode::new("test-project", "app", "module");
    module_a.children = vec![
        factories::node("org.z", "z-lib", "1.0"),
        factories::node("com.a", "a-lib", "1.0"),
    ];
    let mut module_b = DependencyNode::new("test-project", "core", "module");
    module_b.children = vec![
        factories::node("org.z", "z-lib", "1.0"),
        factories::node("com.a", "a-lib", "1.0"),
    ];

    let tree = DependencyTree {
        project_name: "test".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![module_a, module_b],
        conflicts: vec![],
    };

    let results = duplicate_detector::detect_cross_module(&tree);
    let coords: Vec<&str> = results.iter().map(|r| r.coordinate.as_str()).collect();
    let mut sorted = coords.clone();
    sorted.sort();
    assert_eq!(coords, sorted);
}
