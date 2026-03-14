mod support;

use gradle_dependency_check::dependency::models::*;
use gradle_dependency_check::report::tree_export;
use support::factories;

// MARK: - JSON export

#[test]
fn json_export_produces_valid_json() {
    let tree = factories::simple_tree();
    let json_str = tree_export::export_json(&tree).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed.is_object());
}

#[test]
fn json_export_contains_required_fields() {
    let tree = factories::simple_tree();
    let json_str = tree_export::export_json(&tree).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["projectName"], "test-project");
    assert!(parsed["configuration"].is_string());
    assert!(parsed["roots"].is_array());
    assert!(parsed["conflicts"].is_array());
}

#[test]
fn json_export_encodes_resolved_version() {
    let mut node = DependencyNode::new("com.example", "lib", "1.0");
    node.resolved_version = Some("2.0".to_string());
    let tree = DependencyTree {
        project_name: "test".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![node],
        conflicts: vec![],
    };
    let json_str = tree_export::export_json(&tree).unwrap();
    assert!(json_str.contains("resolvedVersion"));
    assert!(json_str.contains("2.0"));
}

#[test]
fn json_export_encodes_omitted_flag() {
    let mut node = DependencyNode::new("com.example", "lib", "1.0");
    node.is_omitted = true;
    let tree = DependencyTree {
        project_name: "test".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![node],
        conflicts: vec![],
    };
    let json_str = tree_export::export_json(&tree).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["roots"][0]["isOmitted"], true);
}

#[test]
fn json_export_encodes_constraint_flag() {
    let mut node = DependencyNode::new("com.example", "lib", "1.0");
    node.is_constraint = true;
    let tree = DependencyTree {
        project_name: "test".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![node],
        conflicts: vec![],
    };
    let json_str = tree_export::export_json(&tree).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["roots"][0]["isConstraint"], true);
}

#[test]
fn json_export_empty_tree_produces_valid_json() {
    let tree = DependencyTree {
        project_name: "empty".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![],
        conflicts: vec![],
    };
    let json_str = tree_export::export_json(&tree).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["roots"].as_array().unwrap().len(), 0);
}

// MARK: - JSON import

#[test]
fn json_import_roundtrip() {
    let tree = factories::simple_tree();
    let json_str = tree_export::export_json(&tree).unwrap();
    let imported = tree_export::import_tree(
        json_str.as_bytes(),
        "test.json",
        GradleConfiguration::CompileClasspath,
    )
    .unwrap();
    assert_eq!(imported.project_name, tree.project_name);
    assert_eq!(imported.total_node_count(), tree.total_node_count());
}

#[test]
fn json_import_invalid_json_returns_error() {
    let result = tree_export::import_tree(
        b"not valid json {{{",
        "test.json",
        GradleConfiguration::CompileClasspath,
    );
    assert!(result.is_err());
}

#[test]
fn json_import_missing_fields_returns_error() {
    let result = tree_export::import_tree(
        b"{\"foo\": \"bar\"}",
        "test.json",
        GradleConfiguration::CompileClasspath,
    );
    // JSON with missing required fields won't parse as DependencyTree,
    // and won't parse as Gradle text either, so it should error
    assert!(result.is_err());
}

// MARK: - Text export

#[test]
fn text_export_single_root_uses_backslash_connector() {
    let tree = DependencyTree {
        project_name: "test".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![factories::node("com.example", "lib", "1.0")],
        conflicts: vec![],
    };
    let text = tree_export::export_text(&tree);
    assert!(text.starts_with("\\---"));
}

#[test]
fn text_export_multiple_roots_use_plus_and_backslash() {
    let tree = DependencyTree {
        project_name: "test".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![
            factories::node("com.a", "lib-a", "1.0"),
            factories::node("com.b", "lib-b", "2.0"),
        ],
        conflicts: vec![],
    };
    let text = tree_export::export_text(&tree);
    assert!(text.contains("+---"));
    assert!(text.contains("\\---"));
}

#[test]
fn text_export_omitted_nodes_have_star() {
    let mut node = DependencyNode::new("com.example", "lib", "1.0");
    node.is_omitted = true;
    let tree = DependencyTree {
        project_name: "test".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![node],
        conflicts: vec![],
    };
    let text = tree_export::export_text(&tree);
    assert!(text.contains("(*)"));
}

