# Architecture

## System Overview

`gradle-dependency-check` follows a binary/library split:

- **`main.rs`** -- CLI wiring only. Parses args with clap, calls library functions, prints output. No business logic.
- **`lib.rs`** -- Re-exports all public modules. All logic lives here and is independently testable.

The boundary between Gradle execution and pure logic is defined by the `GradleRunner` trait, enabling test doubles.

## Module Responsibilities

### `dependency` -- Domain Models

Core types used throughout the codebase:

- `DependencyNode` -- tree node with group, artifact, version, children, conflict tracking
- `DependencyTree` -- project name, configuration, roots, collected conflicts
- `DependencyConflict` -- a version mismatch (requested vs resolved)
- `GradleConfiguration` -- enum of all Gradle configurations (compileClasspath, testImplementation, etc.)
- `GradleModule` -- module name and path
- `FlatDependencyEntry`, `ScopeValidationResult`, `DuplicateDependencyResult`, `DependencyDiffEntry` -- analysis output types
- `ReportFormat` -- `Text` or `Json`

### `parsing` -- Input Parsers

- **`tree_parser`** -- Stack-based parser for Gradle ASCII dependency tree output. Handles version conflicts (`->`), omitted nodes (`(*)`), and constraints (`(c)`). Uses `LazyLock<Regex>` for compiled patterns.
- **`project_list_parser`** -- Parses `gradle projects -q` output to discover submodules.
- **`build_file_parser`** -- Regex-based parser for `build.gradle` and `build.gradle.kts` files. Extracts dependency declarations with configuration, coordinates, and line numbers.

### `analysis` -- Pure Computation

All functions are stateless (no `self`, no side effects):

- **`tree_analysis`** -- Tree traversal utilities (collect all nodes, etc.)
- **`table_calculator`** -- Flattens a tree into `FlatDependencyEntry` with parent tracking and conflict flags
- **`diff_calculator`** -- Builds coordinate-to-version maps from two trees, computes added/removed/changed/unchanged entries
- **`scope_validator`** -- Matches nodes against a list of 34 known test libraries, reports any found in production configurations
- **`duplicate_detector`** -- Cross-module: finds same coordinate in multiple module subtrees. Within-module: parses build files for duplicate declarations.
- **`multi_module_assembler`** -- Wraps per-module trees as children of synthetic root nodes

### `report` -- Output Generators

Each report module has a `report()` function that takes analysis results and a `ReportFormat`, returning a `String`. No I/O.

- `conflict_report` -- groups conflicts by coordinate
- `table_report` -- flat listing with conflict and version info
- `scope_validation_report` -- lists test libraries with recommendations
- `duplicate_report` -- separates cross-module and within-module results
- `diff_report` -- prefixed entries (`+`/`-`/`~`/`=`) with summary counts
- `dot_export` -- Graphviz DOT format with colored nodes (red for conflicts)
- `tree_export` -- JSON serialization (serde) and Gradle text format; also handles import with auto-detection

### `runner` -- Gradle Execution

- **`GradleRunner` trait** -- defines `run_dependencies()`, `run_module_dependencies()`, `list_projects()`
- **`ProcessGradleRunner`** -- executes `./gradlew` via `std::process::Command`

### `error` -- Error Types

Defined with `thiserror`:

- `ParseError` -- parsing failures
- `RunnerError` -- Gradle execution failures (not found, exit code, launch failed)
- `ImportError` -- file import failures (unreadable, no deps, JSON parse)
- `ExportError` -- serialization failures

`main.rs` uses `anyhow` for error propagation.

## Data Flow

```
CLI args (clap)
  |
  v
load_tree() or resolve_tree()
  |-- ProcessGradleRunner.run_dependencies()  -->  ./gradlew dependencies -q
  |-- tree_parser::parse()                    -->  DependencyTree
  |-- OR tree_export::import_tree()           -->  DependencyTree (from file)
  |
  v
analysis (stateless functions)
  |-- table_calculator::flat_entries()
  |-- scope_validator::validate()
  |-- duplicate_detector::detect()
  |-- diff_calculator::diff()
  |
  v
report (String output)
  |-- conflict_report::report()
  |-- table_report::report()
  |-- etc.
  |
  v
println!() in main.rs
```

## Key Design Decisions

1. **Binary/library split** -- `main.rs` is purely CLI wiring. All logic is in the library, testable without process execution.

2. **Trait boundary for GradleRunner** -- the only side effect (subprocess execution) is behind a trait. Tests use factories instead of mocking the runner.

3. **Stateless calculators** -- all analysis functions take inputs and return outputs. No `self`, no mutation, no stored state.

4. **Regex with LazyLock** -- compiled regex patterns are initialized once and reused across calls.

5. **Auto-detect for diff inputs** -- `resolve_tree()` checks if the path is a directory (run Gradle) or file (import), supporting mixed comparisons.

6. **Report generators return String** -- no I/O in the library. `main.rs` handles all printing.

7. **Serde for JSON** -- domain models derive `Serialize`/`Deserialize` for JSON export/import.

## Rust to Swift Module Mapping

| Rust Module | Swift Equivalent |
|-------------|-----------------|
| `dependency::models` | `GradleDependencyVisualizerCore` |
| `parsing::tree_parser` | `GradleDependencyTreeParser` |
| `parsing::build_file_parser` | `BuildFileParser` |
| `analysis::scope_validator` | `ScopeValidationCalculator` |
| `analysis::duplicate_detector` | `DuplicateDependencyCalculator` |
| `analysis::diff_calculator` | `DiffCalculator` |
| `analysis::table_calculator` | `TableCalculator` |
| `report::*` | `*Generator` / `*Exporter` |
| `runner::gradle_runner` | `GradleRunner` protocol |
| `runner::process_runner` | `ProcessGradleRunner` |
