use std::cell::RefCell;

use gradle_dependency_check::dependency::models::{GradleConfiguration, GradleModule};
use gradle_dependency_check::error::RunnerError;
use gradle_dependency_check::runner::gradle_runner::GradleRunner;

/// Recorded call to the GradleRunner.
#[derive(Debug, Clone)]
pub enum RunnerCall {
    RunDependencies {
        project_path: String,
        configuration: GradleConfiguration,
    },
    RunModuleDependencies {
        project_path: String,
        module_path: String,
        configuration: GradleConfiguration,
    },
    ListProjects {
        project_path: String,
    },
}

/// Test double for GradleRunner that returns configured responses
/// and records all calls via RefCell for assertion.
pub struct TestGradleRunner {
    calls: RefCell<Vec<RunnerCall>>,
    dependency_output: RefCell<String>,
    module_outputs: RefCell<std::collections::HashMap<String, String>>,
    modules: RefCell<Vec<GradleModule>>,
    error_to_return: RefCell<Option<RunnerError>>,
}

impl TestGradleRunner {
    pub fn new() -> Self {
        Self {
            calls: RefCell::new(Vec::new()),
            dependency_output: RefCell::new(String::new()),
            module_outputs: RefCell::new(std::collections::HashMap::new()),
            modules: RefCell::new(Vec::new()),
            error_to_return: RefCell::new(None),
        }
    }

    pub fn with_dependency_output(self, output: &str) -> Self {
        *self.dependency_output.borrow_mut() = output.to_string();
        self
    }

    pub fn with_module_output(self, module_path: &str, output: &str) -> Self {
        self.module_outputs
            .borrow_mut()
            .insert(module_path.to_string(), output.to_string());
        self
    }

    pub fn with_modules(self, modules: Vec<GradleModule>) -> Self {
        *self.modules.borrow_mut() = modules;
        self
    }

    pub fn with_error(self, error: RunnerError) -> Self {
        *self.error_to_return.borrow_mut() = Some(error);
        self
    }

    pub fn calls(&self) -> Vec<RunnerCall> {
        self.calls.borrow().clone()
    }

    pub fn call_count(&self) -> usize {
        self.calls.borrow().len()
    }
}

impl GradleRunner for TestGradleRunner {
    fn run_dependencies(
        &self,
        project_path: &str,
        configuration: GradleConfiguration,
    ) -> Result<String, RunnerError> {
        self.calls.borrow_mut().push(RunnerCall::RunDependencies {
            project_path: project_path.to_string(),
            configuration,
        });

        if let Some(err) = self.error_to_return.borrow_mut().take() {
            return Err(err);
        }

        Ok(self.dependency_output.borrow().clone())
    }

    fn run_module_dependencies(
        &self,
        project_path: &str,
        module: &GradleModule,
        configuration: GradleConfiguration,
    ) -> Result<String, RunnerError> {
        self.calls
            .borrow_mut()
            .push(RunnerCall::RunModuleDependencies {
                project_path: project_path.to_string(),
                module_path: module.path.clone(),
                configuration,
            });

        if let Some(err) = self.error_to_return.borrow_mut().take() {
            return Err(err);
        }

        let outputs = self.module_outputs.borrow();
        Ok(outputs
            .get(&module.path)
            .cloned()
            .unwrap_or_default())
    }

    fn list_projects(&self, project_path: &str) -> Result<Vec<GradleModule>, RunnerError> {
        self.calls.borrow_mut().push(RunnerCall::ListProjects {
            project_path: project_path.to_string(),
        });

        Ok(self.modules.borrow().clone())
    }
}
