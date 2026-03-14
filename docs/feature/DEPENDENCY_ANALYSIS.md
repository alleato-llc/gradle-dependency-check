# Dependency Analysis

Core analysis features implemented in `src/analysis/`.

## Conflict Detection

Conflicts are detected during tree parsing. When a node's resolved version differs from its requested version (e.g., `2.13.0 -> 2.14.2`), a `DependencyConflict` is recorded with the coordinate, both versions, and the requesting parent.

- Detected in `tree_parser::parse()` during initial parse
- Stored in `DependencyTree.conflicts`
- Reported via `conflict_report::report()` (text or JSON)
- Text output groups conflicts by coordinate, showing each version resolution

## Scope Validation

Detects test libraries that have leaked into production dependency scopes.

**How it works:**
1. Checks if the tree's configuration is a production scope (`compileClasspath`, `runtimeClasspath`, `implementation`, `compileOnly`, `runtimeOnly`, `api`)
2. Walks all nodes and matches group/artifact against 34 known test libraries
3. Returns `ScopeValidationResult` with the matched library name and a recommendation to move to `testImplementation` or `testRuntimeOnly`

**Recognized libraries include:** JUnit 4/5, TestNG, Mockito, MockK, AssertJ, Hamcrest, WireMock, Testcontainers, Cucumber, Spock, Robolectric, Spring Test, REST Assured, Awaitility, ArchUnit, and more.

**Matching rules:**
- Some libraries match on group + artifact (e.g., `junit:junit`)
- Others match on group prefix (e.g., any `org.mockito:*` artifact)

Source: `src/analysis/scope_validator.rs`

## Duplicate Detection

Two types of duplicates are detected:

### Cross-Module Duplicates

Finds the same dependency coordinate appearing in multiple module subtrees within a multi-module project.

- Examines top-level children of each module root node
- Groups by coordinate across modules
- Flags version mismatches between modules
- Recommends either "Consolidate to root project" or "Version mismatch -- standardize"

### Within-Module Duplicates

Finds the same dependency declared multiple times in a single `build.gradle(.kts)` file.

- Parses build files using `build_file_parser`
- Groups declarations by coordinate
- Reports line numbers where each duplicate appears
- Recommends removing the duplicate declaration

Source: `src/analysis/duplicate_detector.rs`

## Dependency Diff

Compares two `DependencyTree` instances and classifies each dependency as added, removed, version-changed, or unchanged.

**How it works:**
1. Builds a coordinate-to-version summary map for each tree (preferring non-omitted, non-constraint nodes)
2. Iterates baseline entries: if present in current with different effective version, marks as `VersionChanged`; if absent, marks as `Removed`
3. Iterates current entries: if absent from baseline, marks as `Added`
4. Results sorted by coordinate

**Input flexibility:** each side of a diff can be a live Gradle project directory or an exported file (JSON or text). The `resolve_tree()` function auto-detects the input type.

**Filtering:** the `--changes` flag accepts a comma-separated list to show only specific change kinds (e.g., `--changes added,removed`). By default, unchanged entries are hidden.

Source: `src/analysis/diff_calculator.rs`

## Flat Table Generation

Flattens the dependency tree into a deduplicated list of `FlatDependencyEntry` records.

Each entry contains:
- Coordinate, group, artifact, version
- Whether it has a conflict
- Occurrence count (how many times it appears in the tree)
- Used-by list (parent coordinates)
- All observed versions

The `--conflicts-only` flag on the `table` subcommand filters to entries with version conflicts.

Source: `src/analysis/table_calculator.rs`
