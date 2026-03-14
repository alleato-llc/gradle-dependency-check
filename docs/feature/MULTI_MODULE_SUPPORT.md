# Multi-Module Support

## What

Discovers and loads dependencies from multi-module Gradle projects, assembling per-module trees into a unified tree with synthetic module root nodes.

## How

### Data flow

```
tree_loader::load_tree(runner, path, config, module)
  → runner.list_projects() → Vec<GradleModule>
  → if modules.is_empty(): single-module path
  → if modules exist: for each module:
      → runner.run_module_dependencies() → output
      → tree_parser::parse() → per-module DependencyTree
  → multi_module_assembler::assemble()
      → creates synthetic node per module (requested_version = "module")
      → merges all conflicts
      → returns unified DependencyTree
```

### CLI usage

```bash
# Auto-discover and load all modules
gradle-dependency-check conflicts ./my-project

# Load specific module only
gradle-dependency-check conflicts ./my-project --module :app

# List discovered modules
gradle-dependency-check conflicts ./my-project --list-modules
```

## Architecture

- **Module discovery** — `GradleRunner.list_projects()` runs `gradle projects -q` and parses output with `project_list_parser`.
- **Synthetic nodes** — each module becomes a root node with `requested_version == "module"`, children are the module's actual dependencies. This enables cross-module analysis.
- **Specific module** — `--module :app` bypasses discovery and loads only that module's tree.
- **Gradle path mapping** — module path `:app:feature` maps to filesystem path `app/feature/` for build file parsing.

### File organization

```
src/analysis/tree_loader.rs              — orchestrates loading (trait-based)
src/analysis/multi_module_assembler.rs   — assembles per-module trees
src/parsing/project_list_parser.rs       — parses gradle projects output
src/runner/gradle_runner.rs              — GradleRunner trait
src/runner/process_runner.rs             — ProcessGradleRunner (subprocess)
```

## Testing

- `multi_module_assembler_test` — two-module assembly, synthetic nodes, conflict aggregation, empty modules
- `project_list_parser_test` — standard output, nested submodules, empty output
- `tree_loader_test` — single/multi/specific module loading via TestGradleRunner fake
