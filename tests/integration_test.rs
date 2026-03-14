mod support;

use gradle_dependency_check::analysis::{duplicate_detector, multi_module_assembler};
use gradle_dependency_check::dependency::models::*;
use gradle_dependency_check::parsing::tree_parser;

const APP_MODULE_OUTPUT: &str = "\
+--- org.springframework.boot:spring-boot-starter-web -> 3.5.11
|    +--- org.springframework.boot:spring-boot-starter:3.5.11
|    |    +--- org.springframework.boot:spring-boot:3.5.11
|    |    |    +--- org.springframework:spring-core:6.2.16
|    |    |    |    \\--- org.springframework:spring-jcl:6.2.16
|    |    |    \\--- org.springframework:spring-context:6.2.16
|    |    +--- org.springframework.boot:spring-boot-starter-logging:3.5.11
|    |    |    +--- ch.qos.logback:logback-classic:1.5.32
|    |    |    |    \\--- org.slf4j:slf4j-api:2.0.17
|    |    |    \\--- org.slf4j:jul-to-slf4j:2.0.17
|    |    \\--- org.yaml:snakeyaml:2.4
|    \\--- org.springframework.boot:spring-boot-starter-json:3.5.11
|         +--- com.fasterxml.jackson.core:jackson-databind:2.19.4
|         |    +--- com.fasterxml.jackson.core:jackson-annotations:2.19.4
|         |    \\--- com.fasterxml.jackson.core:jackson-core:2.19.4
|         \\--- com.fasterxml.jackson.datatype:jackson-datatype-jdk8:2.19.4
+--- com.google.guava:guava:31.1-jre
|    +--- com.google.guava:failureaccess:1.0.2
|    \\--- com.google.code.findbugs:jsr305:3.0.2
\\--- org.slf4j:slf4j-api:2.0.16 -> 2.0.17";

const CORE_MODULE_OUTPUT: &str = "\
+--- com.google.guava:guava:30.0-jre
|    +--- com.google.guava:failureaccess:1.0.2
|    \\--- com.google.code.findbugs:jsr305:3.0.2
+--- com.fasterxml.jackson.core:jackson-databind:2.18.0
|    +--- com.fasterxml.jackson.core:jackson-annotations:2.18.0
|    \\--- com.fasterxml.jackson.core:jackson-core:2.18.0
+--- org.apache.commons:commons-lang3:3.17.0
\\--- org.slf4j:slf4j-api:2.0.17";

const DATA_MODULE_OUTPUT: &str = "\
+--- com.google.guava:guava:30.0-jre
|    +--- com.google.guava:failureaccess:1.0.2
|    \\--- com.google.code.findbugs:jsr305:3.0.2
+--- org.springframework.data:spring-data-jpa:3.5.9
|    +--- org.springframework.data:spring-data-commons:3.5.9
|    \\--- org.springframework:spring-core:6.2.16
\\--- org.slf4j:slf4j-api:2.0.17";

fn parse_module(output: &str, module_name: &str) -> (GradleModule, DependencyTree) {
    let tree = tree_parser::parse(output, module_name, GradleConfiguration::CompileClasspath);
    let module = GradleModule {
        name: module_name.to_string(),
        path: format!(":{}", module_name),
    };
    (module, tree)
}

fn assemble_modules(module_trees: Vec<(GradleModule, DependencyTree)>) -> DependencyTree {
    multi_module_assembler::assemble("test-project", GradleConfiguration::CompileClasspath, module_trees)
}

// 1. Parse app + core, assemble, detect. Guava and slf4j-api should be cross-module duplicates.
//    jackson-databind is transitive in :app so NOT flagged.
#[test]
fn full_pipeline_detects_cross_module_duplicates() {
    let app = parse_module(APP_MODULE_OUTPUT, "app");
    let core = parse_module(CORE_MODULE_OUTPUT, "core");

    let assembled = assemble_modules(vec![app, core]);
    let results = duplicate_detector::detect_cross_module(&assembled);

    let coords: Vec<&str> = results.iter().map(|r| r.coordinate.as_str()).collect();

    assert!(coords.contains(&"com.google.guava:guava"), "guava should be cross-module duplicate");
    assert!(coords.contains(&"org.slf4j:slf4j-api"), "slf4j-api should be cross-module duplicate");

    // jackson-databind is transitive in :app (nested under spring-boot-starter-json),
    // so it should NOT be flagged as a cross-module duplicate
    assert!(
        !coords.contains(&"com.fasterxml.jackson.core:jackson-databind"),
        "jackson-databind is transitive in :app, should not be flagged"
    );

    for result in &results {
        assert_eq!(result.kind, DuplicateKind::CrossModule);
    }
}

// 2. Guava: app=31.1-jre vs core=30.0-jre should have version mismatch.
#[test]
fn full_pipeline_detects_version_mismatch() {
    let app = parse_module(APP_MODULE_OUTPUT, "app");
    let core = parse_module(CORE_MODULE_OUTPUT, "core");

    let assembled = assemble_modules(vec![app, core]);
    let results = duplicate_detector::detect_cross_module(&assembled);

    let guava = results
        .iter()
        .find(|r| r.coordinate == "com.google.guava:guava")
        .expect("guava should be detected as cross-module duplicate");

    assert!(guava.has_version_mismatch, "guava should have version mismatch");
    assert!(guava.recommendation.contains("mismatch"));

    // Verify the actual versions
    let app_version = guava.versions.get("app").expect("app version should exist");
    let core_version = guava.versions.get("core").expect("core version should exist");
    assert_eq!(app_version, "31.1-jre");
    assert_eq!(core_version, "30.0-jre");
}

