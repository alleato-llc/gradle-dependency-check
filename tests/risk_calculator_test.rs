mod support;

use gradle_dependency_check::analysis::risk_calculator;
use gradle_dependency_check::dependency::models::*;
use support::factories;

fn conflict(
    coordinate: &str,
    requested: &str,
    resolved: &str,
    requested_by: &str,
) -> DependencyConflict {
    DependencyConflict {
        coordinate: coordinate.to_string(),
        requested_version: requested.to_string(),
        resolved_version: resolved.to_string(),
        requested_by: requested_by.to_string(),
        risk_level: None,
        risk_reason: None,
    }
}

fn tree_with_single_conflict(
    config: GradleConfiguration,
    conflict: DependencyConflict,
    roots: Vec<DependencyNode>,
) -> DependencyTree {
    DependencyTree {
        project_name: "test-project".to_string(),
        configuration: config,
        roots,
        conflicts: vec![conflict],
    }
}

#[test]
fn major_version_jump_is_high() {
    let c = conflict("org.example:lib", "1.0.0", "2.0.0", "root");
    let root = factories::node("com.example", "app", "1.0.0");
    let tree = tree_with_single_conflict(GradleConfiguration::CompileClasspath, c, vec![root]);

    let assessed = risk_calculator::assess_conflicts(&tree);
    assert_eq!(assessed.len(), 1);
    assert_eq!(assessed[0].risk_level, Some(RiskLevel::High));
    assert!(assessed[0]
        .risk_reason
        .as_ref()
        .unwrap()
        .contains("Major version jump"));
}

#[test]
fn minor_version_jump_is_medium() {
    let c = conflict("org.example:lib", "1.0.0", "1.5.0", "root");
    let root = factories::node("com.example", "app", "1.0.0");
    let tree = tree_with_single_conflict(GradleConfiguration::CompileClasspath, c, vec![root]);

    let assessed = risk_calculator::assess_conflicts(&tree);
    assert_eq!(assessed[0].risk_level, Some(RiskLevel::Medium));
    assert!(assessed[0]
        .risk_reason
        .as_ref()
        .unwrap()
        .contains("Minor version jump"));
}

#[test]
fn patch_version_bump_is_low() {
    let c = conflict("org.example:lib", "1.0.0", "1.0.5", "root");
    let root = factories::node("com.example", "app", "1.0.0");
    let tree = tree_with_single_conflict(GradleConfiguration::CompileClasspath, c, vec![root]);

    let assessed = risk_calculator::assess_conflicts(&tree);
    assert_eq!(assessed[0].risk_level, Some(RiskLevel::Low));
    assert!(assessed[0]
        .risk_reason
        .as_ref()
        .unwrap()
        .contains("Patch version bump"));
}

#[test]
fn qualifier_only_is_info() {
    // "1.0.0.Final" and "1.0.0.RELEASE" have same numeric parts
    let c = conflict("org.example:lib", "1.0.0.Final", "1.0.0.RELEASE", "root");
    let root = factories::node("com.example", "app", "1.0.0");
    let tree = tree_with_single_conflict(GradleConfiguration::CompileClasspath, c, vec![root]);

    let assessed = risk_calculator::assess_conflicts(&tree);
    assert_eq!(assessed[0].risk_level, Some(RiskLevel::Info));
    assert!(assessed[0]
        .risk_reason
        .as_ref()
        .unwrap()
        .contains("Qualifier change only"));
}

#[test]
fn bom_managed_reduces_risk() {
    let c = conflict("org.example:lib", "1.0.0", "2.0.0", "root");

    // Add a constraint node that matches the coordinate and resolved version
    let mut constraint = DependencyNode::new("org.example", "lib", "2.0.0");
    constraint.is_constraint = true;
    let mut root = factories::node("com.example", "app", "1.0.0");
    root.children = vec![constraint];

    let tree = tree_with_single_conflict(GradleConfiguration::CompileClasspath, c, vec![root]);

    let assessed = risk_calculator::assess_conflicts(&tree);
    // HIGH base, reduced by 1 for BOM -> MEDIUM
    assert_eq!(assessed[0].risk_level, Some(RiskLevel::Medium));
    assert!(assessed[0]
        .risk_reason
        .as_ref()
        .unwrap()
        .contains("BOM-managed"));
}

#[test]
fn downgrade_increases_risk() {
    // resolved (1.0.0) < requested (2.0.0) -> downgrade
    let c = conflict("org.example:lib", "2.0.0", "1.0.0", "root");
    let root = factories::node("com.example", "app", "1.0.0");
    let tree = tree_with_single_conflict(GradleConfiguration::CompileClasspath, c, vec![root]);

    let assessed = risk_calculator::assess_conflicts(&tree);
    // HIGH base (major diff) + 1 for downgrade -> CRITICAL
    assert_eq!(assessed[0].risk_level, Some(RiskLevel::Critical));
    assert!(assessed[0]
        .risk_reason
        .as_ref()
        .unwrap()
        .contains("downgrade detected"));
}

