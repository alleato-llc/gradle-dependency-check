use std::collections::HashMap;

use crate::dependency::models::{
    ChangeKind, DependencyDiffEntry, DependencyDiffResult, DependencyNode, DependencyTree,
};

pub fn diff(baseline: &DependencyTree, current: &DependencyTree) -> DependencyDiffResult {
    let baseline_map = build_summary_map(baseline);
    let current_map = build_summary_map(current);

    let mut entries = Vec::new();

    // Check baseline entries
    for (coord, (before_req, before_res)) in &baseline_map {
        if let Some((after_req, after_res)) = current_map.get(coord) {
            let before_effective = before_res.as_deref().or(Some(before_req.as_str()));
            let after_effective = after_res.as_deref().or(Some(after_req.as_str()));

            let change_kind = if before_effective == after_effective {
                ChangeKind::Unchanged
            } else {
                ChangeKind::VersionChanged
            };

            entries.push(DependencyDiffEntry {
                coordinate: coord.clone(),
                change_kind,
                before_version: Some(before_req.clone()),
                after_version: Some(after_req.clone()),
                before_resolved_version: before_res.clone(),
                after_resolved_version: after_res.clone(),
            });
        } else {
            entries.push(DependencyDiffEntry {
                coordinate: coord.clone(),
                change_kind: ChangeKind::Removed,
                before_version: Some(before_req.clone()),
                after_version: None,
                before_resolved_version: before_res.clone(),
                after_resolved_version: None,
            });
        }
    }

    // Check for added entries
    for (coord, (after_req, after_res)) in &current_map {
        if !baseline_map.contains_key(coord) {
            entries.push(DependencyDiffEntry {
                coordinate: coord.clone(),
                change_kind: ChangeKind::Added,
                before_version: None,
                after_version: Some(after_req.clone()),
                before_resolved_version: None,
                after_resolved_version: after_res.clone(),
            });
        }
    }

    entries.sort_by(|a, b| a.coordinate.cmp(&b.coordinate));

    DependencyDiffResult {
        baseline_name: baseline.project_name.clone(),
        current_name: current.project_name.clone(),
        entries,
    }
}

/// Builds a map of coordinate → (requested_version, resolved_version).
/// Prefers non-omitted, non-constraint nodes.
fn build_summary_map(tree: &DependencyTree) -> HashMap<String, (String, Option<String>)> {
    let mut map: HashMap<String, (String, Option<String>)> = HashMap::new();

    fn visit(
        node: &DependencyNode,
        map: &mut HashMap<String, (String, Option<String>)>,
    ) {
        let coord = node.coordinate();
        let dominated = node.is_omitted || node.is_constraint;

        if !map.contains_key(&coord) || !dominated {
            map.insert(
                coord,
                (
                    node.requested_version.clone(),
                    node.resolved_version.clone(),
                ),
            );
        }

        for child in &node.children {
            visit(child, map);
        }
    }

    for root in &tree.roots {
        visit(root, &mut map);
    }
    map
}
