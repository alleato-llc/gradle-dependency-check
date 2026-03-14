# Testing

## Strategy

- **Unit tests for calculators and parsers** -- all analysis and parsing modules have dedicated test files
- **Integration tests via factories** -- test files in `tests/` use factory helpers to construct realistic domain objects
- **No subprocess tests** -- `ProcessGradleRunner` is not tested directly; the `GradleRunner` trait boundary keeps Gradle execution out of the test suite

## Running Tests

```bash
cargo test                         # Run all 40 tests
cargo test tree_parser             # Run tree parser tests only
cargo test -- --nocapture          # Show println output
```

## Test Infrastructure

### Factory Module (`tests/support/factories.rs`)

Provides pre-built domain objects for tests:

| Factory | Description |
|---------|-------------|
| `simple_tree()` | Single root with 2 leaf dependencies |
| `multi_module_tree(mismatch)` | Two modules with shared dependency, optional version mismatch |
| `tree_with_test_libraries()` | Tree containing JUnit, Mockito, Jupiter in production scope |
| `tree_with_conflicts()` | Tree with jackson-databind version conflict |
| `module(name)` | Creates a `GradleModule` with `:name` path |
| `node(group, artifact, version)` | Creates a single `DependencyNode` |

### Temp Files

The `tempfile` dev-dependency is available for tests that need filesystem fixtures (e.g., build file parsing with within-module duplicate detection).

## Test File Inventory

| Test File | Module Under Test | Test Count |
|-----------|------------------|------------|
| `tree_parser_test.rs` | `parsing::tree_parser` | 7 |
| `build_file_parser_test.rs` | `parsing::build_file_parser` | 9 |
| `diff_calculator_test.rs` | `analysis::diff_calculator` | 5 |
| `duplicate_detector_test.rs` | `analysis::duplicate_detector` | 7 |
| `scope_validator_test.rs` | `analysis::scope_validator` | 4 |
| Inline unit tests (dot_export, conflict_report) | `report::*` | 8 |
| **Total** | | **40** |

## Conventions

- **One test per behavior** -- each test validates a single scenario (e.g., `parses_version_conflict`, `added_dependency_detected`)
- **Descriptive names** -- test function names describe the scenario, not the method (e.g., `cross_module_version_mismatch` not `test_detect`)
- **No shared state** -- each test constructs its own data via factories or inline setup
- **Sorted results** -- analysis functions sort output by coordinate, so tests can assert on stable ordering
