# Contributing

## Development Setup

1. Install Rust via [rustup](https://rustup.rs/)
2. Clone the repository
3. Build and test:

```bash
cargo build
cargo test
```

No other dependencies are required. The project uses Rust 2024 edition.

## Running Tests

```bash
cargo test                   # All tests
cargo test tree_parser       # Filter by name
cargo test -- --nocapture    # Show stdout
cargo clippy                 # Lint
```

## Project Structure

- `src/main.rs` -- CLI entry point (clap). No business logic.
- `src/lib.rs` -- Library root. All logic lives here.
- `src/dependency/models.rs` -- Domain types.
- `src/parsing/` -- Input parsers (tree, project list, build file).
- `src/analysis/` -- Stateless analysis functions.
- `src/report/` -- Text/JSON report generators.
- `src/runner/` -- `GradleRunner` trait and `ProcessGradleRunner` impl.
- `src/error.rs` -- Error types (`thiserror`).
- `tests/` -- Integration tests with factory helpers.

## Adding New Features

### New analysis feature

1. Add a calculator/detector in `src/analysis/` as a module with public functions (no `self`, stateless)
2. Add the corresponding report generator in `src/report/` with `report(results, tree, format) -> String`
3. Add a subcommand variant to `Commands` in `main.rs`
4. Add test file in `tests/` using factories from `tests/support/factories.rs`
5. Re-export the module in `src/analysis/mod.rs` and `src/report/mod.rs`

### New domain type

1. Add the struct/enum in `src/dependency/models.rs`
2. Derive `Serialize` (and `Deserialize` if needed for import)
3. Use `#[serde(rename_all = "camelCase")]` for JSON field naming

### New test factory

Add a function to `tests/support/factories.rs` that returns the needed domain object with sensible defaults.

## Code Style

- Follow existing patterns -- look at neighboring modules for conventions
- Run `cargo clippy` before submitting
- All analysis functions are stateless: take inputs, return outputs
- Report generators return `String`, no I/O
- Use `thiserror` for library errors, `anyhow` only in `main.rs`
- Use `LazyLock<Regex>` for compiled regex patterns
- Sort results by coordinate for stable output
