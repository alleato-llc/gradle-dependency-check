use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// MARK: - GradleConfiguration

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GradleConfiguration {
    CompileClasspath,
    RuntimeClasspath,
    ImplementationDependenciesMetadata,
    TestCompileClasspath,
    TestRuntimeClasspath,
    AnnotationProcessor,
    CompileOnly,
    RuntimeOnly,
    Implementation,
    TestImplementation,
    Api,
}

impl GradleConfiguration {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "compileClasspath" => Some(Self::CompileClasspath),
            "runtimeClasspath" => Some(Self::RuntimeClasspath),
            "implementationDependenciesMetadata" => Some(Self::ImplementationDependenciesMetadata),
            "testCompileClasspath" => Some(Self::TestCompileClasspath),
            "testRuntimeClasspath" => Some(Self::TestRuntimeClasspath),
            "annotationProcessor" => Some(Self::AnnotationProcessor),
            "compileOnly" => Some(Self::CompileOnly),
            "runtimeOnly" => Some(Self::RuntimeOnly),
            "implementation" => Some(Self::Implementation),
            "testImplementation" => Some(Self::TestImplementation),
            "api" => Some(Self::Api),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CompileClasspath => "compileClasspath",
            Self::RuntimeClasspath => "runtimeClasspath",
            Self::ImplementationDependenciesMetadata => "implementationDependenciesMetadata",
            Self::TestCompileClasspath => "testCompileClasspath",
            Self::TestRuntimeClasspath => "testRuntimeClasspath",
            Self::AnnotationProcessor => "annotationProcessor",
            Self::CompileOnly => "compileOnly",
            Self::RuntimeOnly => "runtimeOnly",
            Self::Implementation => "implementation",
            Self::TestImplementation => "testImplementation",
            Self::Api => "api",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::CompileClasspath => "Compile Classpath",
            Self::RuntimeClasspath => "Runtime Classpath",
            Self::ImplementationDependenciesMetadata => "Implementation Dependencies Metadata",
            Self::TestCompileClasspath => "Test Compile Classpath",
            Self::TestRuntimeClasspath => "Test Runtime Classpath",
            Self::AnnotationProcessor => "Annotation Processor",
            Self::CompileOnly => "Compile Only",
            Self::RuntimeOnly => "Runtime Only",
            Self::Implementation => "Implementation",
            Self::TestImplementation => "Test Implementation",
            Self::Api => "API",
        }
    }

    pub fn all() -> &'static [GradleConfiguration] {
        &[
            Self::CompileClasspath,
            Self::RuntimeClasspath,
            Self::ImplementationDependenciesMetadata,
            Self::TestCompileClasspath,
            Self::TestRuntimeClasspath,
            Self::AnnotationProcessor,
            Self::CompileOnly,
            Self::RuntimeOnly,
            Self::Implementation,
            Self::TestImplementation,
            Self::Api,
        ]
    }

    pub fn is_production(&self) -> bool {
        matches!(
            self,
            Self::CompileClasspath
                | Self::RuntimeClasspath
                | Self::Implementation
                | Self::RuntimeOnly
                | Self::CompileOnly
                | Self::Api
        )
    }
}

impl std::fmt::Display for GradleConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// MARK: - DependencyNode

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    #[serde(skip)]
    pub id: String,
    pub group: String,
    pub artifact: String,
    #[serde(rename = "requestedVersion")]
    pub requested_version: String,
    #[serde(rename = "resolvedVersion", skip_serializing_if = "Option::is_none")]
    pub resolved_version: Option<String>,
    #[serde(rename = "isOmitted")]
    pub is_omitted: bool,
    #[serde(rename = "isConstraint")]
    pub is_constraint: bool,
    pub children: Vec<DependencyNode>,
}

impl DependencyNode {
    pub fn new(
        group: impl Into<String>,
        artifact: impl Into<String>,
        requested_version: impl Into<String>,
    ) -> Self {
        let group = group.into();
        let artifact = artifact.into();
        let requested_version = requested_version.into();
        let id = format!(
            "{}:{}:{}:{}",
            group,
            artifact,
            requested_version,
            uuid_v4()
        );
        Self {
            id,
            group,
            artifact,
            requested_version,
            resolved_version: None,
            is_omitted: false,
            is_constraint: false,
            children: Vec::new(),
        }
    }

    pub fn coordinate(&self) -> String {
        format!("{}:{}", self.group, self.artifact)
    }

    pub fn has_conflict(&self) -> bool {
        self.resolved_version
            .as_ref()
            .is_some_and(|rv| rv != &self.requested_version)
    }

    pub fn display_version(&self) -> String {
        match &self.resolved_version {
            Some(rv) => format!("{} -> {}", self.requested_version, rv),
            None => self.requested_version.clone(),
        }
    }

    pub fn subtree_size(&self) -> usize {
        1 + self.children.iter().map(|c| c.subtree_size()).sum::<usize>()
    }

