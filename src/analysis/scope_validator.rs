use std::collections::HashSet;

use crate::analysis::tree_analysis;
use crate::dependency::models::{DependencyTree, ScopeValidationResult};

struct TestLibrary {
    group: &'static str,
    artifact: Option<&'static str>,
    name: &'static str,
}

const TEST_LIBRARIES: &[TestLibrary] = &[
    TestLibrary { group: "junit", artifact: Some("junit"), name: "JUnit 4" },
    TestLibrary { group: "org.junit.jupiter", artifact: None, name: "JUnit 5" },
    TestLibrary { group: "org.junit.vintage", artifact: None, name: "JUnit 5 Vintage" },
    TestLibrary { group: "org.junit.platform", artifact: None, name: "JUnit Platform" },
    TestLibrary { group: "org.testng", artifact: Some("testng"), name: "TestNG" },
    TestLibrary { group: "org.springframework", artifact: Some("spring-test"), name: "Spring Test" },
    TestLibrary { group: "org.springframework.boot", artifact: Some("spring-boot-test"), name: "Spring Boot Test" },
    TestLibrary { group: "org.springframework.boot", artifact: Some("spring-boot-starter-test"), name: "Spring Boot Starter Test" },
    TestLibrary { group: "org.mockito", artifact: None, name: "Mockito" },
    TestLibrary { group: "io.mockk", artifact: Some("mockk"), name: "MockK" },
    TestLibrary { group: "io.mockk", artifact: Some("mockk-jvm"), name: "MockK" },
    TestLibrary { group: "org.assertj", artifact: Some("assertj-core"), name: "AssertJ" },
    TestLibrary { group: "org.hamcrest", artifact: Some("hamcrest"), name: "Hamcrest" },
    TestLibrary { group: "org.hamcrest", artifact: Some("hamcrest-core"), name: "Hamcrest" },
    TestLibrary { group: "org.easymock", artifact: Some("easymock"), name: "EasyMock" },
    TestLibrary { group: "org.powermock", artifact: None, name: "PowerMock" },
    TestLibrary { group: "com.github.tomakehurst", artifact: Some("wiremock"), name: "WireMock" },
    TestLibrary { group: "org.wiremock", artifact: Some("wiremock"), name: "WireMock" },
    TestLibrary { group: "org.jboss.arquillian", artifact: None, name: "Arquillian" },
    TestLibrary { group: "io.rest-assured", artifact: None, name: "REST Assured" },
    TestLibrary { group: "org.awaitility", artifact: Some("awaitility"), name: "Awaitility" },
    TestLibrary { group: "org.testcontainers", artifact: None, name: "Testcontainers" },
    TestLibrary { group: "io.cucumber", artifact: None, name: "Cucumber" },
    TestLibrary { group: "org.spockframework", artifact: None, name: "Spock" },
    TestLibrary { group: "org.jmockit", artifact: Some("jmockit"), name: "JMockit" },
    TestLibrary { group: "com.google.truth", artifact: Some("truth"), name: "Google Truth" },
    TestLibrary { group: "net.javacrumbs.json-unit", artifact: None, name: "JsonUnit" },
    TestLibrary { group: "org.xmlunit", artifact: None, name: "XMLUnit" },
    TestLibrary { group: "org.dbunit", artifact: Some("dbunit"), name: "DbUnit" },
    TestLibrary { group: "com.codeborne", artifact: Some("selenide"), name: "Selenide" },
    TestLibrary { group: "org.seleniumhq.selenium", artifact: None, name: "Selenium" },
    TestLibrary { group: "org.robolectric", artifact: None, name: "Robolectric" },
    TestLibrary { group: "com.tngtech.archunit", artifact: None, name: "ArchUnit" },
    TestLibrary { group: "org.pitest", artifact: None, name: "Pitest" },
];

pub fn validate(tree: &DependencyTree) -> Vec<ScopeValidationResult> {
    if !tree.configuration.is_production() {
        return Vec::new();
    }

    let all_nodes = tree_analysis::all_nodes(tree);
    let mut results = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for node in all_nodes {
        let coord = node.coordinate();
        if seen.contains(&coord) {
            continue;
        }

        if let Some(library_name) = match_test_library(&node.group, &node.artifact) {
            seen.insert(coord.clone());
            results.push(ScopeValidationResult {
                coordinate: coord,
                version: node
                    .resolved_version
                    .as_deref()
                    .unwrap_or(&node.requested_version)
                    .to_string(),
                matched_library: library_name.to_string(),
                configuration: tree.configuration,
                recommendation: "Move to testImplementation or testRuntimeOnly".to_string(),
            });
        }
    }

    results.sort_by(|a, b| a.coordinate.cmp(&b.coordinate));
    results
}

fn match_test_library(group: &str, artifact: &str) -> Option<&'static str> {
    for lib in TEST_LIBRARIES {
        if let Some(expected_artifact) = lib.artifact {
            if group == lib.group && artifact == expected_artifact {
                return Some(lib.name);
            }
        } else if group == lib.group || group.starts_with(&format!("{}.", lib.group)) {
            return Some(lib.name);
        }
    }
    None
}
