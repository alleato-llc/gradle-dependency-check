use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::dependency::models::{
    DependencyTree, DuplicateDependencyResult, DuplicateKind, GradleModule,
};
use crate::parsing::build_file_parser;

pub fn detect_cross_module(tree: &DependencyTree) -> Vec<DuplicateDependencyResult> {
    let module_nodes: Vec<_> = tree
        .roots
        .iter()
        .filter(|n| n.requested_version == "module")
        .collect();

    if module_nodes.len() < 2 {
        return Vec::new();
    }

    let mut coord_modules: HashMap<String, Vec<(String, String)>> = HashMap::new();

    for module_node in &module_nodes {
        let module_name = &module_node.artifact;
        for child in &module_node.children {
            let coord = child.coordinate();
            let version = child
                .resolved_version
                .as_deref()
                .unwrap_or(&child.requested_version)
                .to_string();
            coord_modules
                .entry(coord)
                .or_default()
                .push((module_name.clone(), version));
        }
    }

    let mut results = Vec::new();

    for (coordinate, entries) in &coord_modules {
        if entries.len() < 2 {
            continue;
        }

        let modules: Vec<String> = entries.iter().map(|(m, _)| m.clone()).collect();
        let mut versions: HashMap<String, String> = HashMap::new();
        for (module, version) in entries {
            versions.insert(module.clone(), version.clone());
        }

        let unique_versions: std::collections::HashSet<&str> =
            entries.iter().map(|(_, v)| v.as_str()).collect();
        let has_mismatch = unique_versions.len() > 1;

        let recommendation = if has_mismatch {
            "Version mismatch — standardize".to_string()
        } else {
            "Consolidate to root project".to_string()
        };

        results.push(DuplicateDependencyResult {
            coordinate: coordinate.clone(),
            kind: DuplicateKind::CrossModule,
            modules,
            versions,
            has_version_mismatch: has_mismatch,
            recommendation,
        });
    }

    results.sort_by(|a, b| a.coordinate.cmp(&b.coordinate));
    results
}

pub fn detect_within_module(
    project_path: &str,
    modules: &[GradleModule],
) -> Vec<DuplicateDependencyResult> {
    let mut results = Vec::new();

    let module_paths: Vec<(String, Option<String>)> = if modules.is_empty() {
        vec![("root".to_string(), find_build_file(Path::new(project_path)))]
    } else {
        modules
            .iter()
            .map(|module| {
                let relative = module.path.trim_start_matches(':').replace(':', "/");
                let module_dir = format!("{}/{}", project_path, relative);
                (module.name.clone(), find_build_file(Path::new(&module_dir)))
            })
            .collect()
    };

    for (module_name, build_file_path) in module_paths {
        let Some(path) = build_file_path else {
            continue;
        };
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };

        let declarations = build_file_parser::parse(&content);

        let mut by_coordinate: HashMap<String, Vec<_>> = HashMap::new();
        for decl in &declarations {
            let coord = format!("{}:{}", decl.group, decl.artifact);
            by_coordinate.entry(coord).or_default().push(decl);
        }

        for (coordinate, decls) in &by_coordinate {
            if decls.len() < 2 {
                continue;
            }

            let mut versions: HashMap<String, String> = HashMap::new();
            for decl in decls {
                versions.insert(
                    format!("{} (line {})", decl.configuration, decl.line),
                    decl.version.clone(),
                );
            }

            results.push(DuplicateDependencyResult {
                coordinate: coordinate.clone(),
                kind: DuplicateKind::WithinModule,
                modules: vec![module_name.clone()],
                versions,
                has_version_mismatch: false,
                recommendation: format!(
                    "Declared {} times in {} — remove duplicate declaration",
                    decls.len(),
                    module_name
                ),
            });
        }
    }

    results.sort_by(|a, b| a.coordinate.cmp(&b.coordinate));
    results
}

pub fn detect(
    tree: &DependencyTree,
    project_path: &str,
    modules: &[GradleModule],
) -> Vec<DuplicateDependencyResult> {
    let mut results = detect_cross_module(tree);
    results.extend(detect_within_module(project_path, modules));
    results.sort_by(|a, b| a.coordinate.cmp(&b.coordinate));
    results
}

fn find_build_file(dir: &Path) -> Option<String> {
    let kts = dir.join("build.gradle.kts");
    if kts.exists() {
        return Some(kts.to_string_lossy().to_string());
    }
    let groovy = dir.join("build.gradle");
    if groovy.exists() {
        return Some(groovy.to_string_lossy().to_string());
    }
    None
}
