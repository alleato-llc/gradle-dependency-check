use gradle_dependency_check::dependency::models::*;

pub fn simple_tree() -> DependencyTree {
    let leaf_a = DependencyNode::new("com.google.guava", "guava", "31.1-jre");
    let leaf_b = DependencyNode::new("org.slf4j", "slf4j-api", "1.7.36");
    let mut root = DependencyNode::new("org.springframework", "spring-core", "5.3.20");
    root.children = vec![leaf_a, leaf_b];

    DependencyTree {
        project_name: "test-project".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![root],
        conflicts: vec![],
    }
}

pub fn multi_module_tree(version_mismatch: bool) -> DependencyTree {
    let shared_a = DependencyNode::new("com.google.guava", "guava", "31.1-jre");
    let unique_a = DependencyNode::new("com.example", "lib-a-only", "1.0.0");
    let mut module_a = DependencyNode::new("test-project", "app", "module");
    module_a.children = vec![shared_a, unique_a];

    let shared_b_version = if version_mismatch { "30.0-jre" } else { "31.1-jre" };
    let shared_b = DependencyNode::new("com.google.guava", "guava", shared_b_version);
    let unique_b = DependencyNode::new("com.example", "lib-b-only", "2.0.0");
    let mut module_b = DependencyNode::new("test-project", "core", "module");
    module_b.children = vec![shared_b, unique_b];

    DependencyTree {
        project_name: "test-project".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![module_a, module_b],
        conflicts: vec![],
    }
}

pub fn tree_with_test_libraries() -> DependencyTree {
    let junit = DependencyNode::new("junit", "junit", "4.13.2");
    let mockito = DependencyNode::new("org.mockito", "mockito-core", "5.3.1");
    let jupiter = DependencyNode::new("org.junit.jupiter", "junit-jupiter-api", "5.9.3");
    let guava = DependencyNode::new("com.google.guava", "guava", "31.1-jre");
    let mut root = DependencyNode::new("com.example", "my-app", "1.0.0");
    root.children = vec![junit, mockito, jupiter, guava];

    DependencyTree {
        project_name: "test-project".to_string(),
        configuration: GradleConfiguration::CompileClasspath,
        roots: vec![root],
        conflicts: vec![],
    }
}

pub fn tree_with_conflicts() -> DependencyTree {
    let mut conflict_node =
        DependencyNode::new("com.fasterxml.jackson.core", "jackson-databind", "2.13.0");
    conflict_node.resolved_version = Some("2.14.2".to_string());

    let mut spring_web =
        DependencyNode::new("org.springframework", "spring-web", "5.3.20");
    spring_web.children = vec![conflict_node];

    let direct_jackson =
        DependencyNode::new("com.fasterxml.jackson.core", "jackson-databind", "2.14.2");

    let conflict = DependencyConflict {
        coordinate: "com.fasterxml.jackson.core:jackson-databind".to_string(),
        requested_version: "2.13.0".to_string(),
        resolved_version: "2.14.2".to_string(),
        requested_by: "org.springframework:spring-web".to_string(),
        risk_level: None,
        risk_reason: None,
    };

    DependencyTree {
        project_name: "test-project".to_string(),
        configuration: GradleConfiguration::RuntimeClasspath,
        roots: vec![spring_web, direct_jackson],
        conflicts: vec![conflict],
    }
}

pub fn module(name: &str) -> GradleModule {
    GradleModule {
        name: name.to_string(),
        path: format!(":{}", name),
    }
}

pub fn node(group: &str, artifact: &str, version: &str) -> DependencyNode {
    DependencyNode::new(group, artifact, version)
}
