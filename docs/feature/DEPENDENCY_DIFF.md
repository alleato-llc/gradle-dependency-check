# Dependency Diff

## What

Compares two dependency trees and classifies each dependency as added, removed, version-changed, or unchanged. Each side of the comparison can be a live Gradle project or an exported file.

## How

### Data flow

```
resolve_tree(path)
  → directory? → tree_loader::load_tree() via GradleRunner
  → file? → tree_export::import_tree() (auto-detects JSON or text)

diff_calculator::diff(baseline, current)
  → builds coordinate-to-version summary maps
  → classifies: Added, Removed, VersionChanged, Unchanged
  → returns DependencyDiffResult with sorted entries

diff_report::report() → text or JSON output
```

### CLI usage

```bash
# Two exported files
gradle-dependency-check diff baseline.json current.json

# File vs live project
gradle-dependency-check diff baseline.json ./my-project

# Two live projects
gradle-dependency-check diff ./old-project ./new-project

# Filter to only additions and removals
gradle-dependency-check diff baseline.json current.json --changes added,removed

# JSON output
gradle-dependency-check diff baseline.json current.json --format json
```

## Architecture

- **Auto-detection** — each argument is resolved dynamically: directories run Gradle, files are imported via `TreeImporter` (tries JSON first, falls back to text).
- **Summary maps** — each tree is reduced to a `HashMap<coordinate, (requested, resolved)>`. Prefers non-omitted, non-constraint nodes when a coordinate appears multiple times.
- **Default filtering** — unchanged entries are hidden by default. Use `--changes unchanged` to include them.
- **Text symbols** — `+` added, `-` removed, `~` version changed, `=` unchanged.

### File organization

```
src/analysis/diff_calculator.rs      — comparison logic
src/report/diff_report.rs            — text/JSON report generation
src/report/tree_export.rs            — import_tree() for auto-detection
```

## Testing

- `diff_calculator_test` — identical trees, added/removed/changed detection, sorted output
- `diff_report_test` — text symbols, JSON fields
- `tree_export_test` — JSON/text import, auto-detection, round-trips
