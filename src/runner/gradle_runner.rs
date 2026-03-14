use crate::dependency::models::{GradleConfiguration, GradleModule};
use crate::error::RunnerError;

/// Trait boundary for executing Gradle commands.
pub trait GradleRunner {
    fn run_dependencies(
        &self,
        project_path: &str,
        configuration: GradleConfiguration,
    ) -> Result<String, RunnerError>;

    fn run_module_dependencies(
        &self,
        project_path: &str,
        module: &GradleModule,
        configuration: GradleConfiguration,
    ) -> Result<String, RunnerError>;

    fn list_projects(&self, project_path: &str) -> Result<Vec<GradleModule>, RunnerError>;
}
