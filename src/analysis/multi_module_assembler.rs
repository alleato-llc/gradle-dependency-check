use crate::dependency::models::{
    DependencyNode, DependencyTree, GradleConfiguration, GradleModule,
};

pub fn assemble(
    project_name: &str,
    configuration: GradleConfiguration,
    module_trees: Vec<(GradleModule, DependencyTree)>,
) -> DependencyTree {
    let mut all_conflicts = Vec::new();
    let mut roots = Vec::new();

    for (module, tree) in module_trees {
        let mut synthetic = DependencyNode::new(project_name, &module.name, "module");
        synthetic.children = tree.roots;
        roots.push(synthetic);
        all_conflicts.extend(tree.conflicts);
    }

    DependencyTree {
        project_name: project_name.to_string(),
        configuration,
        roots,
        conflicts: all_conflicts,
    }
}