#[test]
fn text_export_constraint_nodes_have_c() {
    let mut node = DependencyNode::new("com.example", "lib", "1.0");
    node.is_constraint = true;
    let tree = DependencyTree {
        project_name: "test".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![node],
        conflicts: vec![],
    };
    let text = tree_export::export_text(&tree);
    assert!(text.contains("(c)"));
}

#[test]
fn text_export_nested_children_indented_with_pipe() {
    let child = factories::node("com.child", "child-lib", "1.0");
    let grandchild = factories::node("com.grandchild", "gc-lib", "2.0");
    let mut mid = DependencyNode::new("com.mid", "mid-lib", "1.5");
    mid.children = vec![grandchild];

    let mut root = DependencyNode::new("com.root", "root-lib", "3.0");
    root.children = vec![mid, child];

    let tree = DependencyTree {
        project_name: "test".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![root],
        conflicts: vec![],
    };
    let text = tree_export::export_text(&tree);
    assert!(text.contains("|    "));
}

// MARK: - Tree importer (auto-detect)

#[test]
fn import_json_data_as_json() {
    let tree = factories::simple_tree();
    let json_str = tree_export::export_json(&tree).unwrap();
    let imported = tree_export::import_tree(
        json_str.as_bytes(),
        "deps.json",
        GradleConfiguration::CompileClasspath,
    )
    .unwrap();
    assert_eq!(imported.project_name, "test-project");
}

#[test]
fn import_gradle_text_data_as_text() {
    let text = "+--- com.google.guava:guava:31.1-jre\n\\--- org.slf4j:slf4j-api:1.7.36\n";
    let imported = tree_export::import_tree(
        text.as_bytes(),
        "my-app.txt",
        GradleConfiguration::CompileClasspath,
    )
    .unwrap();
    assert!(imported.total_node_count() >= 2);
}

#[test]
fn import_empty_data_returns_error() {
    let result = tree_export::import_tree(
        b"",
        "empty.txt",
        GradleConfiguration::CompileClasspath,
    );
    assert!(result.is_err());
}

#[test]
fn import_filename_suffix_stripping_dependencies() {
    let text = "\\--- com.example:lib:1.0\n";
    let imported = tree_export::import_tree(
        text.as_bytes(),
        "my-app-dependencies.txt",
        GradleConfiguration::CompileClasspath,
    )
    .unwrap();
    assert_eq!(imported.project_name, "my-app");
}

#[test]
fn import_filename_suffix_stripping_compile_classpath() {
    let text = "\\--- com.example:lib:1.0\n";
    let imported = tree_export::import_tree(
        text.as_bytes(),
        "my-app-compileClasspath.txt",
        GradleConfiguration::CompileClasspath,
    )
    .unwrap();
    assert_eq!(imported.project_name, "my-app");
}

// MARK: - Round-trip

#[test]
fn json_roundtrip_preserves_tree_structure() {
    let tree = tree_with_conflicts_for_roundtrip();
    let json_str = tree_export::export_json(&tree).unwrap();
    let imported = tree_export::import_tree(
        json_str.as_bytes(),
        "test.json",
        GradleConfiguration::CompileClasspath,
    )
    .unwrap();
    assert_eq!(imported.total_node_count(), tree.total_node_count());
    assert_eq!(imported.project_name, tree.project_name);
    assert_eq!(imported.conflicts.len(), tree.conflicts.len());
}

#[test]
fn text_export_parse_preserves_root_count() {
    let tree = DependencyTree {
        project_name: "test".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![
            factories::node("com.a", "lib-a", "1.0"),
            factories::node("com.b", "lib-b", "2.0"),
        ],
        conflicts: vec![],
    };
    let text = tree_export::export_text(&tree);
    let reimported = tree_export::import_tree(
        text.as_bytes(),
        "test.txt",
        GradleConfiguration::CompileClasspath,
    )
    .unwrap();
    assert_eq!(reimported.roots.len(), tree.roots.len());
}

fn tree_with_conflicts_for_roundtrip() -> DependencyTree {
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
            risk_level: None,
            risk_reason: None,
        }],
    }
}
