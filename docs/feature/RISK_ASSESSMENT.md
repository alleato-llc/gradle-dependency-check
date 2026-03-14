# Conflict Risk Assessment

## What

Assigns a risk level (CRITICAL, HIGH, MEDIUM, LOW, INFO) to each dependency conflict based on version distance, BOM management status, upgrade direction, and scope. Enables teams to prioritize which conflicts deserve attention rather than treating all conflicts equally.

## How

### CLI usage

```bash
# Text report with risk levels
gradle-dependency-check conflicts ./my-project --risk

# JSON report with risk fields
gradle-dependency-check conflicts ./my-project --risk --format json
```

### Data flow

```
conflicts detected during tree parsing
  → risk_calculator::assess_conflicts(tree, runner, project_path)
    → Step 1: collect unique conflict coordinates
    → Step 2: query dependencyInsight per coordinate (BOM detection)
    → Step 3: parse semver, compute base risk, apply adjustments
    → Step 4: populate risk_level + risk_reason on each conflict
  → conflict_report includes risk data in output
```

### Output example

```
org.slf4j:slf4j-api
  1.7.36 -> 2.0.17 (requested by io.micrometer:micrometer-tracing-bridge-otel) [MEDIUM]
  risk: Major version jump (1.x -> 2.x), reduced: BOM-managed
  2.0.16 -> 2.0.17 (requested by org.apache.logging.log4j:log4j-to-slf4j) [INFO]
  risk: Patch version bump (2.0.16 -> 2.0.17), reduced: BOM-managed
```

```json
{
  "coordinate": "org.slf4j:slf4j-api",
  "requestedVersion": "1.7.36",
  "resolvedVersion": "2.0.17",
  "requestedBy": "io.micrometer:micrometer-tracing-bridge-otel",
  "riskLevel": "MEDIUM",
  "riskReason": "Major version jump (1.x -> 2.x), reduced: BOM-managed"
}
```

## Architecture

### Risk levels

| Level | Meaning |
|-------|---------|
| CRITICAL | Almost certainly breaks at runtime |
| HIGH | Likely to cause subtle issues |
| MEDIUM | Could break in edge cases |
| LOW | Theoretically possible but rare |
| INFO | Expected BOM behavior, safe to ignore |

### Step 1: Base risk from version distance

Versions are parsed as semver (major.minor.patch), stripping qualifiers like `.Final`, `.RELEASE`, `-jre`, `-SNAPSHOT`, `-beta1`. Multi-segment versions like `1.9.22.1` use the first three segments.

| Condition | Base Risk |
|-----------|-----------|
| Different major version (1.x → 2.x) | HIGH |
| Same major, different minor (2.15 → 2.19) | MEDIUM |
| Same major+minor, different patch (6.2.15 → 6.2.16) | LOW |
| Only qualifier differs | INFO |

Unparseable versions default to MEDIUM.

### Step 2: Contextual adjustments

Each adjustment shifts the risk by one level, clamped to INFO..CRITICAL.

**BOM-managed (−1 level):** Queries `gradle dependencyInsight --dependency <coordinate> --configuration <config> -q` and parses the first line. If the selection reason is `(selected by rule)` or `(by constraint)`, the dependency is BOM-managed — the BOM publisher has tested version compatibility. Falls back to checking constraint nodes in the tree if the runner call fails.

**Downgrade (+1 level):** If the resolved version is lower than the requested version (semver comparison), the risk increases. A downgrade means a library gets less than it declared it needs.

**Test scope (−1 level):** If the tree's configuration is a test configuration (`testCompileClasspath`, `testRuntimeClasspath`, `testImplementation`), the risk decreases. Test-only conflicts have no production impact.

### Combined formula

```
risk = base_risk(requested, resolved)
if bom_managed:     risk -= 1
if downgrade:       risk += 1
if test_scope:      risk -= 1
risk = clamp(risk, INFO, CRITICAL)
```

### BOM detection via dependencyInsight

The `dependencyInsight` Gradle task reveals *why* a particular version was selected — information not available from `gradle dependencies` text output. The first line of output contains the selection reason:

- `org.slf4j:slf4j-api:2.0.17 (selected by rule)` → BOM-managed (Spring dependency-management plugin)
- `com.fasterxml:classmate:1.7.3 (selected by rule)` → BOM-managed
- `com.fasterxml.jackson.dataformat:jackson-dataformat-toml:2.19.4` → not BOM-managed (default conflict resolution)

Performance: ~0.4s per query with a warm Gradle daemon. Queries run sequentially — 13 unique coordinates takes ~6 seconds.

### File organization

```
src/analysis/risk_calculator.rs       — risk computation + BOM detection
src/dependency/models.rs              — RiskLevel enum, risk fields on DependencyConflict
src/runner/gradle_runner.rs           — run_dependency_insight trait method
src/runner/process_runner.rs          — dependencyInsight subprocess execution
src/report/conflict_report.rs         — risk data in text/JSON output
```

## Testing

- `risk_calculator_test` — 13 tests: major/minor/patch/qualifier classification, BOM reduction via insight, BOM reduction via constraint fallback, downgrade escalation, test scope reduction, combined adjustments, non-semver fallback, multi-segment versions, real Spring Boot scenario
- `conflict_report_test` — risk fields in text/JSON output