// 3. Parse all 3 modules. Guava appears in all 3. slf4j-api in all 3.
#[test]
fn full_pipeline_three_modules_shared_dependency() {
    let app = parse_module(APP_MODULE_OUTPUT, "app");
    let core = parse_module(CORE_MODULE_OUTPUT, "core");
    let data = parse_module(DATA_MODULE_OUTPUT, "data");

    let assembled = assemble_modules(vec![app, core, data]);
    let results = duplicate_detector::detect_cross_module(&assembled);

    let guava = results
        .iter()
        .find(|r| r.coordinate == "com.google.guava:guava")
        .expect("guava should be detected");
    assert_eq!(guava.modules.len(), 3, "guava should appear in all 3 modules");

    let slf4j = results
        .iter()
        .find(|r| r.coordinate == "org.slf4j:slf4j-api")
        .expect("slf4j-api should be detected");
    assert_eq!(slf4j.modules.len(), 3, "slf4j-api should appear in all 3 modules");
}

// 4. Two modules with no shared direct deps -> empty results.
#[test]
fn full_pipeline_no_duplicates_for_unique_modules() {
    let unique_a_output = "\
+--- com.example:lib-a:1.0.0
\\--- com.example:lib-a2:2.0.0";

    let unique_b_output = "\
+--- com.other:lib-b:1.0.0
\\--- com.other:lib-b2:3.0.0";

    let a = parse_module(unique_a_output, "module-a");
    let b = parse_module(unique_b_output, "module-b");

    let assembled = assemble_modules(vec![a, b]);
    let results = duplicate_detector::detect_cross_module(&assembled);

    assert!(results.is_empty(), "no shared deps means no cross-module duplicates");
}

// 5. Create temp dir with build.gradle containing duplicate guava declarations, detect within-module.
#[test]
fn full_pipeline_within_module_duplicates() {
    let dir = tempfile::tempdir().unwrap();
    let build_content = "\
dependencies {
    implementation 'com.google.guava:guava:31.1-jre'
    testImplementation 'com.google.guava:guava:31.1-jre'
}";
    std::fs::write(dir.path().join("build.gradle"), build_content).unwrap();

    let results = duplicate_detector::detect_within_module(dir.path().to_str().unwrap(), &[]);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].coordinate, "com.google.guava:guava");
    assert_eq!(results[0].kind, DuplicateKind::WithinModule);
    assert!(
        results[0].recommendation.contains("2 times"),
        "recommendation should mention count"
    );
}

// 6. Both cross-module and within-module results from same detect() call.
#[test]
fn full_pipeline_combined_detection() {
    let app = parse_module(APP_MODULE_OUTPUT, "app");
    let core = parse_module(CORE_MODULE_OUTPUT, "core");

    let assembled = assemble_modules(vec![app, core]);

    // Set up a temp dir with a build.gradle containing duplicates
    let dir = tempfile::tempdir().unwrap();
    let build_content = "\
dependencies {
    implementation 'com.example:some-lib:1.0.0'
    testImplementation 'com.example:some-lib:1.0.0'
}";
    std::fs::write(dir.path().join("build.gradle"), build_content).unwrap();

    let results = duplicate_detector::detect(&assembled, dir.path().to_str().unwrap(), &[]);

    let cross_module: Vec<_> = results
        .iter()
        .filter(|r| r.kind == DuplicateKind::CrossModule)
        .collect();
    let within_module: Vec<_> = results
        .iter()
        .filter(|r| r.kind == DuplicateKind::WithinModule)
        .collect();

    assert!(!cross_module.is_empty(), "should have cross-module results");
    assert!(!within_module.is_empty(), "should have within-module results");
}

// 7. Single-module tree has no synthetic nodes -> no cross-module results.
#[test]
fn full_pipeline_single_module_skips_cross_module() {
    let app = parse_module(APP_MODULE_OUTPUT, "app");

    let assembled = assemble_modules(vec![app]);
    let results = duplicate_detector::detect_cross_module(&assembled);

    assert!(
        results.is_empty(),
        "single module should produce no cross-module duplicates"
    );
}

// 8. build.gradle.kts with Kotlin DSL syntax for within-module detection.
#[test]
fn kotlin_dsl_build_file_integration() {
    let dir = tempfile::tempdir().unwrap();
    let build_content = "\
dependencies {
    implementation(\"com.google.guava:guava:31.1-jre\")
    testImplementation(\"com.google.guava:guava:31.1-jre\")
}";
    std::fs::write(dir.path().join("build.gradle.kts"), build_content).unwrap();

    let results = duplicate_detector::detect_within_module(dir.path().to_str().unwrap(), &[]);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].coordinate, "com.google.guava:guava");
    assert_eq!(results[0].kind, DuplicateKind::WithinModule);
}

// 9. Comments should not produce false positives.
#[test]
fn build_file_with_comments_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let build_content = "\
dependencies {
    // implementation 'com.google.guava:guava:31.1-jre'
    implementation 'com.google.guava:guava:31.1-jre'
    /* implementation 'com.google.guava:guava:31.1-jre' */
}";
    std::fs::write(dir.path().join("build.gradle"), build_content).unwrap();

    let results = duplicate_detector::detect_within_module(dir.path().to_str().unwrap(), &[]);

    assert!(
        results.is_empty(),
        "commented-out lines should not produce false positives"
    );
}
