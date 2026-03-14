# Gradle Dependency Check

Rust CLI that analyzes Gradle dependency trees — CI-focused port of the Swift gradle-visualizer CLI.

## Build Commands

```bash
cargo build                # Build
cargo test                 # Run all tests (118 tests)
cargo run -- <subcommand>  # Run CLI
```

## Architecture

- **Pattern**: Binary/library split, trait-based DI, stateless calculators
- **Error handling**: `thiserror` in library, `anyhow` in main.rs
- **CLI**: `clap` derive macros with 7 subcommands

## Module Structure

- **dependency** — Domain models (`DependencyNode`, `DependencyTree`, `GradleConfiguration`, etc.)
- **parsing** — Gradle output parsers (tree, project list, build file)
- **analysis** — Pure computation (scope validation, duplicate detection, diff, table, multi-module assembly, tree loading)
- **report** — Text/JSON report generators (conflict, table, scope, duplicate, diff, DOT, tree export/import)
- **runner** — Gradle execution boundary (`GradleRunner` trait + `ProcessGradleRunner` impl)

## CLI Subcommands

| Command | Description |
|---------|-------------|
| `graph <path>` | Output dependency tree in DOT format |
| `conflicts <path>` | Report dependency conflicts |
| `table <path>` | List dependencies in flat table format |
| `validate <path>` | Check for test libraries in production scopes |
| `duplicates <path>` | Detect duplicate dependencies across/within modules |
| `diff <baseline> <current>` | Compare two trees (file or project directory) |
| `export <path>` | Export tree as JSON or Gradle text format |

## Shared Options

- `--configuration, -c` — Gradle configuration (default: `compileClasspath`)
- `--module, -m` — Specific module (e.g., `:app`)
- `--list-modules` — List discovered modules and exit
- `--format, -f` — Output format: `text` or `json`

## Key Design Decisions

- `main.rs` is purely CLI wiring — all logic in the library
- `GradleRunner` trait enables test doubles for Gradle execution
- Parsers use `regex` with `LazyLock` for compiled patterns
- All calculators are stateless (associated functions, no `self`)
- Report generators produce `String` output, no I/O
- `diff` command auto-detects input type: directory → run Gradle, file → import

## Documentation

- [Architecture](docs/ARCHITECTURE.md) — System design, modules, data flow
- [Testing](docs/TESTING.md) — Test strategy, infrastructure, conventions
- [CLI Commands](docs/feature/CLI_COMMANDS.md) — All 7 subcommands with examples
- [Dependency Analysis](docs/feature/DEPENDENCY_ANALYSIS.md) — Core analysis features

## Skills

Skills in `.claude/skills/` (from rust-cli-reference):
- **project-structure** — Binary/library split, domain-oriented modules
- **component-design** — Calculators, services, method size constraints
- **inversion-of-control** — `GradleRunner` trait, `Box<dyn Trait>`, constructor injection
- **error-handling** — `thiserror` in lib, `anyhow` in main
- **adding-unit-tests** — Tests in `tests/`, helpers in `tests/support/`
- **test-data-isolation** — Fresh state per test, `tempfile` for filesystem
- **testing-boundaries** — `TestGradleRunner` fake with RefCell call capture
- **adding-integration-tests** — `tree_loader_test.rs` with test doubles, `integration_test.rs` for pipeline
- **project-documentation** — README, CONTRIBUTING, docs/

## Relationship to Swift Project

This is the CI companion to `gradle-visualizer` (Swift macOS app + CLI):
- Same parsing logic (stack-based tree parser, regex build file parser)
- Same analysis algorithms (scope validation, duplicate detection, diff)
- Same report formats (text/JSON output matches Swift generators)
- Same CLI subcommands and argument structure
