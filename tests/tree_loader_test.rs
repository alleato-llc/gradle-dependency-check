mod support;

use gradle_dependency_check::analysis::tree_loader;
use gradle_dependency_check::dependency::models::*;
use support::test_runner::{RunnerCall, TestGradleRunner};

const SIMPLE_OUTPUT: &str = "\
+--- com.google.guava:guava:31.1-jre
\\--- org.slf4j:slf4j-api:1.7.36";

const APP_MODULE_OUTPUT: &str = "\
+--- com.google.guava:guava:31.1-jre
\\--- org.springframework:spring-core:5.3.20";

const CORE_MODULE_OUTPUT: &str = "\
+--- com.google.guava:guava:30.0-jre
\\--- org.apache.commons:commons-lang3:3.17.0";

#[test]
fn single_module_calls_run_dependencies() {
    let runner = TestGradleRunner::new().with_dependency_output(SIMPLE_OUTPUT);

    let tree = tree_loader::load_tree(
        &runner,
        "/tmp/my-project",
        GradleConfiguration::CompileClasspath,
        None,
    )
    .unwrap();

    assert_eq!(tree.roots.len(), 2);
    assert_eq!(tree.project_name, "my-project");

    let calls = runner.calls();
    // Should call list_projects first, then run_dependencies
    assert!(calls.len() >= 2);
    assert!(matches!(calls[0], RunnerCall::ListProjects { .. }));
    assert!(matches!(calls[1], RunnerCall::RunDependencies { .. }));
}

#[test]
fn specific_module_calls_run_module_dependencies() {
    let runner = TestGradleRunner::new().with_module_output(":app", APP_MODULE_OUTPUT);

    let tree = tree_loader::load_tree(
        &runner,
        "/tmp/my-project",
        GradleConfiguration::CompileClasspath,
        Some(":app"),
    )
    .unwrap();

    assert_eq!(tree.roots.len(), 2);

    let calls = runner.calls();
    assert_eq!(calls.len(), 1);
    assert!(matches!(
        &calls[0],
        RunnerCall::RunModuleDependencies { module_path, .. } if module_path == ":app"
    ));
}

#[test]
fn multi_module_assembles_all_modules() {
    let app_module = GradleModule {
        name: "app".to_string(),
        path: ":app".to_string(),
    };
    let core_module = GradleModule {
        name: "core".to_string(),
        path: ":core".to_string(),
    };

    let runner = TestGradleRunner::new()
        .with_modules(vec![app_module, core_module])
        .with_module_output(":app", APP_MODULE_OUTPUT)
        .with_module_output(":core", CORE_MODULE_OUTPUT);

    let tree = tree_loader::load_tree(
        &runner,
        "/tmp/my-project",
        GradleConfiguration::CompileClasspath,
        None,
    )
    .unwrap();

    // Should have 2 synthetic module roots
    assert_eq!(tree.roots.len(), 2);
    assert_eq!(tree.roots[0].requested_version, "module");
    assert_eq!(tree.roots[1].requested_version, "module");

    // Each module root should have its dependencies as children
    assert_eq!(tree.roots[0].children.len(), 2);
    assert_eq!(tree.roots[1].children.len(), 2);

    // Should have called list_projects + 2x run_module_dependencies
    assert_eq!(runner.call_count(), 3);
}

#[test]
fn error_propagates_from_runner() {
    let runner = TestGradleRunner::new()
        .with_error(gradle_dependency_check::error::RunnerError::GradlewNotFound(
            "/tmp/my-project/gradlew".to_string(),
        ));

    let result = tree_loader::load_tree(
        &runner,
        "/tmp/my-project",
        GradleConfiguration::CompileClasspath,
        None,
    );

    assert!(result.is_err());
}

#[test]
fn module_name_extracted_from_path() {
    let runner = TestGradleRunner::new().with_module_output(":app:feature", APP_MODULE_OUTPUT);

    let tree = tree_loader::load_tree(
        &runner,
        "/tmp/my-project",
        GradleConfiguration::CompileClasspath,
        Some(":app:feature"),
    )
    .unwrap();

    // Project name comes from the directory, not the module
    assert_eq!(tree.project_name, "my-project");
}

#[test]
fn configuration_passed_to_runner() {
    let runner = TestGradleRunner::new().with_dependency_output(SIMPLE_OUTPUT);

    tree_loader::load_tree(
        &runner,
        "/tmp/my-project",
        GradleConfiguration::RuntimeClasspath,
        None,
    )
    .unwrap();

    let calls = runner.calls();
    // The run_dependencies call should have RuntimeClasspath
    let dep_call = calls.iter().find(|c| matches!(c, RunnerCall::RunDependencies { .. }));
    assert!(matches!(
        dep_call,
        Some(RunnerCall::RunDependencies { configuration: GradleConfiguration::RuntimeClasspath, .. })
    ));
}

#[test]
fn skips_failing_modules_and_loads_remaining() {
    let app_module = GradleModule {
        name: "app".to_string(),
        path: ":app".to_string(),
    };
    let distribution_module = GradleModule {
        name: "distribution".to_string(),
        path: ":distribution".to_string(),
    };
    let core_module = GradleModule {
        name: "core".to_string(),
        path: ":core".to_string(),
    };

    let runner = TestGradleRunner::new()
        .with_modules(vec![
            app_module,
            distribution_module,
            core_module,
        ])
        .with_module_output(":app", APP_MODULE_OUTPUT)
        .with_module_error(
            ":distribution",
            gradle_dependency_check::error::RunnerError::ExecutionFailed {
                exit_code: 1,
                stderr: "configuration 'runtimeClasspath' not found".to_string(),
            },
        )
        .with_module_output(":core", CORE_MODULE_OUTPUT);

    let tree = tree_loader::load_tree(
        &runner,
        "/tmp/my-project",
        GradleConfiguration::CompileClasspath,
        None,
    )
    .unwrap();

    // Should have 2 module roots (app + core), distribution skipped
    assert_eq!(tree.roots.len(), 2);

    // All 3 modules should have been attempted
    assert_eq!(runner.call_count(), 4); // list_projects + 3 module calls
}

#[test]
fn all_modules_failing_returns_error() {
    let dist_module = GradleModule {
        name: "distribution".to_string(),
        path: ":distribution".to_string(),
    };

    let runner = TestGradleRunner::new()
        .with_modules(vec![dist_module])
        .with_module_error(
            ":distribution",
            gradle_dependency_check::error::RunnerError::ExecutionFailed {
                exit_code: 1,
                stderr: "configuration 'runtimeClasspath' not found".to_string(),
            },
        );

    let result = tree_loader::load_tree(
        &runner,
        "/tmp/my-project",
        GradleConfiguration::RuntimeClasspath,
        None,
    );

    assert!(result.is_err());
}

#[test]
fn empty_output_produces_empty_tree() {
    let runner = TestGradleRunner::new().with_dependency_output("");

    let tree = tree_loader::load_tree(
        &runner,
        "/tmp/my-project",
        GradleConfiguration::CompileClasspath,
        None,
    )
    .unwrap();

    assert!(tree.roots.is_empty());
}
