use std::path::Path;

use crate::analysis::multi_module_assembler;
use crate::dependency::models::{DependencyTree, GradleConfiguration, GradleModule};
use crate::error::RunnerError;
use crate::parsing::tree_parser;
use crate::runner::gradle_runner::GradleRunner;

/// Loads a dependency tree from a Gradle project using the provided runner.
/// Handles single-module, specific-module, and multi-module projects.
pub fn load_tree(
    runner: &dyn GradleRunner,
    project_path: &str,
    configuration: GradleConfiguration,
    module: Option<&str>,
) -> Result<DependencyTree, RunnerError> {
    let project_name = Path::new(project_path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| project_path.to_string());

    if let Some(module_path) = module {
        let name = module_path
            .rsplit(':')
            .next()
            .unwrap_or(module_path)
            .to_string();
        let grad_module = GradleModule {
            name,
            path: module_path.to_string(),
        };
        let output = runner.run_module_dependencies(project_path, &grad_module, configuration)?;
        return Ok(tree_parser::parse(&output, &project_name, configuration));
    }

    let modules = runner.list_projects(project_path).unwrap_or_default();
    if modules.is_empty() {
        let output = runner.run_dependencies(project_path, configuration)?;
        return Ok(tree_parser::parse(&output, &project_name, configuration));
    }

    let mut module_trees = Vec::new();
    let mut skipped = Vec::new();
    for m in &modules {
        match runner.run_module_dependencies(project_path, m, configuration) {
            Ok(output) => {
                let module_tree = tree_parser::parse(&output, &m.name, configuration);
                module_trees.push((m.clone(), module_tree));
            }
            Err(e) => {
                eprintln!("Warning: skipping module {}: {}", m.path, e);
                skipped.push(m.path.clone());
            }
        }
    }

    if module_trees.is_empty() {
        return Err(RunnerError::ExecutionFailed {
            exit_code: 1,
            stderr: format!(
                "No modules could be loaded for configuration '{}'. All {} modules failed.",
                configuration.as_str(),
                skipped.len()
            ),
        });
    }

    if !skipped.is_empty() {
        eprintln!(
            "Loaded {}/{} modules ({} skipped)",
            module_trees.len(),
            modules.len(),
            skipped.len()
        );
    }

    Ok(multi_module_assembler::assemble(
        &project_name,
        configuration,
        module_trees,
    ))
}
