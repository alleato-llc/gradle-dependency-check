use std::path::Path;
use std::process::Command;

use crate::dependency::models::{GradleConfiguration, GradleModule};
use crate::error::RunnerError;
use crate::parsing::project_list_parser;
use crate::runner::gradle_runner::GradleRunner;

pub struct ProcessGradleRunner;

impl ProcessGradleRunner {
    fn gradlew_path(project_path: &str) -> Result<String, RunnerError> {
        let path = Path::new(project_path).join("gradlew");
        if path.exists() {
            Ok(path.to_string_lossy().to_string())
        } else {
            Err(RunnerError::GradlewNotFound(
                path.to_string_lossy().to_string(),
            ))
        }
    }

    fn execute(project_path: &str, args: &[&str]) -> Result<String, RunnerError> {
        let gradlew = Self::gradlew_path(project_path)?;

        let output = Command::new(&gradlew)
            .current_dir(project_path)
            .args(args)
            .output()
            .map_err(|e| RunnerError::LaunchFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(RunnerError::ExecutionFailed {
                exit_code: output.status.code().unwrap_or(-1),
                stderr,
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl GradleRunner for ProcessGradleRunner {
    fn run_dependencies(
        &self,
        project_path: &str,
        configuration: GradleConfiguration,
    ) -> Result<String, RunnerError> {
        Self::execute(
            project_path,
            &[
                "dependencies",
                "--configuration",
                configuration.as_str(),
                "-q",
            ],
        )
    }

    fn run_module_dependencies(
        &self,
        project_path: &str,
        module: &GradleModule,
        configuration: GradleConfiguration,
    ) -> Result<String, RunnerError> {
        let task = format!("{}:dependencies", module.path);
        Self::execute(
            project_path,
            &[&task, "--configuration", configuration.as_str(), "-q"],
        )
    }

    fn list_projects(&self, project_path: &str) -> Result<Vec<GradleModule>, RunnerError> {
        let output = Self::execute(project_path, &["projects", "-q"])?;
        Ok(project_list_parser::parse(&output))
    }
}
