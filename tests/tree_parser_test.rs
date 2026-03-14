mod support;

use gradle_dependency_check::dependency::models::GradleConfiguration;
use gradle_dependency_check::parsing::tree_parser;

#[test]
fn parses_single_dependency() {
    let output = r"\--- com.google.guava:guava:31.1-jre";
    let tree = tree_parser::parse(output, "test", GradleConfiguration::CompileClasspath);
    assert_eq!(tree.roots.len(), 1);
    assert_eq!(tree.roots[0].group, "com.google.guava");
    assert_eq!(tree.roots[0].artifact, "guava");
    assert_eq!(tree.roots[0].requested_version, "31.1-jre");
}

#[test]
fn parses_simple_dependency_with_plus_prefix() {
    let output = "+--- org.springframework:spring-core:5.3.20";
    let tree = tree_parser::parse(output, "test", GradleConfiguration::CompileClasspath);
    assert_eq!(tree.roots.len(), 1);
    assert_eq!(tree.roots[0].group, "org.springframework");
    assert_eq!(tree.roots[0].artifact, "spring-core");
    assert_eq!(tree.roots[0].requested_version, "5.3.20");
}

#[test]
fn parses_nested_dependencies() {
    let output = "\
+--- org.springframework:spring-core:5.3.20
|    +--- com.google.guava:guava:31.1-jre
|    \\--- org.slf4j:slf4j-api:1.7.36";
    let tree = tree_parser::parse(output, "test", GradleConfiguration::CompileClasspath);
    assert_eq!(tree.roots.len(), 1);
    assert_eq!(tree.roots[0].children.len(), 2);
    assert_eq!(tree.roots[0].children[0].artifact, "guava");
    assert_eq!(tree.roots[0].children[1].artifact, "slf4j-api");
}

#[test]
fn parses_version_conflict() {
    let output = r"\--- org.slf4j:slf4j-api:2.0.16 -> 2.0.17";
    let tree = tree_parser::parse(output, "test", GradleConfiguration::CompileClasspath);
    assert_eq!(tree.roots.len(), 1);
    assert_eq!(tree.roots[0].requested_version, "2.0.16");
    assert_eq!(tree.roots[0].resolved_version.as_deref(), Some("2.0.17"));
    assert!(tree.roots[0].has_conflict());
    assert_eq!(tree.conflicts.len(), 1);
}

#[test]
fn parses_conflict_marker_with_plus_prefix() {
    let output = "+--- com.fasterxml.jackson.core:jackson-databind:2.13.0 -> 2.14.2";
    let tree = tree_parser::parse(output, "test", GradleConfiguration::CompileClasspath);
    assert_eq!(tree.roots.len(), 1);
    let node = &tree.roots[0];
    assert_eq!(node.requested_version, "2.13.0");
    assert_eq!(node.resolved_version.as_deref(), Some("2.14.2"));
    assert!(node.has_conflict());
    assert_eq!(tree.conflicts.len(), 1);
}

#[test]
fn parses_omitted_node() {
    let output = r"\--- org.slf4j:slf4j-api:2.0.17 (*)";
    let tree = tree_parser::parse(output, "test", GradleConfiguration::CompileClasspath);
    assert_eq!(tree.roots.len(), 1);
    assert!(tree.roots[0].is_omitted);
}

#[test]
fn parses_constraint_node() {
    let output = r"\--- com.fasterxml.jackson.core:jackson-core:2.19.4 (c)";
    let tree = tree_parser::parse(output, "test", GradleConfiguration::CompileClasspath);
    assert_eq!(tree.roots.len(), 1);
    assert!(tree.roots[0].is_constraint);
}

#[test]
fn parses_multiple_roots() {
    let output = "\
+--- org.springframework:spring-core:5.3.20
+--- com.google.guava:guava:31.1-jre
\\--- org.slf4j:slf4j-api:1.7.36";
    let tree = tree_parser::parse(output, "test", GradleConfiguration::CompileClasspath);
    assert_eq!(tree.roots.len(), 3);
}

#[test]
fn parses_deeply_nested_tree() {
    let output = "\
+--- com.example:a:1.0
|    +--- com.example:b:2.0
|    |    \\--- com.example:c:3.0
|    \\--- com.example:d:4.0";
    let tree = tree_parser::parse(output, "test", GradleConfiguration::CompileClasspath);
    assert_eq!(tree.roots.len(), 1);
    let a = &tree.roots[0];
    assert_eq!(a.children.len(), 2);
    assert_eq!(a.children[0].artifact, "b");
    assert_eq!(a.children[0].children.len(), 1);
    assert_eq!(a.children[0].children[0].artifact, "c");
    assert_eq!(a.children[1].artifact, "d");
}