    /// Assigns IDs after deserialization (JSON doesn't include them).
    pub fn assign_ids(&mut self) {
        if self.id.is_empty() {
            self.id = format!(
                "{}:{}:{}:{}",
                self.group,
                self.artifact,
                self.requested_version,
                uuid_v4()
            );
        }
        for child in &mut self.children {
            child.assign_ids();
        }
    }
}

// MARK: - DependencyConflict

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DependencyConflict {
    pub coordinate: String,
    #[serde(rename = "requestedVersion")]
    pub requested_version: String,
    #[serde(rename = "resolvedVersion")]
    pub resolved_version: String,
    #[serde(rename = "requestedBy")]
    pub requested_by: String,
}

// MARK: - DependencyTree

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyTree {
    #[serde(rename = "projectName")]
    pub project_name: String,
    pub configuration: GradleConfiguration,
    pub roots: Vec<DependencyNode>,
    pub conflicts: Vec<DependencyConflict>,
}

impl DependencyTree {
    pub fn total_node_count(&self) -> usize {
        self.roots.iter().map(|r| r.subtree_size()).sum()
    }

    pub fn max_depth(&self) -> usize {
        fn depth(node: &DependencyNode) -> usize {
            if node.children.is_empty() {
                1
            } else {
                1 + node.children.iter().map(|c| depth(c)).max().unwrap_or(0)
            }
        }
        self.roots.iter().map(|r| depth(r)).max().unwrap_or(0)
    }

    /// Assigns IDs to all nodes (needed after JSON deserialization).
    pub fn assign_ids(&mut self) {
        for root in &mut self.roots {
            root.assign_ids();
        }
    }
}

// MARK: - GradleModule

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GradleModule {
    pub name: String,
    pub path: String,
}

// MARK: - FlatDependencyEntry

#[derive(Debug, Clone, Serialize)]
pub struct FlatDependencyEntry {
    pub coordinate: String,
    pub group: String,
    pub artifact: String,
    pub version: String,
    pub has_conflict: bool,
    pub is_omitted: bool,
    pub occurrence_count: usize,
    pub used_by: Vec<String>,
    pub versions: HashSet<String>,
}

// MARK: - ScopeValidationResult

#[derive(Debug, Clone, Serialize)]
pub struct ScopeValidationResult {
    pub coordinate: String,
    pub version: String,
    pub matched_library: String,
    pub configuration: GradleConfiguration,
    pub recommendation: String,
}

// MARK: - DuplicateDependencyResult

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DuplicateKind {
    CrossModule,
    WithinModule,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuplicateDependencyResult {
    pub coordinate: String,
    pub kind: DuplicateKind,
    pub modules: Vec<String>,
    pub versions: HashMap<String, String>,
    pub has_version_mismatch: bool,
    pub recommendation: String,
}

// MARK: - DependencyDiffEntry

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ChangeKind {
    Added,
    Removed,
    VersionChanged,
    Unchanged,
}

impl ChangeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Removed => "removed",
            Self::VersionChanged => "versionChanged",
            Self::Unchanged => "unchanged",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DependencyDiffEntry {
    pub coordinate: String,
    pub change_kind: ChangeKind,
    pub before_version: Option<String>,
    pub after_version: Option<String>,
    pub before_resolved_version: Option<String>,
    pub after_resolved_version: Option<String>,
}

impl DependencyDiffEntry {
    pub fn effective_before_version(&self) -> Option<&str> {
        self.before_resolved_version
            .as_deref()
            .or(self.before_version.as_deref())
    }

    pub fn effective_after_version(&self) -> Option<&str> {
        self.after_resolved_version
            .as_deref()
            .or(self.after_version.as_deref())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DependencyDiffResult {
    pub baseline_name: String,
    pub current_name: String,
    pub entries: Vec<DependencyDiffEntry>,
}

impl DependencyDiffResult {
    pub fn added(&self) -> Vec<&DependencyDiffEntry> {
        self.entries
            .iter()
            .filter(|e| e.change_kind == ChangeKind::Added)
            .collect()
    }

    pub fn removed(&self) -> Vec<&DependencyDiffEntry> {
        self.entries
            .iter()
            .filter(|e| e.change_kind == ChangeKind::Removed)
            .collect()
    }

    pub fn version_changed(&self) -> Vec<&DependencyDiffEntry> {
        self.entries
            .iter()
            .filter(|e| e.change_kind == ChangeKind::VersionChanged)
            .collect()
    }

    pub fn unchanged(&self) -> Vec<&DependencyDiffEntry> {
        self.entries
            .iter()
            .filter(|e| e.change_kind == ChangeKind::Unchanged)
            .collect()
    }
}

// MARK: - DependencyDeclaration (for build file parsing)

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DependencyDeclaration {
    pub configuration: String,
    pub group: String,
    pub artifact: String,
    pub version: String,
    pub line: usize,
}

// MARK: - ReportFormat

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Text,
    Json,
}

// MARK: - UUID helper

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let random = nanos ^ (std::process::id() as u128) ^ ((&nanos as *const u128 as u128) << 32);
    format!("{:032x}", random)
}
