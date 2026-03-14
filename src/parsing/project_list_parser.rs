use regex::Regex;
use std::sync::LazyLock;

use crate::dependency::models::GradleModule;

static PROJECT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[+\\]--- Project '([^']+)'").unwrap());

/// Parses `gradle projects` output to extract module paths.
pub fn parse(output: &str) -> Vec<GradleModule> {
    PROJECT_REGEX
        .captures_iter(output)
        .map(|caps| {
            let path = caps[1].to_string();
            let name = path
                .rsplit(':')
                .next()
                .unwrap_or(&path)
                .to_string();
            GradleModule { name, path }
        })
        .collect()
}
