# Duplicate Detection

## What

Detects duplicate dependencies at two levels: same `group:artifact` declared as a direct dependency across multiple modules (cross-module), and same `group:artifact` declared multiple times within a single `build.gradle(.kts)` file (within-module).

## How

### Data flow

```
duplicate_detector::detect(tree, project_path, modules)
  → detect_cross_module(tree)
      → finds synthetic module nodes (requested_version == "module")
      → groups direct children by coordinate across modules
      → flags version mismatches
  → detect_within_module(project_path, modules)
      → reads build.gradle(.kts) from filesystem
      → parses with build_file_parser::parse()
      → groups declarations by coordinate
  → combined results sorted by coordinate

duplicate_report::report() → text or JSON output
```

### CLI usage

```bash
gradle-dependency-check duplicates ./my-project
gradle-dependency-check duplicates ./my-project --format json
```

## Architecture

- **Cross-module** — uses existing dependency tree. Synthetic module nodes (from `multi_module_assembler`) have `requested_version == "module"`. Only compares direct children (root-level declarations), not transitives.
- **Within-module** — reads build files from disk. Derives path from `project_path` + module Gradle path (`:app:feature` → `app/feature/build.gradle`). Tries `.kts` first, falls back to `.gradle`.
- **Build file parser** — supports Groovy string (`implementation 'g:a:v'`), Kotlin DSL (`implementation("g:a:v")`), and Groovy map notation. Skips line and block comments.
- **Version mismatch** — cross-module results compare versions; mismatches get "Version mismatch — standardize" recommendation.

### File organization

```
src/analysis/duplicate_detector.rs           — cross-module + within-module detection
src/parsing/build_file_parser.rs             — regex-based build file parser
src/report/duplicate_report.rs               — text/JSON report generation
```

## Testing

- `duplicate_detector_test` — shared deps, unique deps, version mismatch, single-module exclusion, within-module duplicates
- `build_file_parser_test` — Groovy/Kotlin DSL, map notation, comments, line numbers
- `duplicate_report_test` — text/JSON output formatting
- `integration_test` — full pipeline: parse → assemble → detect
