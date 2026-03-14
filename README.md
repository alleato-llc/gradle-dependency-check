# gradle-dependency-check

CI-focused Gradle dependency analysis tool. Rust port of the [gradle-visualizer](https://github.com/nycjv321/gradle-visualizer) Swift CLI, designed for automated pipelines.

## Quick Start

```bash
cargo build              # Build the binary
cargo test               # Run all tests (40 tests)
cargo install --path .   # Install to ~/.cargo/bin
```

## Subcommands

### `graph` -- DOT export

Output the dependency tree in Graphviz DOT format. Pipe to `dot` for rendering.

```bash
gradle-dependency-check graph ./my-project
gradle-dependency-check graph ./my-project -c runtimeClasspath -m :app
gradle-dependency-check graph ./my-project | dot -Tpng -o deps.png
```

### `conflicts` -- version conflict report

Show dependencies where the resolved version differs from the requested version.

```bash
gradle-dependency-check conflicts ./my-project
gradle-dependency-check conflicts ./my-project -f json
```

### `table` -- flat dependency listing

List all unique dependencies in a flat table with version and conflict info.

```bash
gradle-dependency-check table ./my-project
gradle-dependency-check table ./my-project --conflicts-only
gradle-dependency-check table ./my-project -f json
```

### `validate` -- scope validation

Detect test libraries (JUnit, Mockito, Testcontainers, etc.) in production scopes.

```bash
gradle-dependency-check validate ./my-project
gradle-dependency-check validate ./my-project -f json
```

### `duplicates` -- duplicate detection

Find dependencies declared in multiple modules or duplicated within a single build file.

```bash
gradle-dependency-check duplicates ./my-project
gradle-dependency-check duplicates ./my-project -f json
```

### `diff` -- dependency comparison

Compare two dependency trees. Each argument can be a Gradle project directory or an exported file (JSON or text).

```bash
gradle-dependency-check diff ./baseline ./current
gradle-dependency-check diff baseline.json current.json
gradle-dependency-check diff ./my-project snapshot.json --changes added,removed
```

### `export` -- tree serialization

Export the dependency tree as JSON or Gradle text format for later diffing or archival.

```bash
gradle-dependency-check export ./my-project -f json > deps.json
gradle-dependency-check export ./my-project -f text > deps.txt
```

## Shared Options

All subcommands (except `diff`) accept:

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--configuration` | `-c` | `compileClasspath` | Gradle configuration to analyze |
| `--module` | `-m` | all modules | Target a specific module (e.g. `:app`) |
| `--list-modules` | | | List discovered modules and exit |
| `--format` | `-f` | `text` | Output format: `text` or `json` |

The `diff` subcommand accepts `--configuration`, `--module`, `--format`, and `--changes` (filter by `added`, `removed`, `changed`, `unchanged`).

## Example Workflows

### CI: detect dependency drift

```bash
# On main branch, export a baseline
gradle-dependency-check export ./my-project -f json > baseline.json

# On PR branch, diff against baseline
gradle-dependency-check diff baseline.json ./my-project --changes added,removed,changed -f json
```

### CI: fail on test libs in production

```bash
gradle-dependency-check validate ./my-project -f json | jq '.issueCount'
# Non-zero means test libraries leaked into production scopes
```

### CI: detect cross-module version mismatches

```bash
gradle-dependency-check duplicates ./my-project -f json \
  | jq '[.duplicates[] | select(.hasVersionMismatch)] | length'
```

## Relationship to Swift Project

This is the CI companion to `gradle-visualizer`, a macOS SwiftUI app for interactive visualization:

- **Same parsing logic** -- stack-based tree parser, regex build file parser
- **Same analysis algorithms** -- scope validation, duplicate detection, diff
- **Same report formats** -- text/JSON output matches Swift generators
- **Same CLI subcommands** -- identical argument structure

Use `gradle-visualizer` for interactive exploration and `gradle-dependency-check` in CI pipelines.

## Architecture

```
src/
  main.rs                    # CLI wiring (clap derive), no business logic
  lib.rs                     # Public module re-exports
  dependency/
    models.rs                # Domain types (DependencyNode, DependencyTree, etc.)
  parsing/
    tree_parser.rs           # Stack-based Gradle ASCII tree parser
    project_list_parser.rs   # Parses `gradle projects` output
    build_file_parser.rs     # Regex parser for build.gradle(.kts) declarations
  analysis/
    tree_analysis.rs         # Tree traversal utilities
    table_calculator.rs      # Flattens tree into FlatDependencyEntry list
    diff_calculator.rs       # Compares two trees, produces DependencyDiffResult
    scope_validator.rs       # Detects test libraries in production scopes
    duplicate_detector.rs    # Cross-module and within-module duplicate detection
    multi_module_assembler.rs # Assembles per-module trees into one tree
  report/
    conflict_report.rs       # Text/JSON conflict reports
    table_report.rs          # Text/JSON table reports
    scope_validation_report.rs
    duplicate_report.rs
    diff_report.rs
    dot_export.rs            # Graphviz DOT format export
    tree_export.rs           # JSON/text export and import
  runner/
    gradle_runner.rs         # GradleRunner trait (test boundary)
    process_runner.rs        # ProcessGradleRunner (executes ./gradlew)
  error.rs                   # Error enums (ParseError, RunnerError, etc.)
tests/
  support/
    factories.rs             # Test data builders (simple_tree, tree_with_conflicts, etc.)
  tree_parser_test.rs
  build_file_parser_test.rs
  diff_calculator_test.rs
  duplicate_detector_test.rs
  scope_validator_test.rs
```

## License

MIT
