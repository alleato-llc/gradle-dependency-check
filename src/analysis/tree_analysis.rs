use std::collections::{HashMap, HashSet};

use crate::dependency::models::{DependencyConflict, DependencyNode, DependencyTree};

pub fn all_nodes(tree: &DependencyTree) -> Vec<&DependencyNode> {
    let mut result = Vec::new();
    for root in &tree.roots {
        collect_nodes(root, &mut result);
    }
    result
}

fn collect_nodes<'a>(node: &'a DependencyNode, result: &mut Vec<&'a DependencyNode>) {
    result.push(node);
    for child in &node.children {
        collect_nodes(child, result);
    }
}

pub fn unique_coordinates(tree: &DependencyTree) -> HashSet<String> {
    all_nodes(tree)
        .into_iter()
        .map(|n| n.coordinate())
        .collect()
}

pub fn subtree_sizes(tree: &DependencyTree) -> HashMap<String, usize> {
    let mut sizes: HashMap<String, usize> = HashMap::new();
    for node in all_nodes(tree) {
        let coord = node.coordinate();
        let size = node.subtree_size();
        let entry = sizes.entry(coord).or_insert(0);
        *entry = (*entry).max(size);
    }
    sizes
}

pub fn conflicts_by_coordinate(tree: &DependencyTree) -> HashMap<String, Vec<&DependencyConflict>> {
    let mut map: HashMap<String, Vec<&DependencyConflict>> = HashMap::new();
    for conflict in &tree.conflicts {
        map.entry(conflict.coordinate.clone())
            .or_default()
            .push(conflict);
    }
    map
}
