mod support;

use gradle_dependency_check::analysis::multi_module_assembler;
use gradle_dependency_check::dependency::models::*;
use support::factories;

#[test]
fn assembles_two_module_trees() {
    let app_module = factories::module("app");
    let core_module = factories::module("core");

    let app_tree = factories::simple_tree();
    let core_tree = factories::simple_tree();

    let result = multi_module_assembler::assemble(
        "my-project",
        GradleConfiguration::CompileClasspath,
        vec![(app_module, app_tree), (core_module, core_tree)],
    );

    assert_eq!(result.roots.len(), 2);
    assert_eq!(result.project_name, "my-project");
    assert_eq!(result.configuration, GradleConfiguration::CompileClasspath);
}

#[test]
fn synthetic_nodes_have_module_version() {
    let app_module = factories::module("app");
    let tree = factories::simple_tree();

    let result = multi_module_assembler::assemble(
        "my-project",
        GradleConfiguration::CompileClasspath,
        vec![(app_module, tree)],
    );

    let synthetic = &result.roots[0];
    assert_eq!(synthetic.group, "my-project");
    assert_eq!(synthetic.artifact, "app");
    assert_eq!(synthetic.requested_version, "module");
}

#[test]
fn aggregates_conflicts_from_multiple_modules() {
    let app_module = factories::module("app");
    let core_module = factories::module("core");

    let app_tree = factories::tree_with_conflicts();
    let core_tree = factories::tree_with_conflicts();

    let expected_count = app_tree.conflicts.len() + core_tree.conflicts.len();

    let result = multi_module_assembler::assemble(
        "my-project",
        GradleConfiguration::RuntimeClasspath,
        vec![(app_module, app_tree), (core_module, core_tree)],
    );

    assert_eq!(result.conflicts.len(), expected_count);
}

#[test]
fn single_module_assembly() {
    let module = factories::module("app");
    let tree = factories::simple_tree();
    let original_root_count = tree.roots.len();

    let result = multi_module_assembler::assemble(
        "my-project",
        GradleConfiguration::CompileClasspath,
        vec![(module, tree)],
    );

    assert_eq!(result.roots.len(), 1);
    assert_eq!(result.roots[0].artifact, "app");
    assert_eq!(result.roots[0].children.len(), original_root_count);
}

#[test]
fn empty_modules_returns_empty_tree() {
    let result = multi_module_assembler::assemble(
        "my-project",
        GradleConfiguration::CompileClasspath,
        vec![],
    );

    assert!(result.roots.is_empty());
    assert!(result.conflicts.is_empty());
}
