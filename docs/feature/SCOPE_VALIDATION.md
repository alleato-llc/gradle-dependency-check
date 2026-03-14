# Scope Validation

## What

Detects test framework libraries that appear in production Gradle configurations and recommends moving them to test-scoped configurations.

## How

### Data flow

```
scope_validator::validate(tree)
  → checks tree.configuration is production scope
  → walks all nodes via tree_analysis::all_nodes()
  → matches group/artifact against 34 known test libraries
  → deduplicates by coordinate
  → returns Vec<ScopeValidationResult>

scope_validation_report::report() → text or JSON output
```

### CLI usage

```bash
gradle-dependency-check validate ./my-project
gradle-dependency-check validate ./my-project --format json
```

## Architecture

- **Production-only** — only scans production configurations (`compileClasspath`, `runtimeClasspath`, `implementation`, `runtimeOnly`, `compileOnly`, `api`). Test configurations are skipped.
- **34 test libraries** — recognizes JUnit 4/5, Mockito, TestNG, Spring Test, AssertJ, Hamcrest, PowerMock, WireMock, Testcontainers, Cucumber, Spock, Robolectric, ArchUnit, and more.
- **Matching strategies** — exact coordinate match (e.g., `junit:junit`) and group prefix match (e.g., `org.mockito:*`).
- **Deduplication** — same coordinate appearing multiple times is reported once.

### File organization

```
src/analysis/scope_validator.rs              — validation logic + test library list
src/report/scope_validation_report.rs        — text/JSON report generation
```

## Testing

- `scope_validator_test` — production config detection, test config exclusion, no-test-libraries case, sorted output
- `scope_validation_report_test` — text/JSON output formatting