#[test]
fn parses_conflict_with_parent_tracking() {
    let output = "\
+--- org.springframework:spring-web:5.3.20
|    +--- com.fasterxml.jackson.core:jackson-databind:2.13.0 -> 2.14.2";
    let tree = tree_parser::parse(output, "test", GradleConfiguration::CompileClasspath);
    assert_eq!(tree.conflicts.len(), 1);
    assert_eq!(tree.conflicts[0].requested_by, "org.springframework:spring-web");
}

#[test]
fn ignores_non_dependency_lines() {
    let output = "\
\n\
------------------------------------------------------------\n\
Project ':app'\n\
------------------------------------------------------------\n\
\n\
compileClasspath - Compile classpath for source set 'main'.\n\
+--- org.springframework:spring-core:5.3.20\n\
\n\
(*) - Repeated dependencies omitted";
    let tree = tree_parser::parse(output, "test", GradleConfiguration::CompileClasspath);
    assert_eq!(tree.roots.len(), 1);
    assert_eq!(tree.roots[0].artifact, "spring-core");
}

#[test]
fn parses_bom_managed_dependency() {
    let output = r"\--- org.springframework.boot:spring-boot-starter-web -> 3.5.11";
    let tree = tree_parser::parse(output, "test", GradleConfiguration::CompileClasspath);
    assert_eq!(tree.roots.len(), 1);
    assert_eq!(tree.roots[0].requested_version, "3.5.11");
}

#[test]
fn parses_bom_managed_dependency_without_requested_version() {
    let output = "+--- org.springframework.boot:spring-boot-starter-web -> 3.5.11";
    let tree = tree_parser::parse(output, "test", GradleConfiguration::CompileClasspath);
    assert_eq!(tree.roots.len(), 1);
    let node = &tree.roots[0];
    assert_eq!(node.group, "org.springframework.boot");
    assert_eq!(node.artifact, "spring-boot-starter-web");
    assert_eq!(node.requested_version, "3.5.11");
    assert!(node.resolved_version.is_none());
    assert!(!node.has_conflict());
}

#[test]
fn parses_bom_managed_dependency_with_children() {
    let output = "\
+--- org.springframework.boot:spring-boot-starter-web -> 3.5.11
|    +--- org.springframework.boot:spring-boot-starter:3.5.11
|    \\--- org.springframework:spring-webmvc:6.2.16";
    let tree = tree_parser::parse(output, "test", GradleConfiguration::CompileClasspath);
    assert_eq!(tree.roots.len(), 1);
    assert_eq!(tree.roots[0].artifact, "spring-boot-starter-web");
    assert_eq!(tree.roots[0].children.len(), 2);
    assert_eq!(tree.roots[0].children[0].artifact, "spring-boot-starter");
    assert_eq!(tree.roots[0].children[1].artifact, "spring-webmvc");
}

