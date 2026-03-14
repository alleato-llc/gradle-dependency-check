use crate::analysis::tree_analysis;
use crate::dependency::models::{DependencyConflict, DependencyTree, RiskLevel};

/// Parsed semver components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SemVer {
    major: u64,
    minor: u64,
    patch: u64,
}

impl SemVer {
    fn cmp_tuple(&self) -> (u64, u64, u64) {
        (self.major, self.minor, self.patch)
    }
}

/// Strips known qualifiers from a version string before numeric parsing.
fn strip_qualifiers(version: &str) -> &str {
    // Strip trailing qualifiers like .Final, .RELEASE, -jre, -beta1, -SNAPSHOT, -android, etc.
    let stripped = version
        .trim_end_matches(".Final")
        .trim_end_matches(".RELEASE")
        .trim_end_matches("-jre")
        .trim_end_matches("-android")
        .trim_end_matches("-SNAPSHOT");

    // Also strip -beta*, -alpha*, -rc* suffixes
    if let Some(pos) = stripped.rfind('-') {
        let suffix = &stripped[pos + 1..];
        if suffix.starts_with("beta")
            || suffix.starts_with("alpha")
            || suffix.starts_with("rc")
            || suffix.starts_with("RC")
            || suffix.starts_with("M")
        {
            return &stripped[..pos];
        }
    }
    stripped
}

/// Parses a version string into SemVer components.
/// Handles formats like "1.7.36", "3.4.3.Final", "1.9.22.1", "31.1-jre".
/// Returns None if the version cannot be parsed at all.
fn parse_semver(version: &str) -> Option<SemVer> {
    let cleaned = strip_qualifiers(version);
    let parts: Vec<&str> = cleaned.split('.').collect();

    if parts.is_empty() {
        return None;
    }

    let major = parts.first().and_then(|p| p.parse::<u64>().ok())?;
    let minor = parts.get(1).and_then(|p| p.parse::<u64>().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|p| p.parse::<u64>().ok()).unwrap_or(0);
    // 4th segment is intentionally ignored (e.g., 1.9.22.1 -> major=1, minor=9, patch=22)

    Some(SemVer {
        major,
        minor,
        patch,
    })
}

/// Determines the base risk level from the version distance between requested and resolved.
fn base_risk(requested: &SemVer, resolved: &SemVer) -> RiskLevel {
    if requested.major != resolved.major {
        RiskLevel::High
    } else if requested.minor != resolved.minor {
        RiskLevel::Medium
    } else if requested.patch != resolved.patch {
        RiskLevel::Low
    } else {
        RiskLevel::Info
    }
}

/// Generates the base risk reason string.
fn base_reason(requested: &SemVer, resolved: &SemVer) -> String {
    if requested.major != resolved.major {
        format!(
            "Major version jump ({}.x -> {}.x)",
            requested.major, resolved.major
        )
    } else if requested.minor != resolved.minor {
        format!(
            "Minor version jump ({}.{} -> {}.{})",
            requested.major, requested.minor, resolved.major, resolved.minor
        )
    } else if requested.patch != resolved.patch {
        format!(
            "Patch version bump ({}.{}.{} -> {}.{}.{})",
            requested.major,
            requested.minor,
            requested.patch,
            resolved.major,
            resolved.minor,
            resolved.patch
        )
    } else {
        "Qualifier change only".to_string()
    }
}

/// Shifts a risk level up by 1, clamped to CRITICAL.
fn shift_up(level: RiskLevel) -> RiskLevel {
    match level {
        RiskLevel::Info => RiskLevel::Low,
        RiskLevel::Low => RiskLevel::Medium,
        RiskLevel::Medium => RiskLevel::High,
        RiskLevel::High => RiskLevel::Critical,
        RiskLevel::Critical => RiskLevel::Critical,
    }
}

/// Shifts a risk level down by 1, clamped to INFO.
fn shift_down(level: RiskLevel) -> RiskLevel {
    match level {
        RiskLevel::Info => RiskLevel::Info,
        RiskLevel::Low => RiskLevel::Info,
        RiskLevel::Medium => RiskLevel::Low,
        RiskLevel::High => RiskLevel::Medium,
        RiskLevel::Critical => RiskLevel::High,
    }
}

/// Checks if a dependency is BOM-managed: any node in the tree has `is_constraint == true`
/// with the same coordinate and version matching the resolved version.
fn is_bom_managed(tree: &DependencyTree, coordinate: &str, resolved_version: &str) -> bool {
    let all_nodes = tree_analysis::all_nodes(tree);
    all_nodes.iter().any(|node| {
        node.is_constraint
            && node.coordinate() == coordinate
            && node.requested_version == resolved_version
    })
}

/// Checks if the resolved version is less than the requested version (a downgrade).
fn is_downgrade(requested: &SemVer, resolved: &SemVer) -> bool {
    resolved.cmp_tuple() < requested.cmp_tuple()
}

/// Assesses risk for all conflicts in a dependency tree.
/// Returns the conflicts with `risk_level` and `risk_reason` populated.
pub fn assess_conflicts(tree: &DependencyTree) -> Vec<DependencyConflict> {
    let is_production = tree.configuration.is_production();

    tree.conflicts
        .iter()
        .map(|conflict| {
            let mut assessed = conflict.clone();

            let requested_sv = parse_semver(&conflict.requested_version);
            let resolved_sv = parse_semver(&conflict.resolved_version);

            match (requested_sv, resolved_sv) {
                (Some(req), Some(res)) => {
                    let mut level = base_risk(&req, &res);
                    let mut reason = base_reason(&req, &res);

                    // BOM-managed adjustment (-1)
                    if is_bom_managed(tree, &conflict.coordinate, &conflict.resolved_version) {
                        level = shift_down(level);
                        reason.push_str(", reduced: BOM-managed");
                    }

                    // Downgrade adjustment (+1)
                    if is_downgrade(&req, &res) {
                        level = shift_up(level);
                        reason.push_str(", downgrade detected");
                    }

                    // Test scope adjustment (-1)
                    if !is_production {
                        level = shift_down(level);
                        reason.push_str(", reduced: test scope");
                    }

                    assessed.risk_level = Some(level);
                    assessed.risk_reason = Some(reason);
                }
                _ => {
                    // Non-parseable versions default to MEDIUM
                    let mut level = RiskLevel::Medium;
                    let mut reason = format!(
                        "Unparseable version(s): {} -> {}",
                        conflict.requested_version, conflict.resolved_version
                    );

                    if is_bom_managed(tree, &conflict.coordinate, &conflict.resolved_version) {
                        level = shift_down(level);
                        reason.push_str(", reduced: BOM-managed");
                    }

                    if !is_production {
                        level = shift_down(level);
                        reason.push_str(", reduced: test scope");
                    }

                    assessed.risk_level = Some(level);
                    assessed.risk_reason = Some(reason);
                }
            }

            assessed
        })
        .collect()
}
