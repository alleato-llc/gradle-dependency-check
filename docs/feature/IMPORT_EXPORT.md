# Import / Export

## What

Exports dependency trees as JSON or Gradle text format, and imports them back. Enables the `diff` workflow and offline analysis.

## How

### Data flow

```
Export:
  tree_export::export_json(tree) → JSON string (serde serialization)
  tree_export::export_text(tree) → Gradle ASCII tree format

Import:
  tree_export::import_tree(data, file_name, config)
    → tries JSON first (serde deserialization)
    → falls back to Gradle text (tree_parser::parse)
    → assigns IDs to deserialized nodes
    → extracts project name from filename
```

### CLI usage

```bash
# Export as JSON (default)
gradle-dependency-check export ./my-project > baseline.json

# Export as Gradle text
gradle-dependency-check export ./my-project --format text > baseline.txt

# Use exported files in diff
gradle-dependency-check diff baseline.json current.json
```

## Architecture

- **JSON format** — uses `serde` derive macros on `DependencyTree`/`DependencyNode`. Field names use camelCase (`#[serde(rename)]`) for compatibility with the Swift JSON exporter.
- **Text format** — renders ASCII tree with `+---`/`\---` connectors, `|    ` continuation, `(*)` for omitted, `(c)` for constraint.
- **Auto-detection** — `import_tree()` tries JSON first; if deserialization fails, falls back to text parsing. No format flag needed.
- **ID assignment** — JSON doesn't serialize node IDs. After import, `assign_ids()` generates fresh IDs for all nodes.
- **Filename stripping** — strips `-dependencies`, `-compileClasspath`, `-runtimeClasspath` suffixes from filenames to derive project name.

### File organization

```
src/report/tree_export.rs       — export_json(), export_text(), import_tree()
src/dependency/models.rs         — Serialize/Deserialize derives, assign_ids()
```

## Testing

- `tree_export_test` — JSON export/import, text export, auto-detection, round-trips, error cases, filename stripping
