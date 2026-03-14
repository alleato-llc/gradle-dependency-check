use std::collections::HashSet;

use crate::dependency::models::{DependencyNode, DependencyTree};

pub fn export(tree: &DependencyTree) -> String {
    let mut lines = Vec::new();
    let mut visited = HashSet::new();

    lines.push("digraph dependencies {".to_string());
    lines.push("  rankdir=TB;".to_string());
    lines.push("  node [shape=box, style=filled];".to_string());

    for root in &tree.roots {
        emit_node(root, &mut lines, &mut visited);
    }

    lines.push("}".to_string());
    lines.join("\n")
}

fn emit_node(node: &DependencyNode, lines: &mut Vec<String>, visited: &mut HashSet<String>) {
    let node_id = sanitize_id(&node.id);

    if !visited.insert(node_id.clone()) {
        return;
    }

    let label = format!("{}\\n{}", node.coordinate(), node.display_version());
    let color = if node.has_conflict() {
        "#ffcccc"
    } else {
        "#e8f4e8"
    };

    lines.push(format!(
        "  \"{}\" [label=\"{}\", fillcolor=\"{}\"];",
        node_id, label, color
    ));

    for child in &node.children {
        let child_id = sanitize_id(&child.id);
        lines.push(format!("  \"{}\" -> \"{}\";", node_id, child_id));
        emit_node(child, lines, visited);
    }
}

fn sanitize_id(id: &str) -> String {
    id.replace('"', "\\\"")
}
