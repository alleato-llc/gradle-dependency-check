use crate::dependency::models::DependencyTree;
use crate::error::ImportError;
use crate::parsing::tree_parser;

/// Exports a tree as JSON.
pub fn export_json(tree: &DependencyTree) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(tree)
}

/// Exports a tree as Gradle text format.
pub fn export_text(tree: &DependencyTree) -> String {
    let mut lines = Vec::new();
    let root_count = tree.roots.len();

    for (i, root) in tree.roots.iter().enumerate() {
        let is_last = i == root_count - 1;
        let connector = if is_last { "\\---" } else { "+---" };
        render_node(root, connector, "", is_last, &mut lines);
    }

    lines.join("\n")
}

fn render_node(
    node: &crate::dependency::models::DependencyNode,
    connector: &str,
    prefix: &str,
    is_last: bool,
    lines: &mut Vec<String>,
) {
    let mut label = format!("{}:{}", node.coordinate(), node.display_version());
    if node.is_omitted {
        label.push_str(" (*)");
    }
    if node.is_constraint {
        label.push_str(" (c)");
    }

    lines.push(format!("{}{} {}", prefix, connector, label));

    let child_prefix = if is_last {
        format!("{}     ", prefix)
    } else {
        format!("{}|    ", prefix)
    };

    let child_count = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        let child_is_last = i == child_count - 1;
        let child_connector = if child_is_last { "\\---" } else { "+---" };
        render_node(child, child_connector, &child_prefix, child_is_last, lines);
    }
}

/// Imports a tree from file data, auto-detecting JSON or Gradle text format.
pub fn import_tree(
    data: &[u8],
    file_name: &str,
    fallback_configuration: crate::dependency::models::GradleConfiguration,
) -> Result<DependencyTree, ImportError> {
    // Try JSON first
    if let Ok(mut tree) = serde_json::from_slice::<DependencyTree>(data) {
        tree.assign_ids();
        return Ok(tree);
    }

    // Fall back to Gradle text
    let text = std::str::from_utf8(data)
        .map_err(|_| ImportError::UnreadableFile("Not valid UTF-8".to_string()))?;

    if text.is_empty() {
        return Err(ImportError::UnreadableFile("Empty file".to_string()));
    }

    let project_name = extract_project_name(file_name);
    let tree = tree_parser::parse(text, &project_name, fallback_configuration);

    if tree.total_node_count() == 0 {
        return Err(ImportError::NoDependenciesFound);
    }

    Ok(tree)
}

fn extract_project_name(file_name: &str) -> String {
    let name = if let Some(dot_pos) = file_name.rfind('.') {
        &file_name[..dot_pos]
    } else {
        file_name
    };

    for suffix in &["-dependencies", "-compileClasspath", "-runtimeClasspath"] {
        if name.ends_with(suffix) {
            return name[..name.len() - suffix.len()].to_string();
        }
    }
    name.to_string()
}
