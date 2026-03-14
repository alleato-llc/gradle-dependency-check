use regex::Regex;
use std::sync::LazyLock;

use crate::dependency::models::DependencyDeclaration;

static GROOVY_STRING_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(implementation|testImplementation|api|compileOnly|runtimeOnly|annotationProcessor)\s+['"]([^:]+):([^:]+):([^'"]+)['"]"#,
    )
    .unwrap()
});

static KOTLIN_DSL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(implementation|testImplementation|api|compileOnly|runtimeOnly|annotationProcessor)\s*\(\s*["']([^:]+):([^:]+):([^"']+)["']\s*\)"#,
    )
    .unwrap()
});

static GROOVY_MAP_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(implementation|testImplementation|api|compileOnly|runtimeOnly|annotationProcessor)\s+group:\s*['"]([^'"]+)['"]\s*,\s*name:\s*['"]([^'"]+)['"]\s*,\s*version:\s*['"]([^'"]+)['"]"#,
    )
    .unwrap()
});

/// Parses dependency declarations from build.gradle or build.gradle.kts content.
pub fn parse(content: &str) -> Vec<DependencyDeclaration> {
    let mut results = Vec::new();
    let mut in_block_comment = false;

    for (index, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.contains("/*") {
            in_block_comment = true;
        }
        if trimmed.contains("*/") {
            in_block_comment = false;
            continue;
        }
        if in_block_comment {
            continue;
        }
        if trimmed.starts_with("//") {
            continue;
        }

        let line_number = index + 1;
        let patterns: &[&Regex] = &[&GROOVY_STRING_REGEX, &KOTLIN_DSL_REGEX, &GROOVY_MAP_REGEX];

        for pattern in patterns {
            if let Some(caps) = pattern.captures(line) {
                results.push(DependencyDeclaration {
                    configuration: caps[1].to_string(),
                    group: caps[2].to_string(),
                    artifact: caps[3].to_string(),
                    version: caps[4].to_string(),
                    line: line_number,
                });
                break;
            }
        }
    }

    results
}
