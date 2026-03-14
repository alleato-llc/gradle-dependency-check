use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("No dependencies found in input")]
    NoDependenciesFound,

    #[error("Failed to parse dependency line: {0}")]
    InvalidLine(String),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
}

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("The file could not be read: {0}")]
    UnreadableFile(String),

    #[error("No dependencies were found in the file")]
    NoDependenciesFound,

    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum RunnerError {
    #[error("gradlew not found at: {0}")]
    GradlewNotFound(String),

    #[error("Gradle execution failed (exit code {exit_code}): {stderr}")]
    ExecutionFailed { exit_code: i32, stderr: String },

    #[error("Failed to launch Gradle: {0}")]
    LaunchFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum ExportError {
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
