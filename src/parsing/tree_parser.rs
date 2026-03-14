use regex::Regex;
use std::sync::LazyLock;

use crate::dependency::models::{
    DependencyConflict, DependencyNode, DependencyTree, GradleConfiguration,
};

static LINE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([| ]*)[+\\]--- (.+)$").unwrap());

static DEP_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([^:]+):([^:]+):([^\s]+)(.*)$").unwrap());

struct StackEntry {
    node: DependencyNode,
    depth: usize,
}

/// Parses Gradle ASCII dependency tree output into a `DependencyTree`.
pub fn parse(output: &str, project_name: &str, configuration: GradleConfiguration) -> DependencyTree {
    let mut roots: Vec<DependencyNode> = Vec::new();
    let mut conflicts: Vec<DependencyConflict> = Vec::new();
    let mut stack: Vec<StackEntry> = Vec::new();

    for line in output.lines() {
        let Some(caps) = LINE_REGEX.captures(line) else {
            continue;
        };

        let prefix = &caps[1];
        let dep_text = caps[2].trim();

        // Skip (n) entries and project dependencies
        if dep_text.ends_with("(n)") || dep_text.starts_with("project ") {
            continue;
        }

        let depth = prefix.len() / 5;

        let Some(node) = parse_dependency(dep_text) else {
            continue;
        };

        // Track conflicts
        if node.has_conflict() {
            let requested_by = stack
                .last()
                .filter(|e| e.depth == depth.saturating_sub(1))
                .map(|e| e.node.coordinate())
                .unwrap_or_default();

            conflicts.push(DependencyConflict {
                coordinate: node.coordinate(),
                requested_version: node.requested_version.clone(),
                resolved_version: node.resolved_version.clone().unwrap_or_default(),
                requested_by,
            });
        }

        if depth == 0 {
            // Finalize previous root's stack
            finalize_stack(&mut stack, &mut roots);
            stack.push(StackEntry { node, depth });
        } else {
            // Pop entries at same or deeper depth
            while stack.last().is_some_and(|e| e.depth >= depth) {
                let child_entry = stack.pop().unwrap();
                if let Some(parent) = stack.last_mut() {
                    if parent.depth == child_entry.depth - 1 {
                        parent.node.children.push(child_entry.node);
                    } else {
                        // Re-push if not direct parent
                        stack.push(child_entry);
                        break;
                    }
                } else {
                    roots.push(child_entry.node);
                    break;
                }
            }
            stack.push(StackEntry { node, depth });
        }
    }

    // Finalize remaining stack
    finalize_stack(&mut stack, &mut roots);

    DependencyTree {
        project_name: project_name.to_string(),
        configuration,
        roots,
        conflicts,
    }
}

fn finalize_stack(stack: &mut Vec<StackEntry>, roots: &mut Vec<DependencyNode>) {
    while let Some(entry) = stack.pop() {
        if let Some(parent) = stack.last_mut() {
            parent.node.children.push(entry.node);
        } else {
            roots.push(entry.node);
        }
    }
}

fn parse_dependency(text: &str) -> Option<DependencyNode> {
    // Strip trailing markers: (*), (c)
    let is_omitted = text.contains("(*)");
    let is_constraint = text.contains("(c)");
    let clean = text
        .replace("(*)", "")
        .replace("(c)", "")
        .trim()
        .to_string();

    // Handle BOM-managed: group:artifact -> version
    if let Some((left, right)) = clean.split_once(" -> ") {
        if let Some(caps) = DEP_REGEX.captures(left.trim()) {
            let group = caps[1].to_string();
            let artifact = caps[2].to_string();
            let requested_version = caps[3].to_string();
            let resolved_version = right.trim().to_string();

            let mut node = DependencyNode::new(group, artifact, requested_version.clone());
            node.is_omitted = is_omitted;
            node.is_constraint = is_constraint;
            if resolved_version != requested_version {
                node.resolved_version = Some(resolved_version);
            }
            return Some(node);
        }

        // group:artifact -> resolvedVersion (no requested version in text)
        let parts: Vec<&str> = left.trim().splitn(3, ':').collect();
        if parts.len() >= 2 {
            let resolved = right.trim().to_string();
            let mut node = DependencyNode::new(
                parts[0].to_string(),
                parts[1].to_string(),
                resolved.clone(),
            );
            node.is_omitted = is_omitted;
            node.is_constraint = is_constraint;
            return Some(node);
        }
    }

    // Standard: group:artifact:version
    if let Some(caps) = DEP_REGEX.captures(&clean) {
        let group = caps[1].to_string();
        let artifact = caps[2].to_string();
        let version_part = caps[3].to_string();
        let rest = caps.get(4).map(|m| m.as_str()).unwrap_or("");

        // Check for -> in the rest
        let (requested, resolved) = if let Some(arrow_pos) = rest.find("->") {
            let resolved = rest[arrow_pos + 2..].trim().to_string();
            (version_part, Some(resolved))
        } else {
            (version_part, None)
        };

        let mut node = DependencyNode::new(group, artifact, requested);
        node.resolved_version = resolved;
        node.is_omitted = is_omitted;
        node.is_constraint = is_constraint;
        return Some(node);
    }

    None
}
