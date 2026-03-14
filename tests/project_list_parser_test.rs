mod support;

use gradle_dependency_check::parsing::project_list_parser;

#[test]
fn parses_standard_project_output() {
    let output = "\
> Task :projects

------------------------------------------------------------
Root project 'my-project'
------------------------------------------------------------

Root project 'my-project'
+--- Project ':app'
+--- Project ':core'
\\--- Project ':data'

To see a list of the tasks of a project, run gradlew <project-path>:tasks";

    let modules = project_list_parser::parse(output);

    assert_eq!(modules.len(), 3);
    assert_eq!(modules[0].name, "app");
    assert_eq!(modules[0].path, ":app");
    assert_eq!(modules[1].name, "core");
    assert_eq!(modules[1].path, ":core");
    assert_eq!(modules[2].name, "data");
    assert_eq!(modules[2].path, ":data");
}

#[test]
fn parses_nested_submodules() {
    let output = "\
Root project 'my-project'
+--- Project ':app'
+--- Project ':app:feature'
\\--- Project ':app:feature:login'";

    let modules = project_list_parser::parse(output);

    assert_eq!(modules.len(), 3);
    assert_eq!(modules[0].name, "app");
    assert_eq!(modules[0].path, ":app");
    assert_eq!(modules[1].name, "feature");
    assert_eq!(modules[1].path, ":app:feature");
    assert_eq!(modules[2].name, "login");
    assert_eq!(modules[2].path, ":app:feature:login");
}

#[test]
fn no_submodules_returns_empty() {
    let output = "\
> Task :projects

------------------------------------------------------------
Root project 'my-project'
------------------------------------------------------------

Root project 'my-project' - A simple Gradle project
No sub-projects

To see a list of the tasks of a project, run gradlew <project-path>:tasks";

    let modules = project_list_parser::parse(output);
    assert!(modules.is_empty());
}

#[test]
fn empty_output_returns_empty() {
    let modules = project_list_parser::parse("");
    assert!(modules.is_empty());
}
