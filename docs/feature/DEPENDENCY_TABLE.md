# Dependency Table

## What

Flattens the dependency tree into a deduplicated list showing each unique dependency with its version, conflict status, occurrence count, and parent dependencies.

## How

### Data flow

```
table_calculator::flat_entries(tree)
  → collects all nodes grouped by coordinate
  → picks preferred node (non-omitted over omitted)
  → aggregates versions, parents, occurrence counts
  → detects conflicts from tree.conflicts and node properties
  → returns Vec<FlatDependencyEntry> sorted by coordinate

table_report::report() → text or JSON output
```

### CLI usage

```bash
gradle-dependency-check table ./my-project
gradle-dependency-check table ./my-project --format json
gradle-dependency-check table ./my-project --conflicts-only
```

## Architecture

- **Deduplication** — each `group:artifact` appears once regardless of how many times it occurs in the tree.
- **Preferred node** — when the same coordinate appears as both omitted and non-omitted, the non-omitted version is used for display.
- **Parent tracking** — `parent_map()` builds a reverse lookup of which coordinates depend on each coordinate.
- **Conflicts-only filter** — `--conflicts-only` retains only entries with version conflicts, useful for CI gates.

### Key types

- `FlatDependencyEntry` — `coordinate`, `version`, `has_conflict`, `occurrence_count`, `used_by`, `versions`
- `table_calculator::parent_map()` — `HashMap<coordinate, HashSet<parent_coordinate>>`

### File organization

```
src/analysis/table_calculator.rs     — flat entry generation + parent mapping
src/report/table_report.rs           — text/JSON report generation
```

## Testing

- `table_calculator_test` — flat entries, conflict tracking, parent mapping, version aggregation, sorting