#[test]
fn parses_spring_boot_multi_root_bom_managed_project() {
    let output = "\
> Task :dependencies

------------------------------------------------------------
Root project 'spring-boot-testing-reference'
------------------------------------------------------------

compileClasspath - Compile classpath for source set 'main'.
+--- org.springframework.boot:spring-boot-starter-web -> 3.5.11
|    +--- org.springframework.boot:spring-boot-starter:3.5.11
|    |    +--- org.springframework.boot:spring-boot:3.5.11
|    |    |    +--- org.springframework:spring-core:6.2.16
|    |    |    |    \\--- org.springframework:spring-jcl:6.2.16
|    |    |    \\--- org.springframework:spring-context:6.2.16
|    |    |         +--- org.springframework:spring-aop:6.2.16
|    |    |         |    +--- org.springframework:spring-beans:6.2.16
|    |    |         |    |    \\--- org.springframework:spring-core:6.2.16 (*)
|    |    |         |    \\--- org.springframework:spring-core:6.2.16 (*)
|    |    |         +--- org.springframework:spring-beans:6.2.16 (*)
|    |    |         +--- org.springframework:spring-core:6.2.16 (*)
|    |    |         \\--- org.springframework:spring-expression:6.2.16
|    |    |              \\--- org.springframework:spring-core:6.2.16 (*)
|    |    +--- org.springframework.boot:spring-boot-autoconfigure:3.5.11
|    |    |    \\--- org.springframework.boot:spring-boot:3.5.11 (*)
|    |    \\--- org.yaml:snakeyaml:2.4
|    +--- org.springframework.boot:spring-boot-starter-json:3.5.11
|    |    +--- com.fasterxml.jackson.core:jackson-databind:2.19.4
|    |    |    +--- com.fasterxml.jackson.core:jackson-annotations:2.19.4
|    |    |    |    \\--- com.fasterxml.jackson:jackson-bom:2.19.4
|    |    |    |         +--- com.fasterxml.jackson.core:jackson-annotations:2.19.4 (c)
|    |    |    |         +--- com.fasterxml.jackson.core:jackson-core:2.19.4 (c)
|    |    |    |         \\--- com.fasterxml.jackson.core:jackson-databind:2.19.4 (c)
|    |    |    \\--- com.fasterxml.jackson.core:jackson-core:2.19.4
|    |    |         \\--- com.fasterxml.jackson:jackson-bom:2.19.4 (*)
|    |    \\--- com.fasterxml.jackson.datatype:jackson-datatype-jsr310:2.19.4
|    |         +--- com.fasterxml.jackson.core:jackson-annotations:2.19.4 (*)
|    |         +--- com.fasterxml.jackson.core:jackson-core:2.19.4 (*)
|    |         +--- com.fasterxml.jackson.core:jackson-databind:2.19.4 (*)
|    |         \\--- com.fasterxml.jackson:jackson-bom:2.19.4 (*)
|    \\--- org.springframework:spring-webmvc:6.2.16
|         +--- org.springframework:spring-aop:6.2.16 (*)
|         +--- org.springframework:spring-beans:6.2.16 (*)
|         \\--- org.springframework:spring-web:6.2.16 (*)
+--- org.springframework.boot:spring-boot-starter-data-jpa -> 3.5.11
|    +--- org.springframework.boot:spring-boot-starter:3.5.11 (*)
|    +--- org.hibernate.orm:hibernate-core:6.6.42.Final
|    |    +--- jakarta.persistence:jakarta.persistence-api:3.1.0
|    |    \\--- jakarta.transaction:jakarta.transaction-api:2.0.1
|    \\--- org.springframework.data:spring-data-jpa:3.5.9
|         +--- org.springframework:spring-core:6.2.15 -> 6.2.16 (*)
|         +--- org.springframework:spring-beans:6.2.15 -> 6.2.16 (*)
|         \\--- org.antlr:antlr4-runtime:4.13.0
+--- software.amazon.awssdk:bom:2.42.11
|    +--- software.amazon.awssdk:s3:2.42.11 (c)
|    \\--- software.amazon.awssdk:sns:2.42.11 (c)
\\--- org.testng:testng:7.0.0
     \\--- com.beust:jcommander:1.72";

    let tree = tree_parser::parse(
        output,
        "spring-boot-testing-reference",
        GradleConfiguration::CompileClasspath,
    );

    // Verify all 4 root dependencies are parsed
    assert_eq!(tree.roots.len(), 4);

    // First root: BOM-managed spring-boot-starter-web
    let web = &tree.roots[0];
    assert_eq!(web.group, "org.springframework.boot");
    assert_eq!(web.artifact, "spring-boot-starter-web");
    assert_eq!(web.requested_version, "3.5.11");
    assert!(web.resolved_version.is_none());
    assert_eq!(web.children.len(), 3); // starter, starter-json, spring-webmvc

    // Second root: BOM-managed spring-boot-starter-data-jpa
    let jpa = &tree.roots[1];
    assert_eq!(jpa.artifact, "spring-boot-starter-data-jpa");
    assert_eq!(jpa.requested_version, "3.5.11");
    assert_eq!(jpa.children.len(), 3); // starter (*), hibernate, spring-data-jpa

    // Third root: BOM with constraints
    let bom = &tree.roots[2];
    assert_eq!(bom.group, "software.amazon.awssdk");
    assert_eq!(bom.artifact, "bom");
    assert_eq!(bom.requested_version, "2.42.11");
    assert_eq!(bom.children.len(), 2);
    assert!(bom.children[0].is_constraint);
    assert!(bom.children[1].is_constraint);

    // Fourth root: regular dependency
    let testng = &tree.roots[3];
    assert_eq!(testng.artifact, "testng");
    assert_eq!(testng.children.len(), 1);
    assert_eq!(testng.children[0].artifact, "jcommander");

    // Verify conflicts from spring-data-jpa resolving spring versions
    let spring_conflicts: Vec<_> = tree
        .conflicts
        .iter()
        .filter(|c| c.coordinate.contains("spring-"))
        .collect();
    assert!(!spring_conflicts.is_empty());

    // Verify constraint nodes from jackson-bom
    let all_nodes = collect_all_nodes(&tree.roots);
    let constraint_count = all_nodes.iter().filter(|n| n.is_constraint).count();
    assert!(constraint_count >= 3); // jackson-annotations (c), jackson-core (c), jackson-databind (c)

    // Verify omitted nodes
    let omitted_count = all_nodes.iter().filter(|n| n.is_omitted).count();
    assert!(omitted_count > 0);

    // Verify total node count covers all parsed lines
    assert!(tree.total_node_count() > 40);
}

#[test]
fn empty_input_returns_empty_tree() {
    let tree = tree_parser::parse("", "test", GradleConfiguration::CompileClasspath);
    assert!(tree.roots.is_empty());
    assert!(tree.conflicts.is_empty());
}

// Helper to collect all nodes recursively
fn collect_all_nodes(
    nodes: &[gradle_dependency_check::dependency::models::DependencyNode],
) -> Vec<&gradle_dependency_check::dependency::models::DependencyNode> {
    let mut result = Vec::new();
    for node in nodes {
        result.push(node);
        result.extend(collect_all_nodes(&node.children));
    }
    result
}
