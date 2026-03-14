use gradle_dependency_check::parsing::build_file_parser;

#[test]
fn parses_groovy_single_quote() {
    let content = "implementation 'com.google.guava:guava:31.1-jre'";
    let results = build_file_parser::parse(content);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].configuration, "implementation");
    assert_eq!(results[0].group, "com.google.guava");
    assert_eq!(results[0].artifact, "guava");
    assert_eq!(results[0].version, "31.1-jre");
}

#[test]
fn parses_groovy_double_quote() {
    let content = r#"implementation "org.springframework:spring-core:5.3.20""#;
    let results = build_file_parser::parse(content);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].group, "org.springframework");
}

#[test]
fn parses_kotlin_dsl() {
    let content = r#"implementation("com.google.guava:guava:31.1-jre")"#;
    let results = build_file_parser::parse(content);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].group, "com.google.guava");
}

#[test]
fn parses_groovy_map_notation() {
    let content = "implementation group: 'com.google.guava', name: 'guava', version: '31.1-jre'";
    let results = build_file_parser::parse(content);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].group, "com.google.guava");
}

#[test]
fn parses_multiple_configurations() {
    let content = "\
implementation 'com.google.guava:guava:31.1-jre'
testImplementation 'junit:junit:4.13.2'
api 'org.slf4j:slf4j-api:1.7.36'
compileOnly 'org.projectlombok:lombok:1.18.24'
runtimeOnly 'mysql:mysql-connector-java:8.0.30'
annotationProcessor 'org.projectlombok:lombok:1.18.24'";
    let results = build_file_parser::parse(content);
    assert_eq!(results.len(), 6);
}

#[test]
fn tracks_line_numbers() {
    let content = "\
plugins {
    id 'java'
}

dependencies {
    implementation 'com.google.guava:guava:31.1-jre'
    testImplementation 'junit:junit:4.13.2'
}";
    let results = build_file_parser::parse(content);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].line, 6);
    assert_eq!(results[1].line, 7);
}

#[test]
fn ignores_line_comments() {
    let content = "\
// implementation 'com.google.guava:guava:31.1-jre'
implementation 'org.slf4j:slf4j-api:1.7.36'";
    let results = build_file_parser::parse(content);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].group, "org.slf4j");
}

#[test]
fn ignores_block_comments() {
    let content = "\
/*
implementation 'com.google.guava:guava:31.1-jre'
*/
implementation 'org.slf4j:slf4j-api:1.7.36'";
    let results = build_file_parser::parse(content);
    assert_eq!(results.len(), 1);
}

#[test]
fn empty_content_returns_empty() {
    let results = build_file_parser::parse("");
    assert!(results.is_empty());
}
