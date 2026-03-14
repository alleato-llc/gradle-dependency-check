# Conflict Detection

## What

Detects Gradle dependency version conflicts — where a transitive dependency requests one version but Gradle resolves a different version.

## How

### Data flow

```
tree_parser::parse()
  → detects " -> " in dependency line
  → creates DependencyNode with resolved_version set
  → creates DependencyConflict with parent tracking
  → DependencyTree.conflicts populated

conflict_report::report() → text or JSON output
```

### CLI usage

```bash
gradle-dependency-check conflicts ./my-project
gradle-dependency-check conflicts ./my-project --format json
```

## Architecture

- **Inline detection** — conflicts are detected during ASCII tree parsing, not in a separate pass. The parser tracks the parent via the stack to populate `requested_by`.
- **Dual representation** — conflicts exist both as `DependencyNode.has_conflict()` (for DOT coloring) and `DependencyConflict` records (for reports).
- **Grouped output** — text report groups conflicts by coordinate, showing each version resolution and requesting parent.

### Key types

- `DependencyConflict` — `coordinate`, `requested_version`, `resolved_version`, `requested_by`
- `conflict_report::report()` — text and JSON formatting

### File organization

```
src/parsing/tree_parser.rs          — conflict detection during parse
src/report/conflict_report.rs       — text/JSON report generation
src/analysis/tree_analysis.rs       — conflicts_by_coordinate() grouping
```

## Testing

- `tree_parser_test` — conflict marker parsing, parent tracking, version extraction
- `conflict_report_test` — text/JSON output, empty/populated conflicts
- `tree_analysis_test` — conflict grouping by coordinate