#[test]
fn test_scope_reduces_risk() {
    let c = conflict("org.example:lib", "1.0.0", "2.0.0", "root");
    let root = factories::node("com.example", "app", "1.0.0");
    let tree = tree_with_single_conflict(GradleConfiguration::TestCompileClasspath, c, vec![root]);

    let assessed = risk_calculator::assess_conflicts(&tree);
    // HIGH base, reduced by 1 for test scope -> MEDIUM
    assert_eq!(assessed[0].risk_level, Some(RiskLevel::Medium));
    assert!(assessed[0]
        .risk_reason
        .as_ref()
        .unwrap()
        .contains("test scope"));
}

#[test]
fn combined_adjustments() {
    let c = conflict("org.example:lib", "1.0.0", "2.0.0", "root");

    // BOM-managed constraint node
    let mut constraint = DependencyNode::new("org.example", "lib", "2.0.0");
    constraint.is_constraint = true;
    let mut root = factories::node("com.example", "app", "1.0.0");
    root.children = vec![constraint];

    // Test scope configuration
    let tree =
        tree_with_single_conflict(GradleConfiguration::TestCompileClasspath, c, vec![root]);

    let assessed = risk_calculator::assess_conflicts(&tree);
    // HIGH base, -1 BOM, -1 test scope -> LOW
    assert_eq!(assessed[0].risk_level, Some(RiskLevel::Low));
    let reason = assessed[0].risk_reason.as_ref().unwrap();
    assert!(reason.contains("BOM-managed"));
    assert!(reason.contains("test scope"));
}

#[test]
fn non_semver_handled_gracefully() {
    let c = conflict("org.example:lib", "RELEASE", "SNAPSHOT", "root");
    let root = factories::node("com.example", "app", "1.0.0");
    let tree = tree_with_single_conflict(GradleConfiguration::CompileClasspath, c, vec![root]);

    let assessed = risk_calculator::assess_conflicts(&tree);
    assert_eq!(assessed[0].risk_level, Some(RiskLevel::Medium));
    assert!(assessed[0]
        .risk_reason
        .as_ref()
        .unwrap()
        .contains("Unparseable"));
}

#[test]
fn multi_segment_patch() {
    // 1.9.22.1 -> 1.9.25.1: both parse as major=1, minor=9, patch differs (22 vs 25) -> LOW
    let c = conflict("org.example:lib", "1.9.22.1", "1.9.25.1", "root");
    let root = factories::node("com.example", "app", "1.0.0");
    let tree = tree_with_single_conflict(GradleConfiguration::CompileClasspath, c, vec![root]);

    let assessed = risk_calculator::assess_conflicts(&tree);
    assert_eq!(assessed[0].risk_level, Some(RiskLevel::Low));
    assert!(assessed[0]
        .risk_reason
        .as_ref()
        .unwrap()
        .contains("Patch version bump"));
}

#[test]
fn real_spring_boot_conflicts() {
    // Simulate a Spring Boot app with typical dependency conflicts

    // SLF4J: major version jump 1.x -> 2.x
    let slf4j_conflict = conflict(
        "org.slf4j:slf4j-api",
        "1.7.36",
        "2.0.17",
        "ch.qos.logback:logback-classic",
    );

    // Jackson: minor version bump
    let jackson_conflict = conflict(
        "com.fasterxml.jackson.core:jackson-databind",
        "2.13.0",
        "2.15.2",
        "org.springframework.boot:spring-boot-starter-web",
    );

    // Snakeyaml: patch bump
    let snakeyaml_conflict = conflict(
        "org.yaml:snakeyaml",
        "1.33.0",
        "1.33.5",
        "org.springframework.boot:spring-boot-starter",
    );

    // BOM-managed jackson (constraint node present)
    let mut jackson_constraint =
        DependencyNode::new("com.fasterxml.jackson.core", "jackson-databind", "2.15.2");
    jackson_constraint.is_constraint = true;

    let slf4j = DependencyNode::new("org.slf4j", "slf4j-api", "1.7.36");
    let jackson =
        DependencyNode::new("com.fasterxml.jackson.core", "jackson-databind", "2.13.0");
    let snakeyaml = DependencyNode::new("org.yaml", "snakeyaml", "1.33.0");

    let mut root = DependencyNode::new("com.example", "my-app", "1.0.0");
    root.children = vec![slf4j, jackson, snakeyaml, jackson_constraint];

    let tree = DependencyTree {
        project_name: "spring-boot-app".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![root],
        conflicts: vec![slf4j_conflict, jackson_conflict, snakeyaml_conflict],
    };

    let assessed = risk_calculator::assess_conflicts(&tree);
    assert_eq!(assessed.len(), 3);

    // SLF4J: major jump -> HIGH
    assert_eq!(assessed[0].risk_level, Some(RiskLevel::High));
    assert!(assessed[0]
        .risk_reason
        .as_ref()
        .unwrap()
        .contains("Major version jump (1.x -> 2.x)"));

    // Jackson: minor jump, but BOM-managed -> MEDIUM - 1 = LOW
    assert_eq!(assessed[1].risk_level, Some(RiskLevel::Low));
    let jackson_reason = assessed[1].risk_reason.as_ref().unwrap();
    assert!(jackson_reason.contains("Minor version jump"));
    assert!(jackson_reason.contains("BOM-managed"));

    // Snakeyaml: patch bump -> LOW
    assert_eq!(assessed[2].risk_level, Some(RiskLevel::Low));
    assert!(assessed[2]
        .risk_reason
        .as_ref()
        .unwrap()
        .contains("Patch version bump"));
}
