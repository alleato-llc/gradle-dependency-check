use std::collections::{HashMap, HashSet};

use crate::dependency::models::{DependencyNode, DependencyTree, FlatDependencyEntry};

pub fn flat_entries(tree: &DependencyTree) -> Vec<FlatDependencyEntry> {
    let parent_map = parent_map(tree);
    let conflict_coords: HashSet<String> = tree.conflicts.iter().map(|c| c.coordinate.clone()).collect();

    let mut coord_nodes: HashMap<String, Vec<&DependencyNode>> = HashMap::new();
    collect_all_nodes(&tree.roots, &mut coord_nodes);

    let mut entries: Vec<FlatDependencyEntry> = Vec::new();

    for (coordinate, nodes) in &coord_nodes {
        // Prefer non-omitted node
        let preferred = nodes
            .iter()
            .find(|n| !n.is_omitted)
            .or(nodes.first())
            .unwrap();

        let versions: HashSet<String> = nodes
            .iter()
            .map(|n| {
                n.resolved_version
                    .as_deref()
                    .unwrap_or(&n.requested_version)
                    .to_string()
            })
            .collect();

        let has_conflict = conflict_coords.contains(coordinate)
            || nodes.iter().any(|n| n.has_conflict());

        let used_by = parent_map
            .get(coordinate)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>();

        entries.push(FlatDependencyEntry {
            coordinate: coordinate.clone(),
            group: preferred.group.clone(),
            artifact: preferred.artifact.clone(),
            version: preferred
                .resolved_version
                .as_deref()
                .unwrap_or(&preferred.requested_version)
                .to_string(),
            has_conflict,
            is_omitted: preferred.is_omitted,
            occurrence_count: nodes.len(),
            used_by,
            versions,
        });
    }

    entries.sort_by(|a, b| a.coordinate.cmp(&b.coordinate));
    entries
}

pub fn parent_map(tree: &DependencyTree) -> HashMap<String, HashSet<String>> {
    let mut map: HashMap<String, HashSet<String>> = HashMap::new();
    for root in &tree.roots {
        build_parent_map(root, &mut map);
    }
    map
}

fn build_parent_map(node: &DependencyNode, map: &mut HashMap<String, HashSet<String>>) {
    let parent_coord = node.coordinate();
    for child in &node.children {
        map.entry(child.coordinate())
            .or_default()
            .insert(parent_coord.clone());
        build_parent_map(child, map);
    }
}

fn collect_all_nodes<'a>(
    nodes: &'a [DependencyNode],
    map: &mut HashMap<String, Vec<&'a DependencyNode>>,
) {
    for node in nodes {
        map.entry(node.coordinate()).or_default().push(node);
        collect_all_nodes(&node.children, map);
    }
}
